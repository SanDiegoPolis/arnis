use crate::coordinate_system::geographic::{LLBBox, LLPoint};
use crate::mapillary::geometry::parse_length_m;
use crate::osm_parser::{OsmElement, OsmMember};
use crate::overture::overture_class_to_osm_building;
use geo::algorithm::{coordinate_position::CoordPos, dimensions::Dimensions, Validation};
use geo::{Area, BoundingRect, Contains, Coord, Intersects, LineString, Polygon, Rect, Relate};
use serde_json::Value;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

// Bit 59 distinguishes imported elements. Bit 44 also survives the shifts
// used to derive internal way IDs from a building relation's ID.
const IMPORTED_ID_BASE: u64 = (1 << 59) | (1 << 44);

pub(crate) fn is_imported(id: u64) -> bool {
    id >> 44 == IMPORTED_ID_BASE >> 44
}

/// Read a WGS84 building layer into the same raw element format as OSM buildings.
pub(crate) fn load(
    path: &Path,
    bbox: &LLBBox,
) -> Result<Vec<OsmElement>, Box<dyn Error + Send + Sync>> {
    let file = File::open(path).map_err(|err| format!("{}: {err}", path.display()))?;
    let doc = serde_json::from_reader(BufReader::new(file))
        .map_err(|err| format!("{}: invalid GeoJSON: {err}", path.display()))?;
    parse(&doc, bbox).map_err(|err| format!("{}: {err}", path.display()).into())
}

fn parse(doc: &Value, bbox: &LLBBox) -> Result<Vec<OsmElement>, String> {
    check_crs(doc)?;
    let extent = Rect::new(
        (bbox.min().lng(), bbox.min().lat()),
        (bbox.max().lng(), bbox.max().lat()),
    );
    let mut output = Vec::new();
    let mut next_id = IMPORTED_ID_BASE;
    if doc["type"] == "FeatureCollection" {
        let features = doc["features"]
            .as_array()
            .ok_or("FeatureCollection must have a features array")?;
        for (index, feature) in features.iter().enumerate() {
            if feature["type"] != "Feature" {
                return Err(format!("feature {}: expected a GeoJSON Feature", index + 1));
            }
            import_feature(feature, &extent, &mut next_id, &mut output)
                .map_err(|err| format!("feature {}: {err}", index + 1))?;
        }
    } else {
        import_feature(doc, &extent, &mut next_id, &mut output)?;
    }
    if output.is_empty() {
        return Err("no building footprints intersect the selected area".into());
    }
    Ok(output)
}

fn check_crs(value: &Value) -> Result<(), String> {
    if value["crs"].is_null() {
        return Ok(());
    }
    let crs = &value["crs"];
    let name = crs["properties"]["name"].as_str().unwrap_or_default();
    if crs["type"] == "name"
        && [
            "EPSG:4326",
            "urn:ogc:def:crs:EPSG::4326",
            "urn:ogc:def:crs:OGC:1.3:CRS84",
            "urn:ogc:def:crs:OGC::CRS84",
            "OGC:CRS84",
            "CRS84",
        ]
        .iter()
        .any(|wgs84| name.eq_ignore_ascii_case(wgs84))
    {
        return Ok(());
    }
    Err("the GeoJSON must use WGS84 coordinates in [longitude, latitude] order".into())
}

fn import_feature(
    feature: &Value,
    extent: &Rect<f64>,
    next_id: &mut u64,
    output: &mut Vec<OsmElement>,
) -> Result<(), String> {
    check_crs(feature)?;
    let geometry = if feature["type"] == "Feature" {
        &feature["geometry"]
    } else {
        feature
    };
    check_crs(geometry)?;
    let coordinates = &geometry["coordinates"];
    let polygons = match geometry["type"].as_str() {
        Some("Polygon") => std::slice::from_ref(coordinates),
        Some("MultiPolygon") => coordinates
            .as_array()
            .ok_or("MultiPolygon coordinates must be an array of polygons")?,
        _ => return Ok(()),
    };
    let properties = &feature["properties"];
    if properties["building"] == "no" {
        return Ok(());
    }
    let tags = building_tags(properties);
    for polygon in polygons {
        let polygon = parse_polygon(polygon)?;
        // Reserve the same IDs for a building regardless of the selected area.
        let id = *next_id;
        *next_id += polygon.exterior().0.len() as u64
            + polygon
                .interiors()
                .iter()
                .map(|ring| ring.0.len() as u64)
                .sum::<u64>()
            + u64::from(!polygon.interiors().is_empty());
        let bounds = polygon
            .bounding_rect()
            .expect("checked building ring is nonempty");
        if !bounds.intersects(extent) {
            continue;
        }
        polygon
            .check_validation()
            .map_err(|err| format!("invalid building polygon: {err}"))?;
        if polygon.unsigned_area() == 0.0 {
            return Err("a building polygon must enclose an area".into());
        }
        if extent.contains(&bounds)
            || polygon
                .relate(extent)
                .get(CoordPos::Inside, CoordPos::Inside)
                == Dimensions::TwoDimensional
        {
            write_polygon(polygon, tags.clone(), id, output);
        }
    }
    Ok(())
}

fn parse_polygon(value: &Value) -> Result<Polygon<f64>, String> {
    let mut rings = value
        .as_array()
        .ok_or("Polygon coordinates must be an array of rings")?
        .iter();
    let exterior = parse_ring(rings.next().ok_or("Polygon has no exterior ring")?)?;
    let interiors = rings.map(parse_ring).collect::<Result<_, _>>()?;
    Ok(Polygon::new(exterior, interiors))
}

fn parse_ring(value: &Value) -> Result<LineString<f64>, String> {
    let positions = value
        .as_array()
        .ok_or("a ring must be an array of positions")?;
    let mut coords = Vec::with_capacity(positions.len());
    for position in positions {
        let (Some(x), Some(y)) = (position[0].as_f64(), position[1].as_f64()) else {
            return Err("each position must contain [longitude, latitude] numbers".into());
        };
        LLPoint::new(y, x).map_err(|err| format!("expected WGS84 coordinates: {err}"))?;
        let coord = Coord { x, y };
        if coords.last() != Some(&coord) {
            coords.push(coord);
        }
    }
    if coords.len() < 4 || coords.first() != coords.last() {
        return Err("a building ring must have at least three vertices and repeat its first position at the end".into());
    }
    Ok(LineString(coords))
}

fn building_tags(properties: &Value) -> HashMap<String, String> {
    let text = |key: &str| {
        properties[key]
            .as_str()
            .map(str::trim)
            .filter(|value| !value.is_empty())
    };
    let building = text("building").unwrap_or_else(|| {
        match overture_class_to_osm_building(text("subtype"), text("class")) {
            "yes" => match text("Function") {
                Some("Office") => "office",
                Some("Residence") => "residential",
                Some("Public service") => "public",
                Some("Business") => "commercial",
                _ => "yes",
            },
            mapped => mapped,
        }
    });
    let mut tags = HashMap::from([("building".to_string(), building.to_string())]);
    for (tag, keys, metric) in [
        (
            "height",
            &["height", "Height", "height_m", "pred_h_r"][..],
            true,
        ),
        (
            "building:levels",
            &["building:levels", "num_floors", "Floor"][..],
            false,
        ),
        ("roof:height", &["roof:height", "roof_height"][..], true),
    ] {
        if let Some(number) = keys.iter().find_map(|key| {
            let value = &properties[*key];
            let number = value.as_f64().or_else(|| {
                let value = value.as_str()?.trim();
                if metric {
                    parse_length_m(value)
                } else {
                    value.parse().ok()
                }
            })?;
            (number > 0.0 && number <= 1000.0).then_some(number)
        }) {
            tags.insert(tag.to_string(), number.to_string());
        }
    }
    for (tag, alias) in [
        ("name", "name"),
        ("roof:shape", "roof_shape"),
        ("roof:material", "roof_material"),
        ("roof:colour", "roof_color"),
        ("building:material", "facade_material"),
        ("building:colour", "facade_color"),
    ] {
        if let Some(value) = text(tag).or_else(|| text(alias)) {
            tags.insert(tag.to_string(), value.to_string());
        }
    }
    tags
}

fn write_polygon(
    polygon: Polygon<f64>,
    mut tags: HashMap<String, String>,
    mut id: u64,
    output: &mut Vec<OsmElement>,
) {
    let (exterior, interiors) = polygon.into_inner();
    if interiors.is_empty() {
        write_ring(exterior, Some(tags), &mut id, output);
        return;
    }
    let members = std::iter::once(("outer", exterior))
        .chain(interiors.into_iter().map(|ring| ("inner", ring)))
        .map(|(role, ring)| OsmMember {
            r#type: "way".into(),
            r#ref: write_ring(ring, None, &mut id, output),
            r#role: role.into(),
        })
        .collect();
    tags.insert("type".into(), "multipolygon".into());
    output.push(OsmElement {
        r#type: "relation".into(),
        id,
        lat: None,
        lon: None,
        nodes: None,
        tags: Some(tags),
        members,
    });
}

fn write_ring(
    ring: LineString<f64>,
    tags: Option<HashMap<String, String>>,
    next_id: &mut u64,
    output: &mut Vec<OsmElement>,
) -> u64 {
    let points = &ring.0[..ring.0.len() - 1];
    let first_node = *next_id;
    for point in points {
        output.push(OsmElement {
            r#type: "node".into(),
            id: *next_id,
            lat: Some(point.y),
            lon: Some(point.x),
            nodes: None,
            tags: None,
            members: Vec::new(),
        });
        *next_id += 1;
    }
    let id = *next_id;
    *next_id += 1;
    output.push(OsmElement {
        r#type: "way".into(),
        id,
        lat: None,
        lon: None,
        nodes: Some(
            (first_node..id)
                .chain(std::iter::once(first_node))
                .collect(),
        ),
        tags,
        members: Vec::new(),
    });
    id
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::collections::HashSet;

    fn shanghai() -> LLBBox {
        LLBBox::new(31.22, 121.45, 31.24, 121.49).unwrap()
    }

    fn feature_with_courtyard() -> Value {
        json!({
            "type": "Feature",
            "properties": {"Height": "7.5 m", "num_floors": 2, "roof_shape": "flat"},
            "geometry": {
                "type": "MultiPolygon",
                "coordinates": [
                    [[[116.3, 39.9], [116.4, 39.9], [116.4, 40], [116.3, 40], [116.3, 39.9]]],
                    [
                        [[121.46, 31.23], [121.48, 31.23], [121.48, 31.239], [121.46, 31.239], [121.46, 31.23]],
                        [[121.465, 31.232], [121.465, 31.236], [121.47, 31.236], [121.47, 31.232], [121.465, 31.232]]
                    ]
                ]
            }
        })
    }

    #[test]
    fn preserves_courtyard_and_height_and_filters_other_parts() {
        let elements = parse(&feature_with_courtyard(), &shanghai()).unwrap();
        assert_eq!(elements.len(), 11);
        assert_eq!(elements[0].lat, Some(31.23));
        assert_eq!(elements[0].lon, Some(121.46));
        let relation = elements.last().unwrap();
        assert_eq!(relation.r#type, "relation");
        let tags = relation.tags.as_ref().unwrap();
        assert_eq!(tags["height"], "7.5");
        assert_eq!(tags["building:levels"], "2");
        assert_eq!(tags["roof:shape"], "flat");
        assert_eq!(relation.members[0].r#role, "outer");
        assert_eq!(relation.members[1].r#role, "inner");
        for member in &relation.members {
            let way = elements.iter().find(|el| el.id == member.r#ref).unwrap();
            assert!(way.tags.is_none());
            let ids = way.nodes.as_ref().unwrap();
            assert_eq!(ids.first(), ids.last());
            assert_eq!(ids.len(), 5);
        }
        assert_eq!(
            elements
                .iter()
                .map(|el| el.id)
                .collect::<HashSet<_>>()
                .len(),
            elements.len()
        );
        assert!(elements.iter().all(|el| is_imported(el.id)));
    }

    #[test]
    fn keeps_ids_when_selection_changes_and_selects_crossing_polygons() {
        let feature = feature_with_courtyard();
        let both = LLBBox::new(31.0, 116.0, 40.5, 122.0).unwrap();
        let crossing = LLBBox::new(31.233, 121.475, 31.235, 121.485).unwrap();
        let full = parse(&feature, &both).unwrap();
        let part = parse(&feature, &crossing).unwrap();
        assert_eq!(full[4].r#type, "way");
        assert_eq!(full[4].tags.as_ref().unwrap()["building"], "yes");
        assert_eq!(full.last().unwrap().id, part.last().unwrap().id);

        let in_hole = LLBBox::new(31.233, 121.466, 31.235, 121.469).unwrap();
        assert!(parse(&feature, &in_hole)
            .unwrap_err()
            .contains("no building"));
        let touches_edge = LLBBox::new(31.233, 121.48, 31.235, 121.485).unwrap();
        assert!(parse(&feature, &touches_edge)
            .unwrap_err()
            .contains("no building"));
    }

    #[test]
    fn rejects_malformed_geometry_and_incompatible_crs() {
        let mut feature = feature_with_courtyard();
        feature["geometry"]["coordinates"][1][0][4] = json!([121.46, 31.24]);
        let error = parse(&feature, &shanghai()).unwrap_err();
        assert!(error.contains("repeat its first position"), "{error}");

        feature["crs"] = json!({"type":"name","properties":{"name":"EPSG:3857"}});
        assert!(parse(&feature, &shanghai()).unwrap_err().contains("WGS84"));
    }

    #[test]
    fn reads_cmab_height_and_floor_fields() {
        let tags = building_tags(&json!({"pred_h_r": 6.72, "Floor": "2"}));
        assert_eq!(tags["height"], "6.72");
        assert_eq!(tags["building:levels"], "2");
    }

    #[test]
    fn maps_surveyed_function_and_preserves_explicit_building_type() {
        let tags = building_tags(&json!({"Function": "Office"}));
        assert_eq!(tags["building"], "office");
        let tags = building_tags(&json!({"Function": "Office", "building": "temple"}));
        assert_eq!(tags["building"], "temple");
    }
}

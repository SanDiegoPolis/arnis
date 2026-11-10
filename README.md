<img src="assets/git/banner.png" width="100%" alt="Banner">

# Arnis [![CI Build Status](https://github.com/louis-e/arnis/actions/workflows/ci-build.yml/badge.svg)](https://github.com/louis-e/arnis/actions) [<img alt="GitHub Release" src="https://img.shields.io/github/v/release/louis-e/arnis" />](https://github.com/louis-e/arnis/releases) [<img alt="GitHub Downloads (all assets, all releases" src="https://img.shields.io/github/downloads/louis-e/arnis/total" />](https://github.com/louis-e/arnis/releases) [![Download here](https://img.shields.io/badge/Download-here-green)](https://github.com/louis-e/arnis/releases) [![Discord](https://img.shields.io/discord/1326192999738249267?label=Discord&color=%237289da)](https://discord.gg/mA2g69Fhxq)

Arnis creates complex and accurate Minecraft Java Edition (1.21.4+), Bedrock Edition, and Luanti (Minetest) worlds that reflect real-world geography, topography, and architecture.

This free and open source project is designed to handle large-scale geographic data from the real world and generate detailed Minecraft worlds. The algorithm processes geospatial data from OpenStreetMap as well as elevation data to create an accurate Minecraft representation of terrain and architecture.
Generate your hometown, big cities, and natural landscapes with ease!

_**Want mobile generation or larger map sizes?** [MapSmith](https://arnismc.com/mapsmith/) generates worlds in your browser, no install required._

![Minecraft Preview](assets/git/preview.jpg)
<i>This GitHub page and [arnismc.com](https://arnismc.com) are the only official project websites. Do not download Arnis from any other website.</i>

## :keyboard: Usage
<img width="60%" src="assets/git/gui.png"><br>
Download the [latest release](https://github.com/louis-e/arnis/releases/) or [compile](#trophy-open-source) the project on your own.

Choose your area on the map using the rectangle tool, then simply click on <i>Start Generation</i>!
Additionally, you can customize various generation settings, such as world scale, spawn point, or building interior generation.

### Use local building data

If the default map is missing buildings or has inaccurate heights, go to **Settings → Generation → Local Building GeoJSON**. This replaces OpenStreetMap and Overture building footprints across the selected area. Choose an area fully covered by your file. Roads and other map features come from OpenStreetMap.

Use WGS84 GeoJSON with longitude first, including `Polygon` or `MultiPolygon` footprints. For height above ground in metres, use `height` or `height_m`. For storeys, use `num_floors` or `building:levels`. The importer also reads `pred_h_r` and `Floor` from the [CMAB China building dataset](https://doi.org/10.6084/m9.figshare.27992417.v2). Arnis estimates the height when the file supplies neither height nor storeys.

You can combine footprints from multiple sources in one file. If the local source only covers part of a landmark, include its more complete footprint in the import layer. Copy heights between datasets only where the outlines match the same building.

In QGIS, export the building layer as GeoJSON using **EPSG:4326 — WGS 84**. If you use GDAL, you can convert a Shapefile with the command below. For a city-wide dataset, export just the neighbourhood you want to generate. If the source uses GCJ-02 or BD-09, convert from that coordinate system to WGS84 before exporting.

```sh
ogr2ogr -t_srs EPSG:4326 -lco RFC7946=YES buildings.geojson buildings.shp
```

On the command line, use `--buildings-geojson` with a bounding box or a local OSM file. For example, for a neighbourhood in Shanghai:

```sh
arnis --output-dir "./saves" --bbox "31.232,121.489,31.243,121.505" --buildings-geojson buildings.geojson
```

Local building data is available for standalone Earth worlds with buildings enabled at scale 0.3 or above. For an offline map-data run, also pass `--file area.osm` to supply the roads and other features locally.

For high-rise districts, the **Extend build height** setting or `--disable-height-limit` in Java Edition lets buildings exceed the vanilla height limit.

## 📚 Documentation

<img src="assets/git/documentation.png" width="100%" alt="Banner">

Full documentation is available in the [GitHub Wiki](https://github.com/louis-e/arnis/wiki/), covering topics such as generation settings, command line usage, technical explanations, FAQs and contribution guidelines.

[backgroundvid.webm](https://github.com/user-attachments/assets/420acc19-a850-418e-8397-1a45b05582ab)

## :trophy: Open Source
#### Key objectives of this project
- **Modularity**: Ensure that all components (e.g., data fetching, processing, and world generation) are cleanly separated into distinct modules for better maintainability and scalability.
- **Performance Optimization**: We aim to maintain strong performance and fast world generation.
- **Comprehensive Documentation**: Detailed in-code documentation for a clear structure and logic.
- **User-Friendly Experience**: Focus on making the project easy to use for end users.
- **Cross-Platform Support**: We want this project to run smoothly on Windows, macOS, and Linux.

#### How to contribute
This project is open source and welcomes contributions from everyone! Whether you're interested in fixing bugs, improving performance, adding new features, or enhancing documentation, your input is valuable. Simply fork the repository, make your changes, and submit a pull request. Please respect the above-mentioned key objectives. Contributions of all levels are appreciated, and your efforts help improve this tool for everyone.

Command line Build: ```cargo run --release --no-default-features -- --output-dir="C:/YOUR_PATH/.minecraft/saves" --bbox="min_lat,min_lng,max_lat,max_lng"```<br>
GUI Build: ```cargo run --release```<br>

All command line options, like the generation modes, winter and One World, are explained on the [Command Line Usage](https://github.com/louis-e/arnis/wiki/Command-Line-Usage) wiki page.

After your pull request is merged, I will take care of regularly creating update releases which will include your changes.

If you are using Nix, you can run the program directly with `nix run github:louis-e/arnis -- --output-dir=YOUR_PATH/.minecraft/saves --bbox="min_lat,min_lng,max_lat,max_lng"`

## :star: Star History

<a href="https://star-history.com/#louis-e/arnis&Date">
 <picture>
   <source media="(prefers-color-scheme: dark)" srcset="https://api.star-history.com/svg?repos=louis-e/arnis&Date&theme=dark" />
   <source media="(prefers-color-scheme: light)" srcset="https://api.star-history.com/svg?repos=louis-e/arnis&Date&type=Date" />
   <img alt="Star History Chart" src="https://api.star-history.com/svg?repos=louis-e/arnis&Date&type=Date" />
 </picture>
</a>

## :newspaper: Academic & Press Recognition

<img src="assets/git/recognition.png" width="100%" alt="Banner">

Arnis has been recognized in various academic and press publications after gaining more attention in December 2024.

[Building realistic Minecraft worlds with Open Data on AWS: How Arnis uses elevation datasets at scale](https://aws.amazon.com/de/blogs/publicsector/building-realistic-minecraft-worlds-with-open-data-on-aws-how-arnis-uses-elevation-datasets-at-scale/)

[Floodcraft: Game-based Interactive Learning Environment using Minecraft for Flood Mitigation and Preparedness for K-12 Education](https://www.researchgate.net/publication/384644535_Floodcraft_Game-based_Interactive_Learning_Environment_using_Minecraft_for_Flood_Mitigation_and_Preparedness_for_K-12_Education)

[Hackaday: Bringing OpenStreetMap Data into Minecraft](https://hackaday.com/2024/12/30/bringing-openstreetmap-data-into-minecraft/)

[TomsHardware: Minecraft Tool Lets You Create Scale Replicas of Real-World Locations](https://www.tomshardware.com/video-games/pc-gaming/minecraft-tool-lets-you-create-scale-replicas-of-real-world-locations-arnis-uses-geospatial-data-from-openstreetmap-to-generate-minecraft-maps)

[XDA Developers: Hometown Minecraft Map: Arnis](https://www.xda-developers.com/hometown-minecraft-map-arnis/)

Free to use press assets, including screenshots and logos, can be found [here](https://arnismc.com/presskit/).

## :lock: Code Signing Policy

Free code signing provided by [SignPath.io](https://about.signpath.io/), certificate by [SignPath Foundation](https://signpath.org/).

The Windows executable is built from this repository by the [release workflow](.github/workflows/release.yml) on GitHub-hosted runners and is only signed after a maintainer approves the signing request. All roles (committer, reviewer and approver) are held by [louis-&#8288;e](https://github.com/louis-e).

Privacy policy: https://arnismc.com/privacypolicy.html

## :copyright: License Information
Copyright (c) 2022-2026 Louis Erbkamm (louis-e)

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.[^3]

The Luanti block-name mapping in `src/luanti_block_map.rs` is derived from [MC2MT](https://github.com/rollerozxa/MC2MT) by rollerozxa and is licensed under the GNU Lesser General Public License v2.1 or later. The full attribution and license header are preserved in that file.

Download Arnis only from the official source https://arnismc.com or https://github.com/louis-e/arnis/. Every other website providing a download and claiming to be affiliated with the project is unofficial and may be malicious.

The logo was made by @nxfx21.

NOT AN OFFICIAL MINECRAFT PRODUCT. NOT APPROVED BY OR ASSOCIATED WITH MOJANG OR MICROSOFT.


[^1]: https://en.wikipedia.org/wiki/OpenStreetMap

[^2]: https://en.wikipedia.org/wiki/Arnis,_Germany

[^3]: https://github.com/louis-e/arnis/blob/main/LICENSE

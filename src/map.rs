//! Multi-LOD KMA chart. Player position is lon/lat; tiles are derived per zoom.

use serde::Deserialize;

use crate::data::MAP_JSON;

pub const ZOOM_KMA: u8 = 0;
pub const ZOOM_ISLAND: u8 = 1;
pub const ZOOM_HARBOR: u8 = 2;
pub const ZOOM_MAX: u8 = 2;

#[derive(Debug, Clone, Deserialize)]
pub struct MapData {
    pub attribution: String,
    pub kma: View,
    pub lods: Vec<LodRaster>,
    pub harbor: HarborLod,
    pub labels: Vec<Label>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct View {
    pub south: f64,
    pub north: f64,
    pub west: f64,
    pub east: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LodRaster {
    pub id: String,
    pub meters_per_tile: f64,
    pub view: View,
    pub width: i32,
    pub height: i32,
    pub tiles: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HarborLod {
    pub id: String,
    pub meters_per_tile: f64,
    pub coast: Vec<Vec<[f64; 2]>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Label {
    pub name: String,
    pub lon: f64,
    pub lat: f64,
    pub kind: String,
    pub min_lod: u8,
}

pub fn load() -> MapData {
    serde_json::from_str(MAP_JSON).expect("kodiak_map.json")
}

pub fn zoom_name(z: u8) -> &'static str {
    match z {
        ZOOM_KMA => "KMA ~2km",
        ZOOM_ISLAND => "island ~500m",
        _ => "harbor ~80m",
    }
}

pub fn meters_per_tile(map: &MapData, z: u8) -> f64 {
    match z {
        ZOOM_KMA => map.lods.first().map(|l| l.meters_per_tile).unwrap_or(2000.0),
        ZOOM_ISLAND => map.lods.get(1).map(|l| l.meters_per_tile).unwrap_or(500.0),
        _ => map.harbor.meters_per_tile,
    }
}

pub fn clamp_kma(map: &MapData, lon: f64, lat: f64) -> (f64, f64) {
    (
        lon.clamp(map.kma.west, map.kma.east),
        lat.clamp(map.kma.south, map.kma.north),
    )
}

pub fn step_deg(lat: f64, meters: f64) -> (f64, f64) {
    let dlat = meters / 111_320.0;
    let dlon = meters / (111_320.0 * lat.to_radians().cos().max(0.2));
    (dlon, dlat)
}

pub fn haversine_m(lon1: f64, lat1: f64, lon2: f64, lat2: f64) -> f64 {
    let r = 6_371_000.0;
    let p1 = lat1.to_radians();
    let p2 = lat2.to_radians();
    let dp = (lat2 - lat1).to_radians();
    let dl = (lon2 - lon1).to_radians();
    let a = (dp / 2.0).sin().powi(2) + p1.cos() * p2.cos() * (dl / 2.0).sin().powi(2);
    2.0 * r * a.sqrt().asin()
}

pub fn lonlat_to_xy(view: &View, w: i32, h: i32, lon: f64, lat: f64) -> (f64, f64) {
    let x = (lon - view.west) / (view.east - view.west) * (w - 1) as f64;
    let y = (view.north - lat) / (view.north - view.south) * (h - 1) as f64;
    (x, y)
}

pub fn tile_char(lod: &LodRaster, x: i32, y: i32) -> char {
    if x < 0 || y < 0 || x >= lod.width || y >= lod.height {
        return ' ';
    }
    lod.tiles[y as usize].chars().nth(x as usize).unwrap_or(' ')
}

/// Sample a pre-raster LOD (0 or 1).
pub fn sample_lod(lod: &LodRaster, lon: f64, lat: f64) -> char {
    let (x, y) = lonlat_to_xy(&lod.view, lod.width, lod.height, lon, lat);
    tile_char(lod, x.round() as i32, y.round() as i32)
}

fn draw_seg(buf: &mut [Vec<char>], x0: f64, y0: f64, x1: f64, y1: f64) {
    let h = buf.len() as i32;
    let w = buf[0].len() as i32;
    let n = ((x1 - x0).abs().max((y1 - y0).abs()) as i32).max(1);
    for i in 0..=n {
        let t = i as f64 / n as f64;
        let x = (x0 + (x1 - x0) * t).round() as i32;
        let y = (y0 + (y1 - y0) * t).round() as i32;
        if x >= 0 && y >= 0 && x < w && y < h {
            buf[y as usize][x as usize] = '#';
        }
    }
}

/// Harbor LOD: rasterize OSM coastline into the current viewport (not a stretched coarse grid).
pub fn raster_harbor(
    map: &MapData,
    lon0: f64,
    lat0: f64,
    vw: i32,
    vh: i32,
) -> (View, Vec<String>) {
    let m = map.harbor.meters_per_tile;
    let (dlon, dlat) = step_deg(lat0, m);
    let view = View {
        west: lon0 - dlon * (vw as f64 / 2.0),
        east: lon0 + dlon * (vw as f64 / 2.0),
        north: lat0 + dlat * (vh as f64 / 2.0),
        south: lat0 - dlat * (vh as f64 / 2.0),
    };
    let mut buf = vec![vec!['?'; vw as usize]; vh as usize];

    // Seed from island LOD so interiors stay land when zoomed onto an island.
    if let Some(island) = map.lods.get(1) {
        for y in 0..vh {
            for x in 0..vw {
                let lon = view.west + (x as f64 / (vw - 1).max(1) as f64) * (view.east - view.west);
                let lat = view.north - (y as f64 / (vh - 1).max(1) as f64) * (view.north - view.south);
                let ch = sample_lod(island, lon, lat);
                buf[y as usize][x as usize] = if ch == ' ' { '~' } else { ch };
            }
        }
    }

    for line in &map.harbor.coast {
        if line.len() < 2 {
            continue;
        }
        let mut prev: Option<(f64, f64)> = None;
        for p in line {
            let (x, y) = lonlat_to_xy(&view, vw, vh, p[0], p[1]);
            if let Some((px, py)) = prev {
                // skip segments wholly outside a padded box
                if (x < -2.0 && px < -2.0)
                    || (y < -2.0 && py < -2.0)
                    || (x > vw as f64 + 2.0 && px > vw as f64 + 2.0)
                    || (y > vh as f64 + 2.0 && py > vh as f64 + 2.0)
                {
                    prev = Some((x, y));
                    continue;
                }
                draw_seg(&mut buf, px, py, x, y);
            }
            prev = Some((x, y));
        }
    }

    // Recolor land/water boundary as coast
    let mut out = vec![vec!['~'; vw as usize]; vh as usize];
    for y in 0..vh as usize {
        for x in 0..vw as usize {
            let ch = buf[y][x];
            if ch == '#' {
                out[y][x] = '#';
            } else if ch == '.' {
                let mut coast = false;
                for (dx, dy) in [(-1isize, 0), (1, 0), (0, -1), (0, 1)] {
                    let nx = x as isize + dx;
                    let ny = y as isize + dy;
                    if nx >= 0
                        && ny >= 0
                        && nx < vw as isize
                        && ny < vh as isize
                        && buf[ny as usize][nx as usize] == '~'
                    {
                        coast = true;
                    }
                }
                out[y][x] = if coast { '#' } else { '.' };
            } else {
                out[y][x] = '~';
            }
        }
    }

    (
        view,
        out.into_iter()
            .map(|row| row.into_iter().collect())
            .collect(),
    )
}

pub fn raster_window(
    map: &MapData,
    zoom: u8,
    lon: f64,
    lat: f64,
    vw: i32,
    vh: i32,
) -> (View, Vec<String>, i32, i32) {
    if zoom >= ZOOM_HARBOR {
        let (view, tiles) = raster_harbor(map, lon, lat, vw, vh);
        let cx = vw / 2;
        let cy = vh / 2;
        return (view, tiles, cx, cy);
    }
    let lod = if zoom == ZOOM_KMA {
        &map.lods[0]
    } else {
        map.lods.get(1).unwrap_or(&map.lods[0])
    };
    let (fx, fy) = lonlat_to_xy(&lod.view, lod.width, lod.height, lon, lat);
    let px = fx.round() as i32;
    let py = fy.round() as i32;
    let x0 = (px - vw / 2).clamp(0, (lod.width - vw).max(0));
    let y0 = (py - vh / 2).clamp(0, (lod.height - vh).max(0));
    let mut tiles = Vec::new();
    for y in y0..y0 + vh {
        let mut row = String::new();
        for x in x0..x0 + vw {
            row.push(tile_char(lod, x, y));
        }
        tiles.push(row);
    }
    // Geographic box of the *window*, not the full LOD, so sites/labels line up.
    let west = lod.view.west
        + (x0 as f64 / (lod.width - 1).max(1) as f64) * (lod.view.east - lod.view.west);
    let east = lod.view.west
        + ((x0 + vw - 1) as f64 / (lod.width - 1).max(1) as f64) * (lod.view.east - lod.view.west);
    let north = lod.view.north
        - (y0 as f64 / (lod.height - 1).max(1) as f64) * (lod.view.north - lod.view.south);
    let south = lod.view.north
        - ((y0 + vh - 1) as f64 / (lod.height - 1).max(1) as f64) * (lod.view.north - lod.view.south);
    let view = View {
        west,
        east,
        north,
        south,
    };
    (view, tiles, px - x0, py - y0)
}

pub fn labels_in_view<'a>(
    map: &'a MapData,
    view: &View,
    zoom: u8,
    vw: i32,
    vh: i32,
) -> Vec<(&'a Label, i32, i32)> {
    let mut out = Vec::new();
    for lab in &map.labels {
        if lab.min_lod > zoom {
            continue;
        }
        if lab.lon < view.west || lab.lon > view.east || lab.lat < view.south || lab.lat > view.north
        {
            continue;
        }
        let (x, y) = lonlat_to_xy(view, vw, vh, lab.lon, lab.lat);
        let xi = x.round() as i32;
        let yi = y.round() as i32;
        if xi >= 0 && yi >= 0 && xi < vw && yi < vh {
            out.push((lab, xi, yi));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn atlas_has_three_lods_and_kma_clip() {
        let m = load();
        assert!(m.attribution.contains("OpenStreetMap"));
        assert!(m.attribution.contains("5 AAC 18.100"));
        assert_eq!(m.lods.len(), 2);
        assert_eq!(m.lods[0].id, "kma");
        assert_eq!(m.lods[1].id, "island");
        assert!(m.harbor.coast.len() > 10);
        assert!(m.kma.west <= -156.3);
        assert!(m.kma.east >= -150.0);
        assert!(m.kma.north >= 58.85);
        assert!(m.kma.south <= 55.51);
    }

    #[test]
    fn osm_names_only_no_invented_towns() {
        let m = load();
        assert!(m.labels.iter().any(|l| l.name == "Uganik Bay" && l.kind == "bay"));
        assert!(m.labels.iter().any(|l| l.name == "Kodiak" && l.kind == "town"));
        assert!(m
            .labels
            .iter()
            .any(|l| l.name == "Saint Paul Harbor" && l.kind == "harbor"));
        assert!(m.labels.iter().any(|l| l.name == "Womens Bay" && l.kind == "bay"));
        for l in &m.labels {
            assert!(!l.name.to_lowercase().contains("district"));
            assert!(!l.name.to_lowercase().contains("statistical"));
            if l.name == "Ugak Bay" || l.name == "Alitak Bay" {
                assert_eq!(l.kind, "bay", "{} must be a bay", l.name);
            }
            if l.name == "Ugak" || l.name == "Alitak" {
                panic!("bare {} is not an OSM town", l.name);
            }
        }
    }

    #[test]
    fn harbor_raster_is_not_stretch_of_kma() {
        let m = load();
        // Saint Paul Harbor (OSM) — should resolve water + coast, not a solid glyph.
        let (_v, tiles, ..) = raster_window(&m, ZOOM_HARBOR, -152.441, 57.767, 80, 24);
        let kma_ch = sample_lod(&m.lods[0], -152.407, 57.790);
        let harbor_unique: std::collections::HashSet<char> =
            tiles.iter().flat_map(|r| r.chars()).collect();
        assert!(harbor_unique.contains(&'~'));
        assert!(kma_ch == '.' || kma_ch == '#' || kma_ch == '~');
        // Harbor window should mix coast/water, not a single glyph.
        assert!(harbor_unique.len() >= 2);
    }

    #[test]
    fn zoom_step_moves_less_at_harbor() {
        let (d0, _) = step_deg(57.8, 2000.0);
        let (d2, _) = step_deg(57.8, 80.0);
        assert!(d2 < d0 / 10.0);
    }
}

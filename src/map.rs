use serde::Deserialize;

use crate::data::MAP_JSON;

#[derive(Debug, Clone, Deserialize)]
pub struct MapData {
    pub attribution: String,
    pub view: View,
    pub width: i32,
    pub height: i32,
    pub tiles: Vec<String>,
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
pub struct Label {
    pub name: String,
    pub lon: f64,
    pub lat: f64,
    pub x: i32,
    pub y: i32,
}

pub fn load() -> MapData {
    serde_json::from_str(MAP_JSON).expect("kodiak_map.json")
}

pub fn latlon_to_tile(map: &MapData, lon: f64, lat: f64) -> (i32, i32) {
    let x = ((lon - map.view.west) / (map.view.east - map.view.west) * (map.width - 1) as f64).round()
        as i32;
    let y = ((map.view.north - lat) / (map.view.north - map.view.south) * (map.height - 1) as f64)
        .round() as i32;
    (x, y)
}

pub fn tile_to_latlon(map: &MapData, x: i32, y: i32) -> (f64, f64) {
    let lon = map.view.west
        + (x as f64 / (map.width - 1) as f64) * (map.view.east - map.view.west);
    let lat = map.view.north
        - (y as f64 / (map.height - 1) as f64) * (map.view.north - map.view.south);
    (lon, lat)
}

pub fn in_bounds(map: &MapData, x: i32, y: i32) -> bool {
    x >= 0 && y >= 0 && x < map.width && y < map.height
}

pub fn tile(map: &MapData, x: i32, y: i32) -> char {
    if !in_bounds(map, x, y) {
        return ' ';
    }
    map.tiles[y as usize].chars().nth(x as usize).unwrap_or(' ')
}

pub fn is_water(ch: char) -> bool {
    ch == '~'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labels_are_osm_names() {
        let m = load();
        assert!(m.attribution.contains("OpenStreetMap"));
        assert!(m.labels.iter().any(|l| l.name == "Uganik Bay"));
        assert!(m.labels.iter().any(|l| l.name == "Kodiak"));
        assert!(m.labels.iter().any(|l| l.name == "Amook Pass"));
        for l in &m.labels {
            assert!(!l.name.to_lowercase().contains("district"));
            assert!(!l.name.to_lowercase().contains("statistical"));
        }
    }

    #[test]
    fn kodiak_town_is_on_land_or_coast() {
        let m = load();
        let k = m.labels.iter().find(|l| l.name == "Kodiak").unwrap();
        let ch = tile(&m, k.x, k.y);
        assert!(ch == '.' || ch == '#' || ch == '~', "got {ch}");
    }
}

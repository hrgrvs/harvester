use serde::Deserialize;

pub const SEASON_JSON: &str = include_str!("../data/season_2025.json");
pub const WEATHER_JSON: &str = include_str!("../data/weather_2025.json");
pub const MAP_JSON: &str = include_str!("../data/kodiak_map.json");

#[derive(Debug, Clone, Deserialize)]
pub struct SeasonSummary {
    pub title: String,
    pub released: String,
    pub source: String,
    pub note: String,
    pub harvest: SpeciesCounts,
    pub harvest_10yr_avg: SpeciesCounts,
    pub forecast_2025: SpeciesCounts,
    pub exvessel_common_property_usd: i64,
    pub run_notes: Vec<String>,
    pub prices_preliminary_2025: PricesBlock,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SpeciesCounts {
    pub chinook: u32,
    pub sockeye: u32,
    pub coho: u32,
    pub pink: u32,
    pub chum: u32,
    pub total: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PricesBlock {
    pub source: String,
    pub chinook: Price,
    pub sockeye: Price,
    pub coho: Price,
    pub pink: Price,
    pub chum: Price,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Price {
    pub lb: f64,
    pub usd_per_lb: f64,
}

pub fn season() -> SeasonSummary {
    serde_json::from_str(SEASON_JSON).expect("season_2025.json")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn official_totals_match_season_summary() {
        let s = season();
        assert_eq!(s.harvest.chinook, 1_315);
        assert_eq!(s.harvest.sockeye, 1_436_881);
        assert_eq!(s.harvest.coho, 315_681);
        assert_eq!(s.harvest.pink, 34_605_698);
        assert_eq!(s.harvest.chum, 740_485);
        assert_eq!(s.harvest.total, 37_100_060);
        assert_eq!(
            s.harvest.chinook + s.harvest.sockeye + s.harvest.coho + s.harvest.pink + s.harvest.chum,
            s.harvest.total
        );
    }
}

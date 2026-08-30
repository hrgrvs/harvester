use chrono::NaiveDate;
use serde::Deserialize;
use std::collections::HashMap;

use crate::data::WEATHER_JSON;

#[derive(Debug, Clone, Deserialize)]
pub struct DayWeather {
    pub date: String,
    pub padq_temp_f: Option<f64>,
    pub padq_wind_kt: Option<f64>,
    pub padq_gust_kt: Option<f64>,
    pub padq_dir: Option<String>,
    pub padq_vsby_mi: Option<f64>,
    pub padq_wx: Option<String>,
    pub ndbc_wind_kt: Option<f64>,
    pub ndbc_gust_kt: Option<f64>,
    pub ndbc_dir: Option<String>,
    pub ndbc_seas_ft: Option<f64>,
    pub ndbc_air_f: Option<f64>,
    pub ndbc_sst_f: Option<f64>,
    pub cwfaer: String,
    pub wind_kt: f64,
    pub seas_ft: f64,
    pub dir: String,
    pub temp_f: f64,
    pub fishable: bool,
    pub sources: String,
}

pub fn load() -> HashMap<String, DayWeather> {
    serde_json::from_str(WEATHER_JSON).expect("weather_2025.json")
}

pub fn for_date(table: &HashMap<String, DayWeather>, date: NaiveDate) -> Option<&DayWeather> {
    table.get(&date.format("%Y-%m-%d").to_string())
}

pub fn line(w: &DayWeather) -> String {
    let padq = format!(
        "PADQ {:.0}F {} {:.0}kt{}",
        w.temp_f,
        w.padq_dir.as_deref().unwrap_or(&w.dir),
        w.padq_wind_kt.unwrap_or(w.wind_kt),
        w.padq_wx
            .as_deref()
            .map(|x| format!(" {x}"))
            .unwrap_or_default()
    );
    let buoy = match (w.ndbc_wind_kt, w.ndbc_seas_ft) {
        (Some(k), Some(s)) => format!(
            "46077 {:.0}kt {} {:.0}ft",
            k,
            w.ndbc_dir.as_deref().unwrap_or("?"),
            s
        ),
        _ => "46077 (no obs this day)".to_string(),
    };
    format!("{padq}  |  {buoy}  |  {}", w.cwfaer)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn june_first_uses_padq() {
        let t = load();
        let d = NaiveDate::from_ymd_opt(2025, 6, 1).unwrap();
        let w = for_date(&t, d).expect("1 Jun 2025");
        assert!(w.padq_temp_f.is_some());
        assert!(w.sources.contains("PADQ"));
        assert!(w.cwfaer.contains("PKZ"));
    }

    #[test]
    fn july_sixth_has_buoy() {
        let t = load();
        let d = NaiveDate::from_ymd_opt(2025, 7, 6).unwrap();
        let w = for_date(&t, d).expect("6 Jul 2025");
        assert!(w.ndbc_wind_kt.is_some());
        assert!(w.ndbc_seas_ft.is_some());
    }
}

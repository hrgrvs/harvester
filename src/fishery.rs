//! Permit-scale catch. Not official week-by-district harvest (no 2025 AMR).

use chrono::{Datelike, NaiveDate};
use rand::Rng;

use crate::calendar::{opening, OpenState};
use crate::geography::{gear_legal, Gear, Section};
use crate::weather::DayWeather;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Catch {
    pub chinook: u32,
    pub sockeye: u32,
    pub coho: u32,
    pub pink: u32,
    pub chum: u32,
}

impl Catch {
    pub fn total(self) -> u32 {
        self.chinook + self.sockeye + self.coho + self.pink + self.chum
    }

    pub fn add_assign(&mut self, other: Catch) {
        self.chinook += other.chinook;
        self.sockeye += other.sockeye;
        self.coho += other.coho;
        self.pink += other.pink;
        self.chum += other.chum;
    }

    pub fn take(&mut self) -> Catch {
        let c = *self;
        *self = Catch::default();
        c
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FishError {
    Closed,
    IllegalGear,
    Weather,
    NotOnSite,
    EngineDead,
    NetTangled,
}

/// Species timing 0..1 from well-known KMA run timing (not harvest counts).
fn timing(species: &str, date: NaiveDate) -> f64 {
    let doy = date.ordinal() as i32;
    let bell = |peak: i32, width: f64| {
        let z = (doy - peak) as f64 / width;
        (-0.5 * z * z).exp()
    };
    match species {
        "chinook" => bell(165, 14.0) * 0.35, // incidental; very weak 2025
        "sockeye_early" => bell(170, 16.0),
        "sockeye_late" => bell(215, 18.0),
        "pink" => bell(205, 18.0),
        "chum" => bell(200, 16.0),
        "coho" => bell(240, 16.0),
        _ => 0.0,
    }
}

/// Qualitative 2025 run-strength modifiers from the season summary.
fn strength(section: Section, species: &str) -> f64 {
    match species {
        "chinook" => 0.12, // lowest in 43 years except 1989
        "sockeye_early" => match section {
            Section::Central | Section::OuterKarluk | Section::InnerKarluk => 0.28, // Karluk very weak
            Section::OuterAyakulik | Section::InnerAyakulik => 1.05, // average; BEG exceeded
            Section::AlitakBay | Section::MoserBay | Section::OlgaBay | Section::CapeAlitak => 0.35, // Frazer weak
            _ => 0.55,
        },
        "sockeye_late" => match section {
            Section::Central | Section::OuterKarluk => 0.70,
            Section::OuterAyakulik => 0.45,
            Section::AlitakBay | Section::MoserBay | Section::OlgaBay => 0.85,
            _ => 0.65,
        },
        "pink" => match section {
            Section::Central | Section::Eastside | Section::Northeast => 1.55, // particularly strong
            Section::KitoiComplex => 1.20,
            _ => 1.25,
        },
        "chum" => match section {
            Section::Central => 1.35, // several NW runs particularly strong
            Section::KitoiComplex => 1.15,
            _ => 0.90,
        },
        "coho" => match section {
            Section::KitoiComplex => 1.40,
            _ => 1.10,
        },
        _ => 1.0,
    }
}

fn poissonish(rng: &mut impl Rng, lambda: f64) -> u32 {
    if lambda <= 0.0 {
        return 0;
    }
    // Simple draw: round a gamma-ish jitter around lambda.
    let jitter = rng.gen_range(0.65..1.35);
    (lambda * jitter).round().max(0.0) as u32
}

pub fn can_fish(
    date: NaiveDate,
    gear: Gear,
    section: Section,
    weather: &DayWeather,
    on_legal_site: bool,
    engine_dead: bool,
    net_tangled: bool,
) -> Result<(), FishError> {
    let after_sept4 = date >= NaiveDate::from_ymd_opt(2025, 9, 5).unwrap();
    if !gear_legal(gear, section, after_sept4) {
        return Err(FishError::IllegalGear);
    }
    if opening(date, section) != OpenState::Open {
        return Err(FishError::Closed);
    }
    if !weather.fishable {
        return Err(FishError::Weather);
    }
    if gear == Gear::Setnet && !on_legal_site {
        return Err(FishError::NotOnSite);
    }
    if gear == Gear::PurseSeine && engine_dead {
        return Err(FishError::EngineDead);
    }
    if net_tangled {
        return Err(FishError::NetTangled);
    }
    Ok(())
}

/// One fishing day for a single permit. Not an official district harvest.
/// `crew_factor` is 0.35–1.0 from sleep / hunger / motivation (exhausted hands pick less).
pub fn fish_day(
    rng: &mut impl Rng,
    date: NaiveDate,
    gear: Gear,
    section: Section,
    weather: &DayWeather,
    crew_factor: f64,
) -> Catch {
    let weather_mod = if weather.wind_kt >= 25.0 || weather.seas_ft >= 8.0 {
        0.35
    } else if weather.wind_kt >= 18.0 {
        0.70
    } else {
        1.0
    };
    let gear_mod = match gear {
        Gear::Setnet => 1.0,
        Gear::PurseSeine => 6.5,
    };
    let crew_mod = crew_factor.clamp(0.35, 1.0);
    // Peak setnet pink day ~120 fish in a strong 2025 NW opening; scaled by timing.
    let base = 90.0 * gear_mod * weather_mod * crew_mod;

    let sockeye = (timing("sockeye_early", date) * strength(section, "sockeye_early")
        + timing("sockeye_late", date) * strength(section, "sockeye_late"))
        * base
        * 0.22;
    let pink = timing("pink", date) * strength(section, "pink") * base;
    let chum = timing("chum", date) * strength(section, "chum") * base * 0.18;
    let coho = timing("coho", date) * strength(section, "coho") * base * 0.12;
    let chinook = timing("chinook", date) * strength(section, "chinook") * base * 0.04;

    let mut c = Catch {
        chinook: poissonish(rng, chinook),
        sockeye: poissonish(rng, sockeye),
        coho: poissonish(rng, coho),
        pink: poissonish(rng, pink),
        chum: poissonish(rng, chum),
    };
    // 5 AAC / EO: purse seine non-retention Chinook ≥ 28" areawide all season.
    if gear == Gear::PurseSeine {
        c.chinook = 0;
    }
    c
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::weather::load;
    use rand::SeedableRng;

    #[test]
    fn cannot_fish_closed_or_illegal() {
        let w = load();
        let d = NaiveDate::from_ymd_opt(2025, 6, 15).unwrap();
        let weather = w.get("2025-06-15").unwrap();
        assert_eq!(
            can_fish(d, Gear::Setnet, Section::Central, weather, true, false, false),
            Err(FishError::Closed)
        );
        let d2 = NaiveDate::from_ymd_opt(2025, 7, 7).unwrap();
        let weather2 = w.get("2025-07-07").unwrap();
        assert_eq!(
            can_fish(d2, Gear::Setnet, Section::Eastside, weather2, true, false, false),
            Err(FishError::IllegalGear)
        );
        assert!(can_fish(d2, Gear::Setnet, Section::Central, weather2, true, false, false).is_ok());
        assert_eq!(
            can_fish(d2, Gear::Setnet, Section::Central, weather2, false, false, false),
            Err(FishError::NotOnSite)
        );
    }

    #[test]
    fn seine_releases_chinook() {
        let mut rng = rand::rngs::StdRng::seed_from_u64(1);
        let w = load();
        let weather = w.get("2025-07-07").unwrap();
        let d = NaiveDate::from_ymd_opt(2025, 7, 7).unwrap();
        let c = fish_day(&mut rng, d, Gear::PurseSeine, Section::Eastside, weather, 1.0);
        assert_eq!(c.chinook, 0);
    }

    #[test]
    fn exhausted_crew_pick_fewer() {
        let w = load();
        let weather = w.get("2025-07-07").unwrap();
        let d = NaiveDate::from_ymd_opt(2025, 7, 7).unwrap();
        let mut fresh = rand::rngs::StdRng::seed_from_u64(9);
        let mut tired = rand::rngs::StdRng::seed_from_u64(9);
        let a = fish_day(&mut fresh, d, Gear::Setnet, Section::Central, weather, 1.0);
        let b = fish_day(&mut tired, d, Gear::Setnet, Section::Central, weather, 0.40);
        assert!(b.total() <= a.total());
    }
}

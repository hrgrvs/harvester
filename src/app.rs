use chrono::NaiveDate;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::collections::HashMap;

use crate::calendar::{bulletin, opening, season_end, season_start};
use crate::crew::Company;
use crate::economy::{format_money, value_cents};
use crate::fishery::{can_fish, fish_day, Catch, FishError};
use crate::geography::{
    camps_for, gear_legal, nearest_site, section_at, site_by_id, site_tile, Gear, Section, Site,
    SITES,
};
use crate::map::{self, latlon_to_tile, tile_to_latlon, MapData};
use crate::weather::{self, DayWeather};
use crate::wildlife::{self, WildlifeEvent};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Title,
    NewGame,
    PickSite,
    Play,
    Town,
    Almanac,
    Help,
    GameOver,
}

#[derive(Debug, Clone)]
pub struct Game {
    pub date: NaiveDate,
    pub gear: Gear,
    pub camp_id: String,
    pub x: i32,
    pub y: i32,
    pub cash: i64,
    pub hold: Catch,
    pub landed: Catch,
    pub fuel: i32,
    pub company: Company,
    pub engine_dead: bool,
    pub net_tangled: bool,
    pub log: Vec<String>,
    pub over: Option<String>,
}

pub struct App {
    pub screen: Screen,
    pub map: MapData,
    pub weather: HashMap<String, DayWeather>,
    pub game: Option<Game>,
    pub pick_gear: usize,
    pub pick_site: usize,
    pub log_scroll: usize,
    pub almanac_scroll: usize,
    rng: StdRng,
}

impl App {
    pub fn new(seed: u64) -> Self {
        Self {
            screen: Screen::Title,
            map: map::load(),
            weather: weather::load(),
            game: None,
            pick_gear: 0,
            pick_site: 0,
            log_scroll: 0,
            almanac_scroll: 0,
            rng: StdRng::seed_from_u64(seed),
        }
    }

    pub fn gear_choices() -> [(Gear, &'static str); 2] {
        [
            (Gear::Setnet, "S04K set gillnet — Central Section + inner Alitak"),
            (Gear::PurseSeine, "S01K purse seine — mobile (beach seine is not default)"),
        ]
    }

    pub fn current_camps(&self) -> Vec<&'static Site> {
        let g = Self::gear_choices()[self.pick_gear].0;
        camps_for(g)
    }

    fn push_log(game: &mut Game, msg: impl Into<String>) {
        game.log.push(msg.into());
        if game.log.len() > 80 {
            let n = game.log.len() - 80;
            game.log.drain(0..n);
        }
    }

    pub fn start_game(&mut self) {
        let gear = Self::gear_choices()[self.pick_gear].0;
        let camps = camps_for(gear);
        let site = camps[self.pick_site.min(camps.len().saturating_sub(1))];
        let (x, y) = site_tile(&self.map, site);
        let cash = match gear {
            Gear::Setnet => 1_800_000,
            Gear::PurseSeine => 4_500_000,
        };
        let names: &[&str] = match gear {
            Gear::Setnet => &["Mara", "Iosif", "Denny"],
            Gear::PurseSeine => &["Mara", "Iosif", "Denny", "Ruth"],
        };
        let mut game = Game {
            date: season_start(),
            gear,
            camp_id: site.id.to_string(),
            x,
            y,
            cash,
            hold: Catch::default(),
            landed: Catch::default(),
            fuel: 40,
            company: Company::new(names),
            engine_dead: false,
            net_tangled: false,
            log: Vec::new(),
            over: None,
        };
        Self::push_log(
            &mut game,
            format!(
                "Fish camp on {} ({}). You do not have to live in town. Site picking is the job.",
                site.osm_name,
                site.section.name()
            ),
        );
        let day = game.date;
        Self::push_log(&mut game, bulletin(day));
        self.game = Some(game);
        self.screen = Screen::Play;
    }

    pub fn weather_today(&self) -> Option<&DayWeather> {
        let g = self.game.as_ref()?;
        weather::for_date(&self.weather, g.date)
    }

    pub fn camp(&self) -> Option<&'static Site> {
        self.game.as_ref().and_then(|g| site_by_id(&g.camp_id))
    }

    pub fn here_section(&self) -> Option<Section> {
        let g = self.game.as_ref()?;
        let (lon, lat) = tile_to_latlon(&self.map, g.x, g.y);
        Some(section_at(lon, lat))
    }

    pub fn on_camp(&self) -> bool {
        let Some(g) = &self.game else { return false };
        let Some(c) = site_by_id(&g.camp_id) else {
            return false;
        };
        let (cx, cy) = site_tile(&self.map, c);
        (g.x - cx).abs() <= 2 && (g.y - cy).abs() <= 2
    }

    pub fn nearby_town(&self) -> Option<&'static Site> {
        let g = self.game.as_ref()?;
        SITES.iter().find(|s| {
            if !s.town {
                return false;
            }
            let (tx, ty) = site_tile(&self.map, s);
            (g.x - tx).abs() <= 2 && (g.y - ty).abs() <= 2
        })
    }

    pub fn move_by(&mut self, dx: i32, dy: i32) {
        let Some(g) = self.game.as_mut() else { return };
        if g.engine_dead && g.gear == Gear::PurseSeine {
            Self::push_log(g, "Engine is dead. A whale pod stove it. Tow to town or wait for a skiff.");
            return;
        }
        let nx = (g.x + dx).clamp(0, self.map.width - 1);
        let ny = (g.y + dy).clamp(0, self.map.height - 1);
        g.x = nx;
        g.y = ny;
        if g.fuel > 0 && (dx != 0 || dy != 0) && g.gear == Gear::PurseSeine {
            g.fuel -= 1;
        }
    }

    pub fn fish(&mut self) {
        let Some(weather) = self.weather_today().cloned() else {
            return;
        };
        let on_camp = self.on_camp();
        let Some(section) = self.here_section() else {
            return;
        };
        let Some(g) = self.game.as_mut() else { return };
        match can_fish(
            g.date,
            g.gear,
            section,
            &weather,
            on_camp,
            g.engine_dead,
            g.net_tangled,
        ) {
            Err(FishError::Closed) => Self::push_log(
                g,
                format!(
                    "{} is closed today. {}",
                    section.name(),
                    bulletin(g.date)
                ),
            ),
            Err(FishError::IllegalGear) => Self::push_log(
                g,
                format!(
                    "{} {} is not legal in {} (5 AAC 18.330).",
                    g.gear.permit_code(),
                    g.gear.name(),
                    section.name()
                ),
            ),
            Err(FishError::Weather) => Self::push_log(
                g,
                format!(
                    "Blown out. {} / {:.0} ft. Sit camp.",
                    weather.cwfaer, weather.seas_ft
                ),
            ),
            Err(FishError::NotOnSite) => Self::push_log(
                g,
                "Setnet fishes the site you picked. Walk back to camp (or move camp).",
            ),
            Err(FishError::EngineDead) => {
                Self::push_log(g, "No engine. Whale pod. Repair in town.")
            }
            Err(FishError::NetTangled) => {
                Self::push_log(g, "Otters still in the gear. Mend at camp (c) first.")
            }
            Ok(()) => {
                let c = fish_day(&mut self.rng, g.date, g.gear, section, &weather);
                g.hold.add_assign(c);
                Self::push_log(
                    g,
                    format!(
                        "Picked {} — {} k, {} r, {} h, {} p, {} c  (your permit, not an official district count)",
                        section.name(),
                        c.chinook, c.sockeye, c.coho, c.pink, c.chum
                    ),
                );
                match wildlife::roll(&mut self.rng, g.gear, &mut g.hold) {
                    WildlifeEvent::OttersInNet => {
                        g.net_tangled = true;
                        Self::push_log(g, "Sea otters in the net. Gear fouled — mend at camp.");
                    }
                    WildlifeEvent::SeaLionsSteal(s) => Self::push_log(
                        g,
                        format!(
                            "Steller sea lions hit the gear. Lost {} fish off the boat.",
                            s.total()
                        ),
                    ),
                    WildlifeEvent::WhaleKillsEngine => {
                        g.engine_dead = true;
                        Self::push_log(
                            g,
                            "A whale pod rolled the boat. Engine is dead. Get to town.",
                        );
                    }
                    WildlifeEvent::None => {}
                }
                g.company.day_at_camp(true);
                self.finish_day();
            }
        }
    }

    pub fn camp_day(&mut self) {
        let Some(g) = self.game.as_mut() else { return };
        if g.net_tangled {
            g.net_tangled = false;
            Self::push_log(g, "Mended the net. Otter mess cleared.");
        } else {
            Self::push_log(
                g,
                "Camp day. Crew stayed on the beach. Closures are for mending, not town.",
            );
        }
        g.company.day_at_camp(true);
        self.finish_day();
    }

    pub fn wait_day(&mut self) {
        let Some(g) = self.game.as_mut() else { return };
        g.company.day_at_camp(true);
        Self::push_log(g, "Waited out the day at the present position.");
        self.finish_day();
    }

    pub fn deliver(&mut self) {
        let Some(g) = self.game.as_mut() else { return };
        if g.hold.total() == 0 {
            Self::push_log(g, "Hold is empty.");
            return;
        }
        let c = g.hold.take();
        let cents = value_cents(c);
        g.cash += cents;
        g.landed.add_assign(c);
        Self::push_log(
            g,
            format!(
                "Tender took {} fish for {} (2025 preliminary Kodiak prices).",
                c.total(),
                format_money(cents)
            ),
        );
    }

    pub fn enter_town(&mut self) {
        if self.nearby_town().is_some() {
            self.screen = Screen::Town;
        } else if let Some(g) = self.game.as_mut() {
            Self::push_log(g, "No town here. Walk to an OSM village/city (Kodiak, Larsen Bay, …).");
        }
    }

    pub fn town_food(&mut self) {
        let Some(g) = self.game.as_mut() else { return };
        if g.cash < 12_000 {
            Self::push_log(g, "Cannot cover a grocery run.");
            return;
        }
        g.cash -= 12_000;
        g.company.food_days += 7;
        g.company.town_resupply();
        Self::push_log(g, "Bought a week's food. Get back to camp.");
        self.finish_day();
        self.screen = Screen::Play;
    }

    pub fn town_play(&mut self) {
        let Some(g) = self.game.as_mut() else { return };
        g.cash = (g.cash - 8_000).max(0);
        g.company.town_play_day();
        Self::push_log(
            g,
            "Town/playtime. Crew liked it tonight. Too much of this and they quit.",
        );
        self.finish_day();
        self.screen = Screen::Play;
    }

    pub fn town_repair(&mut self) {
        let Some(g) = self.game.as_mut() else { return };
        if !g.engine_dead {
            Self::push_log(g, "Engine is fine.");
            self.screen = Screen::Play;
            return;
        }
        if g.cash < 250_000 {
            Self::push_log(g, "Yard wants $2,500. You do not have it.");
            return;
        }
        g.cash -= 250_000;
        g.engine_dead = false;
        g.company.town_resupply();
        Self::push_log(g, "Engine rebuilt after the whale. Leave town.");
        self.finish_day();
        self.screen = Screen::Play;
    }

    pub fn move_camp(&mut self) {
        let Some(g) = self.game.as_ref() else { return };
        let gear = g.gear;
        let (lon, lat) = tile_to_latlon(&self.map, g.x, g.y);
        let near = nearest_site(lon, lat);
        let after = g.date >= NaiveDate::from_ymd_opt(2025, 9, 5).unwrap();
        if !near.camp || !gear_legal(gear, near.section, after) {
            if let Some(g) = self.game.as_mut() {
                Self::push_log(
                    g,
                    format!(
                        "Cannot camp a {} site at {}.",
                        gear.name(),
                        near.osm_name
                    ),
                );
            }
            return;
        }
        let Some(g) = self.game.as_mut() else { return };
        g.cash -= 40_000;
        g.camp_id = near.id.to_string();
        let (x, y) = latlon_to_tile(&self.map, near.lon, near.lat);
        g.x = x;
        g.y = y;
        g.company.day_at_camp(true);
        Self::push_log(
            g,
            format!(
                "Moved fish camp to {} ({}). Site picking is the season.",
                near.osm_name,
                near.section.name()
            ),
        );
        self.finish_day();
    }

    fn finish_day(&mut self) {
        let Some(g) = self.game.as_mut() else { return };
        if g.company.food_days <= 0 {
            Self::push_log(g, "No food. Morale is falling. Get groceries or they walk.");
        }
        let quits = g.company.maybe_quit(&mut self.rng);
        for q in quits {
            Self::push_log(g, q);
        }
        if !g.company.alive() {
            g.over = Some("Crew quit. You cannot fish alone this year.".into());
            self.screen = Screen::GameOver;
            return;
        }
        g.date = g.date.succ_opt().unwrap();
        if g.date > season_end() {
            g.over = Some("Season closed 15 September 2025.".into());
            self.screen = Screen::GameOver;
            return;
        }
        if self.rng.gen_bool(0.04) {
            Self::push_log(g, bulletin(g.date).to_string());
        }
    }

    pub fn status_lines(&self) -> Vec<String> {
        let Some(g) = &self.game else {
            return Vec::new();
        };
        let camp = self.camp().map(|s| s.osm_name).unwrap_or("?");
        let section = self.here_section();
        let open = section
            .map(|s| {
                if opening(g.date, s).is_open() {
                    "OPEN"
                } else {
                    "CLOSED"
                }
            })
            .unwrap_or("?");
        let after = g.date >= NaiveDate::from_ymd_opt(2025, 9, 5).unwrap();
        let legal = section
            .map(|s| {
                if gear_legal(g.gear, s, after) {
                    "legal"
                } else {
                    "not legal"
                }
            })
            .unwrap_or("?");
        vec![
            format!(
                "{}  {} {}   camp {}   here {} {} ({})",
                g.date.format("%a %d %b 2025"),
                g.gear.permit_code(),
                g.gear.name(),
                camp,
                section.map(|s| s.name()).unwrap_or("?"),
                open,
                legal
            ),
            format!(
                "cash {}  hold {} fish  landed {}  food {}d  fuel {}  morale {}  town/play {}  crew {}",
                format_money(g.cash),
                g.hold.total(),
                g.landed.total(),
                g.company.food_days,
                g.fuel,
                g.company.mean_morale(),
                g.company.town_play,
                g.company.hands.len()
            ),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_setnet_season_starts_closed_on_june_1() {
        let mut app = App::new(7);
        app.pick_gear = 0;
        app.pick_site = 0;
        app.start_game();
        let g = app.game.as_ref().unwrap();
        assert_eq!(g.gear, Gear::Setnet);
        assert_eq!(g.date, season_start());
        assert!(site_by_id(&g.camp_id).unwrap().camp);
        app.fish();
        let g = app.game.as_ref().unwrap();
        assert!(g
            .log
            .iter()
            .any(|l| l.to_ascii_lowercase().contains("closed")));
        assert_eq!(g.hold.total(), 0);
    }
}

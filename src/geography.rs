//! KMA geography: 5 AAC 18.100 / 18.200 / 18.330. OSM names only on the map.

use crate::map::{latlon_to_tile, MapData};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Gear {
    /// CFEC S04K set gillnet.
    Setnet,
    /// CFEC S01K purse seine (mobile).
    PurseSeine,
}

impl Gear {
    pub fn permit_code(self) -> &'static str {
        match self {
            Gear::Setnet => "S04K",
            Gear::PurseSeine => "S01K",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Gear::Setnet => "set gillnet",
            Gear::PurseSeine => "purse seine",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum District {
    Afognak,
    NorthwestKodiak,
    SouthwestKodiak,
    Alitak,
    EastsideKodiak,
    NortheastKodiak,
    Mainland,
}

impl District {
    pub fn name(self) -> &'static str {
        match self {
            District::Afognak => "Afognak District",
            District::NorthwestKodiak => "Northwest Kodiak District",
            District::SouthwestKodiak => "Southwest Kodiak District",
            District::Alitak => "Alitak District",
            District::EastsideKodiak => "Eastside Kodiak District",
            District::NortheastKodiak => "Northeast Kodiak District",
            District::Mainland => "Mainland District",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Section {
    Central,
    TerrorBay,
    InnerUganik,
    Spiridon,
    Zachar,
    Uyak,
    NorthCape,
    AntonLarsen,
    Sharatin,
    Kizhuyak,
    OuterKarluk,
    InnerKarluk,
    Sturgeon,
    HalibutBay,
    OuterAyakulik,
    InnerAyakulik,
    CapeAlitak,
    HumpyDeadman,
    AlitakBay,
    MoserBay,
    OlgaBay,
    SouthwestAfognak,
    KitoiComplex,
    FoulBaySha,
    Eastside,
    Northeast,
    Mainland,
    CapeIgvak,
}

impl Section {
    pub fn name(self) -> &'static str {
        match self {
            Section::Central => "Central Section",
            Section::TerrorBay => "Terror Bay Section",
            Section::InnerUganik => "Inner Uganik Bay Section",
            Section::Spiridon => "Spiridon Bay Section",
            Section::Zachar => "Zachar Bay Section",
            Section::Uyak => "Uyak Bay Section",
            Section::NorthCape => "North Cape Section",
            Section::AntonLarsen => "Anton Larsen Bay Section",
            Section::Sharatin => "Sharatin Bay Section",
            Section::Kizhuyak => "Kizhuyak Bay Section",
            Section::OuterKarluk => "Outer Karluk Section",
            Section::InnerKarluk => "Inner Karluk Section",
            Section::Sturgeon => "Sturgeon Section",
            Section::HalibutBay => "Halibut Bay Section",
            Section::OuterAyakulik => "Outer Ayakulik Section",
            Section::InnerAyakulik => "Inner Ayakulik Section",
            Section::CapeAlitak => "Cape Alitak Section",
            Section::HumpyDeadman => "Humpy-Deadman Section",
            Section::AlitakBay => "Alitak Bay Section",
            Section::MoserBay => "Moser Bay Section",
            Section::OlgaBay => "Olga Bay Section",
            Section::SouthwestAfognak => "Southwest Afognak Section",
            Section::KitoiComplex => "Kitoi / Duck / Izhut Bay",
            Section::FoulBaySha => "Foul Bay SHA",
            Section::Eastside => "Eastside Kodiak District",
            Section::Northeast => "Northeast Kodiak District",
            Section::Mainland => "Mainland District",
            Section::CapeIgvak => "Cape Igvak Section",
        }
    }

    pub fn district(self) -> District {
        match self {
            Section::Central
            | Section::TerrorBay
            | Section::InnerUganik
            | Section::Spiridon
            | Section::Zachar
            | Section::Uyak
            | Section::NorthCape
            | Section::AntonLarsen
            | Section::Sharatin
            | Section::Kizhuyak => District::NorthwestKodiak,
            Section::OuterKarluk
            | Section::InnerKarluk
            | Section::Sturgeon
            | Section::HalibutBay
            | Section::OuterAyakulik
            | Section::InnerAyakulik => District::SouthwestKodiak,
            Section::CapeAlitak
            | Section::HumpyDeadman
            | Section::AlitakBay
            | Section::MoserBay
            | Section::OlgaBay => District::Alitak,
            Section::SouthwestAfognak | Section::KitoiComplex | Section::FoulBaySha => {
                District::Afognak
            }
            Section::Eastside => District::EastsideKodiak,
            Section::Northeast => District::NortheastKodiak,
            Section::Mainland | Section::CapeIgvak => District::Mainland,
        }
    }

    /// Inner Alitak setnet-only waters until 4 September (5 AAC 18.330(d)(2)).
    pub fn inner_alitak_setnet(self) -> bool {
        matches!(
            self,
            Section::AlitakBay | Section::MoserBay | Section::OlgaBay
        )
    }
}

/// 5 AAC 18.330 gear legality for a section on `after_sept4`.
pub fn gear_legal(gear: Gear, section: Section, after_sept4: bool) -> bool {
    match gear {
        Gear::Setnet => match section {
            Section::Central => true,
            s if s.inner_alitak_setnet() => true,
            _ => false,
        },
        Gear::PurseSeine => {
            if section.inner_alitak_setnet() {
                after_sept4
            } else if section == Section::Central {
                true
            } else {
                // Beach seine is not a default permit; purse seine is mobile
                // in seine districts/sections when those waters are open.
                !matches!(section, Section::Central) || true
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Site {
    pub id: &'static str,
    pub osm_name: &'static str,
    pub lon: f64,
    pub lat: f64,
    pub section: Section,
    pub camp: bool,
    pub town: bool,
}

/// Camps and towns. Coordinates from Nominatim/OSM except Amook Pass (OSM name
/// at the pass west of Amook Island).
pub const SITES: &[Site] = &[
    Site {
        id: "uganik",
        osm_name: "Uganik Bay",
        lon: -153.5323,
        lat: 57.8348,
        section: Section::Central,
        camp: true,
        town: false,
    },
    Site {
        id: "uganik-passage",
        osm_name: "Uganik Passage",
        lon: -153.4105,
        lat: 57.8402,
        section: Section::Central,
        camp: true,
        town: false,
    },
    Site {
        id: "village-islands",
        osm_name: "Village Islands",
        lon: -153.5367,
        lat: 57.7825,
        section: Section::Central,
        camp: true,
        town: false,
    },
    Site {
        id: "terror",
        osm_name: "Terror Bay",
        lon: -153.2089,
        lat: 57.7471,
        section: Section::Central,
        camp: true,
        town: false,
    },
    Site {
        id: "zachar",
        osm_name: "Zachar Bay",
        lon: -153.7740,
        lat: 57.5566,
        section: Section::Central,
        camp: true,
        town: false,
    },
    Site {
        id: "uyak",
        osm_name: "Uyak Bay",
        lon: -153.9000,
        lat: 57.5250,
        section: Section::Central,
        camp: true,
        town: false,
    },
    Site {
        id: "amook-pass",
        osm_name: "Amook Pass",
        lon: -153.8500,
        lat: 57.4300,
        section: Section::Central,
        camp: true,
        town: false,
    },
    Site {
        id: "port-bailey",
        osm_name: "Port Bailey Seaplane Base",
        lon: -153.0402,
        lat: 57.9304,
        section: Section::Central,
        camp: true,
        town: false,
    },
    Site {
        id: "larsen-camp",
        osm_name: "Larsen Bay",
        lon: -153.9816,
        lat: 57.5366,
        section: Section::Central,
        camp: true,
        town: true,
    },
    Site {
        id: "alitak-bay",
        osm_name: "Alitak Bay",
        lon: -154.1217,
        lat: 56.8853,
        section: Section::AlitakBay,
        camp: true,
        town: false,
    },
    Site {
        id: "moser",
        osm_name: "Moser Bay",
        lon: -154.1451,
        lat: 57.0006,
        section: Section::MoserBay,
        camp: true,
        town: false,
    },
    Site {
        id: "olga",
        osm_name: "Olga Bay",
        lon: -154.3965,
        lat: 57.1055,
        section: Section::OlgaBay,
        camp: true,
        town: false,
    },
    Site {
        id: "lazy",
        osm_name: "Lazy Bay",
        lon: -154.2461,
        lat: 56.8943,
        section: Section::AlitakBay,
        camp: true,
        town: false,
    },
    Site {
        id: "akhiok",
        osm_name: "Akhiok",
        lon: -154.1736,
        lat: 56.9456,
        section: Section::AlitakBay,
        camp: true,
        town: true,
    },
    Site {
        id: "kodiak",
        osm_name: "Kodiak",
        lon: -152.4067,
        lat: 57.7902,
        section: Section::Northeast,
        camp: false,
        town: true,
    },
    Site {
        id: "port-lions",
        osm_name: "Port Lions",
        lon: -152.8831,
        lat: 57.8676,
        section: Section::NorthCape,
        camp: false,
        town: true,
    },
    Site {
        id: "ouzinkie",
        osm_name: "Ouzinkie",
        lon: -152.5020,
        lat: 57.9235,
        section: Section::Northeast,
        camp: false,
        town: true,
    },
    Site {
        id: "old-harbor",
        osm_name: "Old Harbor",
        lon: -153.3057,
        lat: 57.2028,
        section: Section::Eastside,
        camp: false,
        town: true,
    },
    Site {
        id: "karluk",
        osm_name: "Karluk",
        lon: -154.3040,
        lat: 57.6019,
        section: Section::InnerKarluk,
        camp: true,
        town: true,
    },
];

pub fn site_by_id(id: &str) -> Option<&'static Site> {
    SITES.iter().find(|s| s.id == id)
}

pub fn camps_for(gear: Gear) -> Vec<&'static Site> {
    SITES
        .iter()
        .filter(|s| s.camp && gear_legal(gear, s.section, false))
        .collect()
}

pub fn nearest_site(lon: f64, lat: f64) -> &'static Site {
    SITES
        .iter()
        .min_by(|a, b| {
            let da = (a.lon - lon).hypot(a.lat - lat);
            let db = (b.lon - lon).hypot(b.lat - lat);
            da.partial_cmp(&db).unwrap()
        })
        .unwrap()
}

pub fn site_tile(map: &MapData, site: &Site) -> (i32, i32) {
    latlon_to_tile(map, site.lon, site.lat)
}

/// Coarse section from lon/lat using 5 AAC 18.200 latitudes/longitudes.
pub fn section_at(lon: f64, lat: f64) -> Section {
    // Mainland / Igvak west of the archipelago
    if lon <= -155.2 {
        return if lat < 57.6 {
            Section::CapeIgvak
        } else {
            Section::Mainland
        };
    }
    // Alitak south of Low Cape ~56.99
    if lat < 56.99 && lon < -153.6 {
        if lat < 56.86 {
            return Section::CapeAlitak;
        }
        if lon > -154.02 {
            return Section::HumpyDeadman;
        }
        if lat < 56.97 {
            return Section::AlitakBay;
        }
        if lat < 57.04 {
            return Section::MoserBay;
        }
        return Section::OlgaBay;
    }
    if lat < 57.16 && lon < -153.85 {
        return Section::OlgaBay;
    }
    // Southwest Kodiak
    if lat < 57.397 && lon < -154.15 {
        if lat < 57.219 {
            return Section::InnerAyakulik;
        }
        if lat < 57.29 {
            return Section::OuterAyakulik;
        }
        return Section::HalibutBay;
    }
    if lat < 57.57 && lon < -154.15 {
        if lat < 57.511 {
            return Section::Sturgeon;
        }
        return Section::InnerKarluk;
    }
    if lat < 57.663 && lon < -154.05 {
        return Section::OuterKarluk;
    }
    // Afognak / north
    if lat > 58.06 && lon > -153.0 {
        if lon > -152.55 && lat > 58.12 && lat < 58.25 {
            return Section::KitoiComplex;
        }
        if lat > 58.35 && lon > -152.8 {
            return Section::FoulBaySha;
        }
        return Section::SouthwestAfognak;
    }
    if lat > 58.03 && lon < -153.0 {
        return Section::SouthwestAfognak;
    }
    // East / northeast
    if lon > -152.7 && lat < 57.85 {
        if lat < 57.55 {
            return Section::Eastside;
        }
        return Section::Northeast;
    }
    // Northwest inner vs Central (5 AAC 18.200(b))
    if lat < 57.50 && lon < -153.55 {
        // south of Amook Island tips → Uyak Bay Section (inner)
        if lat < 57.429 {
            return Section::Uyak;
        }
        return Section::Central;
    }
    if lon < -153.77 && lat > 57.55 && lat < 57.72 && lon > -154.05 {
        // inner Zachar east of Carlsen line is Zachar Section; west is Central
        if lon > -153.83 {
            return Section::Zachar;
        }
        return Section::Central;
    }
    if lon < -153.77 && lat > 57.62 && lat < 57.75 {
        return Section::Spiridon;
    }
    if lat < 57.50 && lon > -153.21 && lon < -152.95 {
        return Section::Eastside;
    }
    if lat < 57.833 && lon > -153.21 && lon < -152.9 && lat > 57.70 {
        return Section::TerrorBay;
    }
    Section::Central
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn setnet_only_central_and_inner_alitak() {
        assert!(gear_legal(Gear::Setnet, Section::Central, false));
        assert!(gear_legal(Gear::Setnet, Section::AlitakBay, false));
        assert!(gear_legal(Gear::Setnet, Section::OlgaBay, false));
        assert!(!gear_legal(Gear::Setnet, Section::Eastside, false));
        assert!(!gear_legal(Gear::Setnet, Section::OuterAyakulik, false));
        assert!(!gear_legal(Gear::Setnet, Section::Uyak, false));
        assert!(!gear_legal(Gear::PurseSeine, Section::OlgaBay, false));
        assert!(gear_legal(Gear::PurseSeine, Section::OlgaBay, true));
        assert!(gear_legal(Gear::PurseSeine, Section::Central, false));
        assert!(gear_legal(Gear::PurseSeine, Section::Eastside, false));
    }

    #[test]
    fn camps_are_legal_for_setnet() {
        let camps = camps_for(Gear::Setnet);
        assert!(camps.iter().any(|s| s.osm_name == "Uganik Bay"));
        assert!(camps.iter().any(|s| s.osm_name == "Amook Pass"));
        assert!(camps.iter().any(|s| s.osm_name == "Olga Bay"));
        assert!(camps.iter().all(|s| gear_legal(Gear::Setnet, s.section, false)));
        assert!(!camps.iter().any(|s| s.osm_name == "Kodiak"));
    }

    #[test]
    fn uganik_is_central() {
        assert_eq!(section_at(-153.5323, 57.8348), Section::Central);
    }

    #[test]
    fn osm_names_only() {
        for s in SITES {
            assert!(!s.osm_name.is_empty());
            assert!(!s.osm_name.contains("Village of"));
        }
    }
}

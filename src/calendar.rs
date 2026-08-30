//! 2025 KMA openings from published EOs and the season summary.
//! No invented week-by-district harvest. Extra EO dates are not fabricated.

use chrono::{Datelike, NaiveDate, Weekday};

use crate::geography::Section;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenState {
    Open,
    Closed,
}

impl OpenState {
    pub fn is_open(self) -> bool {
        matches!(self, OpenState::Open)
    }
}

const SEASON_START: (i32, u32, u32) = (2025, 6, 1);
const SEASON_END: (i32, u32, u32) = (2025, 9, 15);

pub fn season_start() -> NaiveDate {
    NaiveDate::from_ymd_opt(SEASON_START.0, SEASON_START.1, SEASON_START.2).unwrap()
}

pub fn season_end() -> NaiveDate {
    NaiveDate::from_ymd_opt(SEASON_END.0, SEASON_END.1, SEASON_END.2).unwrap()
}

fn ymd(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).unwrap()
}

fn in_range(date: NaiveDate, a: NaiveDate, b: NaiveDate) -> bool {
    date >= a && date <= b
}

/// Sunday noon – Thursday 21:00 weekly pink periods described in the 2025
/// season summary (105-hour weekly periods in July; extended to late August).
/// First confirmed period: EO #11, 6–10 July 2025.
fn westside_pink_105h(date: NaiveDate) -> bool {
    let first = ymd(2025, 7, 6);
    let last_thu = ymd(2025, 8, 28);
    if date < first || date > last_thu {
        return false;
    }
    matches!(
        date.weekday(),
        Weekday::Sun | Weekday::Mon | Weekday::Tue | Weekday::Wed | Weekday::Thu
    )
}

pub fn opening(date: NaiveDate, section: Section) -> OpenState {
    if date < season_start() || date > season_end() {
        return OpenState::Closed;
    }

    match section {
        // EO #01 / season summary: closed 1 June through 5 July.
        Section::Central
        | Section::TerrorBay
        | Section::InnerUganik
        | Section::Spiridon
        | Section::Zachar
        | Section::Uyak
        | Section::NorthCape
        | Section::AntonLarsen
        | Section::Sharatin
        | Section::Kizhuyak
        | Section::SouthwestAfognak
        | Section::OuterKarluk
        | Section::InnerKarluk => {
            if in_range(date, ymd(2025, 6, 1), ymd(2025, 7, 5)) {
                OpenState::Closed
            } else if westside_pink_105h(date) {
                OpenState::Open
            } else {
                OpenState::Closed
            }
        }
        // EO #01: Ayakulik / Halibut closed 1 June–15 July unless BEG exceeded.
        // EO #11: Outer Ayakulik 105 hours 6–10 July after BEG exceeded.
        // Season summary: two periods before 15 July; late-run weak.
        Section::OuterAyakulik => {
            if in_range(date, ymd(2025, 7, 6), ymd(2025, 7, 10)) {
                OpenState::Open
            } else if westside_pink_105h(date) && date >= ymd(2025, 7, 16) {
                OpenState::Open
            } else {
                OpenState::Closed
            }
        }
        Section::HalibutBay | Section::InnerAyakulik | Section::Sturgeon => {
            if westside_pink_105h(date) && date >= ymd(2025, 7, 16) {
                OpenState::Open
            } else {
                OpenState::Closed
            }
        }
        // EO #02: 33 hours noon 9 June – 21:00 10 June.
        // Season summary: several June periods, then significant closure late
        // June and most of July (weak Frazer). Only the EO #02 dates are
        // coded for June. Late-run: liberal time (low setnet effort) — weekly
        // 105-hour periods in August through late August.
        Section::CapeAlitak
        | Section::HumpyDeadman
        | Section::AlitakBay
        | Section::MoserBay
        | Section::OlgaBay => {
            if in_range(date, ymd(2025, 6, 9), ymd(2025, 6, 10)) {
                OpenState::Open
            } else if date >= ymd(2025, 8, 3) && westside_pink_105h(date) {
                OpenState::Open
            } else {
                OpenState::Closed
            }
        }
        // EO #01: open noon 9 June until further notice.
        // EO #11: Duck / Izhut / Kitoi remain closed during 6–10 July period.
        Section::KitoiComplex => {
            if in_range(date, ymd(2025, 6, 9), ymd(2025, 7, 5)) {
                OpenState::Open
            } else if in_range(date, ymd(2025, 7, 6), ymd(2025, 7, 10)) {
                OpenState::Closed
            } else if date >= ymd(2025, 7, 11) {
                OpenState::Open
            } else {
                OpenState::Closed
            }
        }
        Section::FoulBaySha => {
            if date >= ymd(2025, 6, 9) {
                OpenState::Open
            } else {
                OpenState::Closed
            }
        }
        // EO #11 first 105-hour period; then weekly pink periods (summary).
        Section::Eastside | Section::Northeast => {
            if westside_pink_105h(date) {
                OpenState::Open
            } else {
                OpenState::Closed
            }
        }
        // EO #11: 57 hours noon 6 July – 21:00 8 July. No further mainland
        // EOs were copied into this tree.
        Section::Mainland => {
            if in_range(date, ymd(2025, 7, 6), ymd(2025, 7, 8)) {
                OpenState::Open
            } else {
                OpenState::Closed
            }
        }
        // No Igvak through 5 July (summary). Harvest occurred 6 July–1 August
        // in the Igvak management unit; seaward closed 15 July 13:00.
        Section::CapeIgvak => {
            if westside_pink_105h(date) && date <= ymd(2025, 8, 1) {
                OpenState::Open
            } else {
                OpenState::Closed
            }
        }
    }
}

pub fn bulletin(date: NaiveDate) -> &'static str {
    if in_range(date, ymd(2025, 6, 1), ymd(2025, 6, 8)) {
        "EO #01 (4-FS-K-01-25): westside (NW Kodiak, Karluk, SW Afognak) closed 1 Jun–5 Jul. Ayakulik/Halibut closed until BEG exceeded. Kitoi complex + Foul Bay SHA open noon 9 Jun until further notice. Purse-seine Chinook ≥28\" non-retention areawide."
    } else if in_range(date, ymd(2025, 6, 9), ymd(2025, 6, 10)) {
        "EO #02 (4-FS-K-02-25): Alitak 33-hour period noon 9 Jun–21:00 10 Jun (Cape Alitak, Humpy-Deadman, Alitak Bay, Moser Bay, Olga Bay). Westside remains closed through 5 Jul."
    } else if in_range(date, ymd(2025, 6, 11), ymd(2025, 7, 5)) {
        "Westside still closed (EO #01 / season summary). Alitak: after the 9–10 Jun period the season summary records a significant closure late June and most of July (weak Frazer). Kitoi/Foul Bay remain open until further notice."
    } else if in_range(date, ymd(2025, 7, 6), ymd(2025, 7, 10)) {
        "EO #11 (4-FS-K-11-25): 105-hour period noon 6 Jul–21:00 10 Jul — NW Kodiak, NE Kodiak, Eastside, Afognak (Kitoi/Duck/Izhut closed this period). Outer Ayakulik 105 hours. Mainland 57 hours 6–8 Jul."
    } else if in_range(date, ymd(2025, 7, 11), ymd(2025, 7, 31)) {
        "Season summary: 105-hour weekly fishing periods allowed in July on the westside (strong pinks). Alitak remains largely closed (weak Frazer) through most of July."
    } else if in_range(date, ymd(2025, 8, 1), ymd(2025, 8, 28)) {
        "Season summary: very strong pink run — extended fishing time allowed until late August. Alitak late-run: liberal time (low setnet effort). Karluk late sockeye below average."
    } else {
        "Late season. Inner Alitak remains setnet-only through 4 Sep (5 AAC 18.330(d)(2)); seines may join those sections after 4 Sep. Coho still moving near Kitoi."
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn westside_closed_june() {
        let d = ymd(2025, 6, 15);
        assert_eq!(opening(d, Section::Central), OpenState::Closed);
        assert_eq!(opening(d, Section::OuterKarluk), OpenState::Closed);
    }

    #[test]
    fn alitak_open_june_9() {
        assert_eq!(opening(ymd(2025, 6, 9), Section::OlgaBay), OpenState::Open);
        assert_eq!(opening(ymd(2025, 6, 10), Section::AlitakBay), OpenState::Open);
        assert_eq!(opening(ymd(2025, 6, 11), Section::OlgaBay), OpenState::Closed);
    }

    #[test]
    fn july_6_central_open() {
        assert_eq!(opening(ymd(2025, 7, 6), Section::Central), OpenState::Open);
        assert_eq!(opening(ymd(2025, 7, 10), Section::Central), OpenState::Open);
        assert_eq!(opening(ymd(2025, 7, 11), Section::Central), OpenState::Closed);
        assert_eq!(opening(ymd(2025, 7, 13), Section::Central), OpenState::Open);
    }

    #[test]
    fn mainland_only_jul_6_8() {
        assert_eq!(opening(ymd(2025, 7, 6), Section::Mainland), OpenState::Open);
        assert_eq!(opening(ymd(2025, 7, 9), Section::Mainland), OpenState::Closed);
    }

    #[test]
    fn igvak_closed_before_july_6() {
        assert_eq!(opening(ymd(2025, 7, 5), Section::CapeIgvak), OpenState::Closed);
        assert_eq!(opening(ymd(2025, 7, 6), Section::CapeIgvak), OpenState::Open);
    }
}

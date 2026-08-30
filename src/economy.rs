use crate::data::season;
use crate::fishery::Catch;

pub fn value_cents(c: Catch) -> i64 {
    let p = season().prices_preliminary_2025;
    let v = |n: u32, price: crate::data::Price| (n as f64 * price.lb * price.usd_per_lb * 100.0).round() as i64;
    v(c.chinook, p.chinook)
        + v(c.sockeye, p.sockeye)
        + v(c.coho, p.coho)
        + v(c.pink, p.pink)
        + v(c.chum, p.chum)
}

pub fn format_money(cents: i64) -> String {
    let sign = if cents < 0 { "-" } else { "" };
    let abs = cents.unsigned_abs();
    format!("{sign}${}.{:02}", abs / 100, abs % 100)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pink_price_matches_preliminary() {
        let c = Catch {
            pink: 10,
            ..Catch::default()
        };
        // 10 * 3.2 lb * $0.30 = $9.60
        assert_eq!(value_cents(c), 960);
    }
}

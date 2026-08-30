//! Crew disposition and camp stores.
//!
//! Hands are game characters on a small Kodiak fish-camp crew (skipper plus a
//! couple of relatives/hands). They are not real 2025 CFEC permit holders.
//!
//! Morale still follows the locked rules: feed them; too much town/playtime
//! and they quit; keep them at camp during closures. Sleep is new — they need
//! rest after a long opener. Exhausted crew fish worse and quit more easily.

use rand::Rng;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Skipper,
    Relative,
    Hand,
}

impl Role {
    pub fn label(self) -> &'static str {
        match self {
            Role::Skipper => "skipper",
            Role::Relative => "relative",
            Role::Hand => "hand",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Crew {
    pub name: &'static str,
    pub role: Role,
    /// 100 = rested, 0 = exhausted.
    pub sleep: i32,
    /// 0 = full, 100 = starving.
    pub hunger: i32,
    /// 100 = high drive, 0 = walking.
    pub motivation: i32,
}

impl Crew {
    fn fresh(name: &'static str, role: Role) -> Self {
        Self {
            name,
            role,
            sleep: 78,
            hunger: 18,
            motivation: 72,
        }
    }

    /// Combined morale used by the existing quit/feed loop.
    pub fn morale(&self) -> i32 {
        let mut m = self.motivation;
        if self.sleep < 25 {
            m -= 15;
        }
        if self.hunger > 70 {
            m -= 12;
        }
        m.clamp(0, 100)
    }

    pub fn sleep_word(&self) -> &'static str {
        match self.sleep {
            0..=19 => "exhausted",
            20..=39 => "tired",
            40..=69 => "worn",
            _ => "rested",
        }
    }

    pub fn hunger_word(&self) -> &'static str {
        match self.hunger {
            0..=24 => "full",
            25..=49 => "peckish",
            50..=74 => "hungry",
            _ => "starving",
        }
    }

    pub fn motivation_word(&self) -> &'static str {
        match self.morale() {
            0..=24 => "quitting",
            25..=49 => "low",
            50..=74 => "steady",
            _ => "high",
        }
    }

    /// Satiety 0–100 (full bar = fed) so hunger reads at a glance.
    pub fn fed(&self) -> i32 {
        (100 - self.hunger).clamp(0, 100)
    }
}

#[derive(Debug, Clone)]
pub struct Stores {
    pub food_days: i32,
    pub coffee_tins: i32,
    pub stove_gal: i32,
    pub mesh_coils: i32,
    pub first_aid: i32,
}

impl Stores {
    fn starter() -> Self {
        Self {
            food_days: 10,
            coffee_tins: 4,
            stove_gal: 6,
            mesh_coils: 2,
            first_aid: 1,
        }
    }

    /// Readable camp-store lines for the crew panel.
    pub fn lines(&self) -> Vec<String> {
        vec![
            format!("  Food          {:>3} days", self.food_days),
            format!("  Coffee        {:>3} tins", self.coffee_tins),
            format!("  Stove fuel    {:>3} gal", self.stove_gal),
            format!("  Spare mesh    {:>3} coils", self.mesh_coils),
            format!("  First aid     {:>3} kit", self.first_aid),
        ]
    }
}

#[derive(Debug, Clone)]
pub struct Company {
    pub hands: Vec<Crew>,
    pub stores: Stores,
    pub town_play: i32,
    pub camp_streak: i32,
}

impl Company {
    pub fn new(names: &[&'static str]) -> Self {
        Self {
            hands: names
                .iter()
                .map(|n| Crew::fresh(n, Role::Hand))
                .collect(),
            stores: Stores::starter(),
            town_play: 0,
            camp_streak: 0,
        }
    }

    /// Small Kodiak fish-camp roster: skipper plus relatives/hands.
    /// Game characters only — not real 2025 permit holders.
    pub fn starter(setnet: bool) -> Self {
        if setnet {
            Self::with_roster(&[
                ("Mara", Role::Skipper),
                ("Iosif", Role::Relative),
                ("Denny", Role::Hand),
            ])
        } else {
            Self::with_roster(&[
                ("Mara", Role::Skipper),
                ("Iosif", Role::Relative),
                ("Denny", Role::Hand),
                ("Ruth", Role::Relative),
            ])
        }
    }

    pub fn with_roster(roster: &[(&'static str, Role)]) -> Self {
        Self {
            hands: roster
                .iter()
                .map(|(n, r)| Crew::fresh(n, *r))
                .collect(),
            stores: Stores::starter(),
            town_play: 0,
            camp_streak: 0,
        }
    }

    pub fn food_days(&self) -> i32 {
        self.stores.food_days
    }

    pub fn alive(&self) -> bool {
        !self.hands.is_empty()
    }

    pub fn mean_morale(&self) -> i32 {
        self.mean(|c| c.morale())
    }

    pub fn mean_sleep(&self) -> i32 {
        self.mean(|c| c.sleep)
    }

    pub fn mean_hunger(&self) -> i32 {
        self.mean(|c| c.hunger)
    }

    fn mean(&self, f: impl Fn(&Crew) -> i32) -> i32 {
        if self.hands.is_empty() {
            return 0;
        }
        self.hands.iter().map(f).sum::<i32>() / self.hands.len() as i32
    }

    fn clamp(v: i32) -> i32 {
        v.clamp(0, 100)
    }

    fn feed_or_starve(&mut self, ate: bool) {
        if ate && self.stores.food_days > 0 {
            self.stores.food_days -= 1;
            for c in &mut self.hands {
                c.hunger = Self::clamp(c.hunger - 22);
                c.motivation = Self::clamp(c.motivation + 1);
            }
        } else {
            for c in &mut self.hands {
                c.hunger = Self::clamp(c.hunger + 28);
                c.motivation = Self::clamp(c.motivation - 12);
            }
        }
    }

    /// A day spent at fish camp (mending or sitting a closure). Restores sleep.
    pub fn day_at_camp(&mut self, ate: bool) {
        self.camp_streak += 1;
        self.town_play = (self.town_play - 1).max(0);
        self.feed_or_starve(ate);
        for c in &mut self.hands {
            c.sleep = Self::clamp(c.sleep + 25);
        }
    }

    /// A fishing day — long opener. Burns sleep hard.
    pub fn after_opener(&mut self, ate: bool) {
        self.camp_streak += 1;
        self.town_play = (self.town_play - 1).max(0);
        self.feed_or_starve(ate);
        for c in &mut self.hands {
            c.sleep = Self::clamp(c.sleep - 22);
            if c.sleep < 30 {
                c.motivation = Self::clamp(c.motivation - 4);
            }
        }
    }

    /// Resupply in town without a binge. Rests some; still town time.
    pub fn town_resupply(&mut self) {
        self.camp_streak = 0;
        self.town_play += 1;
        for c in &mut self.hands {
            c.motivation = Self::clamp(c.motivation - 2);
            c.sleep = Self::clamp(c.sleep + 10);
            c.hunger = Self::clamp(c.hunger - 18);
        }
    }

    /// Grocery run: food plus camp stores that already live on the beach.
    pub fn restock_town(&mut self) {
        self.stores.food_days += 7;
        self.stores.coffee_tins += 2;
        self.stores.stove_gal += 2;
        self.town_resupply();
    }

    /// Bars / playtime in town or village.
    pub fn town_play_day(&mut self) {
        self.camp_streak = 0;
        self.town_play += 3;
        for c in &mut self.hands {
            c.motivation = Self::clamp(c.motivation + 8);
            c.sleep = Self::clamp(c.sleep + 6);
            c.hunger = Self::clamp(c.hunger + 8);
        }
    }

    /// 0.35–1.0. Exhausted or starving hands pick less.
    pub fn fishing_factor(&self) -> f64 {
        if self.hands.is_empty() {
            return 0.35;
        }
        let mean: f64 = self
            .hands
            .iter()
            .map(|c| {
                let rest = (c.sleep as f64 / 100.0).clamp(0.30, 1.0);
                let fed = (c.fed() as f64 / 100.0).clamp(0.40, 1.0);
                let drive = (c.motivation as f64 / 100.0).clamp(0.40, 1.0);
                rest * 0.50 + fed * 0.20 + drive * 0.30
            })
            .sum::<f64>()
            / self.hands.len() as f64;
        let wrecked = self.hands.iter().any(|c| c.sleep < 20);
        let f = if wrecked { mean * 0.70 } else { mean };
        f.clamp(0.35, 1.0)
    }

    /// Crew quit if hungry too long, wrecked from no sleep, or town/playtime runs away.
    pub fn maybe_quit(&mut self, rng: &mut impl Rng) -> Vec<String> {
        let mut gone = Vec::new();
        let mut keep = Vec::new();
        for c in self.hands.drain(..) {
            let morale = c.morale();
            let hungry = morale < 20 || c.hunger >= 85;
            let wrecked = c.sleep < 18 && (morale < 50 || rng.gen_bool(0.40));
            let town_soft = self.town_play >= 8 && rng.gen_bool(0.35);
            let town_hard = self.town_play >= 12;
            if hungry || wrecked || town_hard || town_soft {
                let why = if wrecked && !hungry && !town_hard && !town_soft {
                    "exhausted"
                } else if hungry {
                    "hungry"
                } else {
                    "town/play"
                };
                gone.push(format!(
                    "{} quit the skiff ({}; morale {}, sleep {}, town/play {}). Keep them at fish camp during closures.",
                    c.name, why, morale, c.sleep, self.town_play
                ));
            } else {
                keep.push(c);
            }
        }
        self.hands = keep;
        gone
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    #[test]
    fn town_binge_quits() {
        let mut c = Company::new(&["Aja", "Pete"]);
        for _ in 0..5 {
            c.town_play_day();
        }
        let mut rng = rand::rngs::StdRng::seed_from_u64(2);
        let gone = c.maybe_quit(&mut rng);
        assert!(!gone.is_empty());
        assert!(!c.alive());
    }

    #[test]
    fn camp_during_closure_holds() {
        let mut c = Company::new(&["Aja", "Pete"]);
        for _ in 0..8 {
            c.day_at_camp(true);
        }
        let mut rng = rand::rngs::StdRng::seed_from_u64(2);
        let gone = c.maybe_quit(&mut rng);
        assert!(gone.is_empty());
        assert_eq!(c.hands.len(), 2);
        assert!(c.mean_morale() >= 70);
        assert!(c.mean_sleep() >= 90);
    }

    #[test]
    fn no_food_drops_morale() {
        let mut c = Company::new(&["Aja"]);
        c.stores.food_days = 0;
        c.day_at_camp(false);
        assert!(c.hands[0].morale() < 70);
        assert!(c.hands[0].hunger > 18);
    }

    #[test]
    fn opener_burns_sleep_camp_restores() {
        let mut c = Company::starter(true);
        let start = c.mean_sleep();
        c.after_opener(true);
        assert!(c.mean_sleep() < start);
        let after_fish = c.mean_sleep();
        c.day_at_camp(true);
        assert!(c.mean_sleep() > after_fish);
    }

    #[test]
    fn exhausted_crew_fish_worse() {
        let rested = Company::starter(true);
        let mut wrecked = Company::starter(true);
        for h in &mut wrecked.hands {
            h.sleep = 8;
            h.hunger = 80;
            h.motivation = 40;
        }
        assert!(wrecked.fishing_factor() < rested.fishing_factor());
        assert!(wrecked.fishing_factor() < 0.55);
    }

    #[test]
    fn exhausted_more_likely_to_quit() {
        let mut c = Company::starter(true);
        for h in &mut c.hands {
            h.sleep = 10;
            h.motivation = 35;
        }
        let mut rng = rand::rngs::StdRng::seed_from_u64(3);
        let gone = c.maybe_quit(&mut rng);
        assert!(!gone.is_empty());
        assert!(gone.iter().any(|g| g.contains("exhausted") || g.contains("quit")));
    }

    #[test]
    fn starter_roster_is_small_camp_crew() {
        let set = Company::starter(true);
        assert_eq!(set.hands.len(), 3);
        assert_eq!(set.hands[0].name, "Mara");
        assert_eq!(set.hands[0].role, Role::Skipper);
        assert!(set.hands.iter().any(|h| h.role == Role::Relative));
        let seine = Company::starter(false);
        assert_eq!(seine.hands.len(), 4);
        assert_eq!(seine.stores.food_days, 10);
        assert!(seine.stores.coffee_tins > 0);
    }
}

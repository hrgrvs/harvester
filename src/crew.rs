//! Morale: food; too much town/playtime and crew quit; keep them at camp.

use rand::Rng;

#[derive(Debug, Clone)]
pub struct Crew {
    pub name: &'static str,
    pub morale: i32,
}

#[derive(Debug, Clone)]
pub struct Company {
    pub hands: Vec<Crew>,
    pub food_days: i32,
    pub town_play: i32,
    pub camp_streak: i32,
}

impl Company {
    pub fn new(names: &[&'static str]) -> Self {
        Self {
            hands: names
                .iter()
                .map(|n| Crew {
                    name: n,
                    morale: 72,
                })
                .collect(),
            food_days: 10,
            town_play: 0,
            camp_streak: 0,
        }
    }

    pub fn alive(&self) -> bool {
        !self.hands.is_empty()
    }

    pub fn mean_morale(&self) -> i32 {
        if self.hands.is_empty() {
            return 0;
        }
        self.hands.iter().map(|c| c.morale).sum::<i32>() / self.hands.len() as i32
    }

    fn clamp_morale(v: i32) -> i32 {
        v.clamp(0, 100)
    }

    /// A day spent at fish camp (working, mending, or sitting a closure).
    pub fn day_at_camp(&mut self, ate: bool) {
        self.camp_streak += 1;
        self.town_play = (self.town_play - 1).max(0);
        if ate && self.food_days > 0 {
            self.food_days -= 1;
            for c in &mut self.hands {
                c.morale = Self::clamp_morale(c.morale + 1);
            }
        } else {
            for c in &mut self.hands {
                c.morale = Self::clamp_morale(c.morale - 12);
            }
        }
    }

    /// Resupply in town without a binge.
    pub fn town_resupply(&mut self) {
        self.camp_streak = 0;
        self.town_play += 1;
        for c in &mut self.hands {
            c.morale = Self::clamp_morale(c.morale - 2);
        }
    }

    /// Bars / playtime in town or village.
    pub fn town_play_day(&mut self) {
        self.camp_streak = 0;
        self.town_play += 3;
        for c in &mut self.hands {
            c.morale = Self::clamp_morale(c.morale + 8);
        }
    }

    /// Crew quit if hungry too long or town/playtime runs away with them.
    pub fn maybe_quit(&mut self, rng: &mut impl Rng) -> Vec<String> {
        let mut gone = Vec::new();
        let mut keep = Vec::new();
        for c in self.hands.drain(..) {
            let hungry = c.morale < 20;
            let town_soft = self.town_play >= 8 && rng.gen_bool(0.35);
            let town_hard = self.town_play >= 12;
            if hungry || town_hard || town_soft {
                gone.push(format!(
                    "{} quit the skiff (morale {}, town/play {}). Keep them at fish camp during closures.",
                    c.name, c.morale, self.town_play
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
    }

    #[test]
    fn no_food_drops_morale() {
        let mut c = Company::new(&["Aja"]);
        c.food_days = 0;
        c.day_at_camp(false);
        assert!(c.hands[0].morale < 70);
    }
}

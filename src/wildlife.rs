//! Otters in the net, sea lions steal fish, whale pods kill the engine.

use rand::Rng;

use crate::fishery::Catch;
use crate::geography::Gear;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WildlifeEvent {
    None,
    OttersInNet,
    SeaLionsSteal(Catch),
    WhaleKillsEngine,
}

pub fn roll(rng: &mut impl Rng, gear: Gear, hold: &mut Catch) -> WildlifeEvent {
    let otter = if gear == Gear::Setnet { 0.08 } else { 0.02 };
    let lion = 0.06;
    let whale = if gear == Gear::PurseSeine { 0.045 } else { 0.02 };
    let r: f64 = rng.gen();
    if r < otter {
        WildlifeEvent::OttersInNet
    } else if r < otter + lion {
        let steal = Catch {
            chinook: 0,
            sockeye: (hold.sockeye / 5).min(hold.sockeye),
            coho: (hold.coho / 6).min(hold.coho),
            pink: (hold.pink / 4).min(hold.pink),
            chum: (hold.chum / 5).min(hold.chum),
        };
        hold.sockeye -= steal.sockeye;
        hold.coho -= steal.coho;
        hold.pink -= steal.pink;
        hold.chum -= steal.chum;
        WildlifeEvent::SeaLionsSteal(steal)
    } else if r < otter + lion + whale {
        WildlifeEvent::WhaleKillsEngine
    } else {
        WildlifeEvent::None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    #[test]
    fn events_can_fire() {
        let mut seen_otter = false;
        let mut seen_lion = false;
        let mut seen_whale = false;
        for seed in 0..400u64 {
            let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
            let mut hold = Catch {
                pink: 80,
                sockeye: 20,
                ..Catch::default()
            };
            match roll(&mut rng, Gear::Setnet, &mut hold) {
                WildlifeEvent::OttersInNet => seen_otter = true,
                WildlifeEvent::SeaLionsSteal(s) => {
                    seen_lion = true;
                    assert!(s.total() > 0);
                }
                WildlifeEvent::WhaleKillsEngine => seen_whale = true,
                WildlifeEvent::None => {}
            }
        }
        assert!(seen_otter && seen_lion && seen_whale);
    }
}

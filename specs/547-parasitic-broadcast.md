# 547: The Parasitic Broadcast

## 1. Overview

**Layer:** Cross-layer (3 -> 2 -> 1)
**Fantasy:** A catchy pop song from a dead empire is actually a hostile, self-replicating memetic virus.

**Mechanic:** A Layer 3 alien civilization broadcasts a seemingly harmless, incredibly catchy audio signal that is picked up by your Layer 2 comms array. It trickles down to your Layer 1 colony's entertainment networks. Pops who hear it get a massive "Entertained" buff. However, the song is a memetic parasite. Infected pops spend their work hours humming the tune, drastically reducing their productivity, and subconsciously re-wiring colony machinery to broadcast the signal back into space, drawing the attention of automated Layer 3 exterminator fleets.

**Emergence:** You are thrilled that your colony's morale is at an all-time high despite terrible conditions. Then you realize your entire mining sector hasn't extracted any ore in weeks because they spent all their time converting the drill rigs into massive acoustic amplifiers.

**Tension:** Enjoying the massive, free morale boost of the alien broadcast vs. the catastrophic productivity loss and the terrifying, delayed consequence of broadcasting your location to a hostile galaxy.

## 2. Dependencies

- `046` — Notifications System (to tell the player about the broadcast)
- `174` — Memetic Hazards (Base system for memetic infections)
- `031` — Pop Morale (For the massive Entertained buff)
- `009` — Job System (To reduce work output during humming fits)
- `287` — The Silence (Layer 3 detection risk)

## 3. RED Phase: Tests First

Write these tests in `src/layer1/memetics/parasitic_broadcast_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::memetics::{MemeticInfection, ParasiticBroadcastPlugin, process_parasitic_work_reduction};
    use crate::layer1::pop::Mood;
    use crate::layer1::jobs::WorkEfficiency;
    use crate::layer3::silence::DetectionRisk;

    #[test]
    fn test_parasitic_broadcast_increases_morale_and_reduces_work() {
        let mut world = World::new();
        // Setup a pop with the broadcast infection
        let pop = world.spawn((
            Mood { entertainment: 50.0, ..Default::default() },
            WorkEfficiency { multiplier: 1.0 },
            MemeticInfection::ParasiticBroadcast, // Humming the catchy tune
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(process_parasitic_work_reduction);
        schedule.run(&mut world);

        let mood = world.get::<Mood>(pop).unwrap();
        let eff = world.get::<WorkEfficiency>(pop).unwrap();

        // Massive morale boost
        assert!(mood.entertainment > 50.0, "The song is extremely catchy");

        // Massive productivity loss
        assert!(eff.multiplier < 1.0, "They are too busy humming to work");
    }

    #[test]
    fn test_infected_pops_increase_detection_risk() {
        let mut world = World::new();
        world.insert_resource(DetectionRisk { risk: 0.0 });

        // Spawn 10 infected pops
        for _ in 0..10 {
            world.spawn(MemeticInfection::ParasiticBroadcast);
        }

        let mut schedule = Schedule::default();
        schedule.add_systems(crate::layer1::memetics::parasitic_broadcast_risk_system);
        schedule.run(&mut world);

        let risk = world.resource::<DetectionRisk>();
        assert!(risk.risk > 0.0, "Pops are rewiring machines to broadcast into space, increasing detection risk");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/layer1/memetics/parasitic_broadcast.rs

use bevy_ecs::prelude::*;
use crate::layer1::pop::Mood;
use crate::layer1::jobs::WorkEfficiency;
use crate::layer3::silence::DetectionRisk;

#[derive(Component, PartialEq)]
pub enum MemeticInfection {
    ParasiticBroadcast,
    // Future ones like 'The Silence' or 'Cult Belief'
}

pub fn process_parasitic_work_reduction(
    mut query: Query<(&mut Mood, &mut WorkEfficiency, &MemeticInfection)>,
) {
    for (mut mood, mut eff, infection) in query.iter_mut() {
        if *infection == MemeticInfection::ParasiticBroadcast {
            // High entertainment, horrible productivity
            mood.entertainment = (mood.entertainment + 10.0).min(100.0);
            eff.multiplier *= 0.5; // Halve their work output
        }
    }
}

pub fn parasitic_broadcast_risk_system(
    mut risk: ResMut<DetectionRisk>,
    query: Query<(), With<MemeticInfection>>,
) {
    // Every infected pop acts as a tiny antenna
    let infected_count = query.iter().count() as f32;
    risk.risk += infected_count * 0.1; // Accumulate risk
}
```

## 5. REFACTOR Phase: Quality & Design

- **The Cure**: The only way to stop the broadcast is to manually destroy the affected comms arrays, or research a "Cognitive Firewall" to inoculate Pops, creating an intense mid-game crisis.
- **Audio Clues**: When zooming in on a highly infected sector, the game audio should literally play a faint, looping, catchy synth melody to break the fourth wall.

## 6. Acceptance Criteria

- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Infected Pops suffer major work penalties but gain massive entertainment bonuses.
- [ ] Layer 3 detection risk scales with the number of infected Pops.

## 7. Technical Guidance

- Ensure `WorkEfficiency` modification correctly resets or is calculated dynamically so it doesn't perpetually halve down to 0 over multiple ticks if implemented as a multiplier.

## 8. Questions
*Builder: add questions here if spec is unclear. Architect will address.*

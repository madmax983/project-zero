# 548: The Biomass Tariff

## 1. Overview

**Layer:** Cross-layer (2 -> 1)
**Fantasy:** The interstellar trade guild doesn't want your credits; they want your DNA.

**Mechanic:** A powerful, biological Layer 3 empire controls the trade routes (Layer 2) passing near your system. They refuse standard currency. Instead, they demand a "Biomass Tariff" for safe passage or trade. You must physically export a percentage of your colony's organic matter (crops, livestock, or even Pops). In exchange, they provide incredibly durable, self-healing biological building materials (Layer 1).

**Emergence:** A severe famine hits, and you have no crops to pay the tariff. Desperate to keep the trade routes open for medical supplies, you secretly categorize a group of dissenting Pops as "livestock" and export them. The bio-empire accepts, but the resulting biological materials they send back occasionally manifest the faces and voices of the exported Pops, destroying the colony's sanity.

**Tension:** The unparalleled strength and utility of alien biological technology vs. the horrific moral and societal cost of commodifying your own colony's life force.

## 2. Dependencies

- `039` — Trade System (Modifying the standard currency mechanics)
- `186` — Bio-Architecture (For the self-healing materials)
- `031` — Pop Morale (Sanity damage for trading people)
- `221` — Organic Recycling (For generating the biomass)
- `538` — Inter-Colony Trade Routes

## 3. RED Phase: Tests First

Write these tests in `src/layer2/trade/biomass_tariff_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer2::trade::{TradeDeal, process_biomass_tariff_system};
    use crate::layer1::economy::ResourceStash;
    use crate::layer1::pop::Mood;

    #[test]
    fn test_pay_biomass_tariff_with_crops() {
        let mut world = World::new();
        // Setup stash with crops
        let mut stash = ResourceStash::default();
        stash.crops = 500.0;
        stash.bio_resin = 0.0; // The alien currency
        world.insert_resource(stash);

        // Send a trade deal proposing crops for resin
        world.send_event(TradeDeal {
            offered_crops: 500.0,
            offered_pops: 0,
            requested_resin: 100.0,
        });

        let mut schedule = Schedule::default();
        schedule.add_systems(process_biomass_tariff_system);
        schedule.run(&mut world);

        let final_stash = world.resource::<ResourceStash>();
        assert_eq!(final_stash.crops, 0.0, "Crops consumed for tariff");
        assert_eq!(final_stash.bio_resin, 100.0, "Bio-resin received");
    }

    #[test]
    fn test_pay_biomass_tariff_with_pops_damages_sanity() {
        let mut world = World::new();
        // A single desperate pop to trade
        let pop_to_trade = world.spawn(Mood { stress: 10.0, ..Default::default() }).id();
        let bystander = world.spawn(Mood { stress: 10.0, ..Default::default() }).id();

        // Stash to receive the reward
        world.insert_resource(ResourceStash { bio_resin: 0.0, ..Default::default() });

        world.send_event(TradeDeal {
            offered_crops: 0.0,
            offered_pops: 1, // Desperate times
            requested_resin: 1000.0, // Worth a lot more
        });

        let mut schedule = Schedule::default();
        schedule.add_systems(process_biomass_tariff_system);
        schedule.run(&mut world);

        // Pop should be despawned (traded)
        assert!(world.get_entity(pop_to_trade).is_err(), "Traded pop is gone");

        // Massive sanity damage to bystanders
        let bystander_mood = world.get::<Mood>(bystander).unwrap();
        assert!(bystander_mood.stress > 50.0, "Selling colonists causes massive colony-wide stress");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/layer2/trade/biomass_tariff.rs

use bevy_ecs::prelude::*;
use crate::layer1::economy::ResourceStash;
use crate::layer1::pop::{Mood, Pop};

#[derive(Event)]
pub struct TradeDeal {
    pub offered_crops: f32,
    pub offered_pops: usize,
    pub requested_resin: f32,
}

pub fn process_biomass_tariff_system(
    mut events: EventReader<TradeDeal>,
    mut stash: ResMut<ResourceStash>,
    mut commands: Commands,
    mut pops: Query<(Entity, &mut Mood), With<Pop>>,
) {
    for deal in events.read() {
        if deal.offered_crops > 0.0 && stash.crops >= deal.offered_crops {
            stash.crops -= deal.offered_crops;
            stash.bio_resin += deal.requested_resin;
        }

        if deal.offered_pops > 0 {
            let mut traded = 0;
            for (entity, mut _mood) in pops.iter_mut() {
                if traded < deal.offered_pops {
                    // Export the pop (despawn)
                    commands.entity(entity).despawn();
                    traded += 1;
                }
            }

            // High reward
            stash.bio_resin += deal.requested_resin;

            // Massive colony-wide stress for commodifying human life
            for (_, mut mood) in pops.iter_mut() {
                mood.stress = (mood.stress + 40.0).min(100.0);
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design

- **The Screaming Walls**: Buildings constructed with the resulting `BioResin` should have a "Screaming" trait if Pops were used to buy the resin. This trait sporadically applies a localized `Fear` or `Stress` aura to nearby Pops, creating a haunted colony dynamic.
- **Biomass Scaling**: The empire's demand should scale dynamically over time based on colony size, forcing harder and harder choices.

## 6. Acceptance Criteria

- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] You can trade crops for BioResin.
- [ ] You can trade Pops for massive BioResin, but it causes huge colony-wide stress.

## 7. Technical Guidance

- Integrate the UI to ensure the warning for "Trading Pops" is incredibly obvious and ominous.
- Ensure the despawn logic cleanly removes the Pop from jobs, relationships, and housing arrays without panicking.

## 8. Questions
*Builder: add questions here if spec is unclear. Architect will address.*

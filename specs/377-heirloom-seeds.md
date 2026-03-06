# 377: Heirloom Seeds

## Overview

"The plants you brought from Earth are the last of their kind."

**Heirloom Seeds** introduces limited stocks of "Earth Seeds" (Wheat, Potatoes) to Layer 1. Harvesting these yields food and a chance of recovering seeds. A crop disease or fire can wipe out this vital stock, forcing the colony to rely on native crops ("Grub-Root"), which are edible but carry side effects, permanently changing the colony's diet and culture.

This creates tension: Do you plant all your seeds for maximum harvest (risking total loss to a disaster) or keep a "Doomsday Vault" reserve?

## Dependencies

- `008` — Farm (for crop planting and harvesting)
- `120` — Crop Diversity (for different types of crops and seeds)

## RED Phase: Tests First

Write these tests in `src/layer1/agriculture/heirloom_seeds_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::agriculture::heirloom::{SeedStock, EarthCrop, HarvestCropEvent, process_harvest_yield_system};
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    #[test]
    fn test_harvesting_earth_crop_yields_food_and_maybe_seeds() {
        let mut world = World::new();
        world.init_resource::<Events<HarvestCropEvent>>();
        world.insert_resource(SeedStock { amount: 0 });

        let crop = world.spawn(EarthCrop {
            yield_amount: 10,
            seed_chance: 1.0, // 100% chance to return a seed
        }).id();

        world.send_event(HarvestCropEvent { crop });

        let mut schedule = Schedule::default();
        schedule.add_systems(process_harvest_yield_system);
        schedule.run(&mut world);

        // Expect to have recovered at least 1 seed
        let stock = world.resource::<SeedStock>();
        assert!(stock.amount >= 1);
    }

    #[test]
    fn test_planting_consumes_seed() {
        let mut world = World::new();
        world.insert_resource(SeedStock { amount: 5 });

        let crop = world.spawn(EarthCrop {
            yield_amount: 10,
            seed_chance: 0.5,
        }).id();

        // Simulate a planting event/system
        world.send_event(crate::layer1::agriculture::PlantCropEvent { crop });

        let mut schedule = Schedule::default();
        schedule.add_systems(crate::layer1::agriculture::heirloom::process_planting_system);
        schedule.run(&mut world);

        // The stock should be reduced by 1
        let stock = world.resource::<SeedStock>();
        assert_eq!(stock.amount, 4);
    }
}
```

## GREEN Phase: Minimal Implementation

Implement this minimal logic in `src/layer1/agriculture/heirloom.rs`:

```rust
use bevy_ecs::prelude::*;

#[derive(Resource)]
pub struct SeedStock {
    pub amount: u32,
}

#[derive(Component)]
pub struct EarthCrop {
    pub yield_amount: u32,
    pub seed_chance: f32,
}

#[derive(Event)]
pub struct HarvestCropEvent {
    pub crop: Entity,
}

#[derive(Event)]
pub struct PlantCropEvent {
    pub crop: Entity,
}

pub fn process_harvest_yield_system(
    mut events: EventReader<HarvestCropEvent>,
    mut stock: ResMut<SeedStock>,
    crops: Query<&EarthCrop>,
) {
    for event in events.read() {
        if let Ok(crop) = crops.get(event.crop) {
            // MVP: Simplified yield calculation, just return a seed if chance is 1.0
            if crop.seed_chance >= 1.0 {
                stock.amount += 1;
            }
        }
    }
}

pub fn process_planting_system(
    mut events: EventReader<PlantCropEvent>,
    mut stock: ResMut<SeedStock>,
) {
    for _event in events.read() {
        if stock.amount > 0 {
            stock.amount -= 1;
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- Integrating `rand` for the `seed_chance` calculation so the drop rate isn't deterministic but instead a probability check.
- Tie the `SeedStock` into the `ColonyResources` or a specialized inventory system to make it visible to the player and interactable (e.g., storing it in a "Seed Vault" building).
- If all seeds are lost, ensure the game gracefully forces the colony to adopt native flora, replacing the `EarthCrop` options with native counterparts.
- A "Crop Disease" event should target and destroy either planted crops or stored seeds.

## Acceptance Criteria

- [ ] Harvesting an `EarthCrop` yields food and has a chance to return a seed.
- [ ] Planting an `EarthCrop` consumes one seed from the `SeedStock`.
- [ ] The colony starts with a non-replenishing initial supply of `SeedStock`.
- [ ] Test coverage for the new module is >= 85%.
- [ ] `cargo test` and `cargo clippy -- -D warnings` pass.

## Technical Guidance

- Ensure `SeedStock` is initialized as a resource on startup.
- The `seed_chance` should be balanced carefully to make seed management a difficult choice. If it's too high, the player never runs out. If too low, it's an immediate death sentence.

## Questions

*Builder: add questions here if spec is unclear.*

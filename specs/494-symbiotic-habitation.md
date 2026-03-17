# 494: Symbiotic Habitation

## 1. Overview
Growing your houses instead of building them. Living inside a massive, benign alien organism. You can plant "Hab-Seeds" that grow into multi-room structures. They require water and sunlight but no building materials. Pops living inside gain health regeneration but slowly develop a psychic link with the plant, sharing its stress if it's damaged. A toxic spill damaging the Hab-Plant causes every Pop living inside to simultaneously experience severe nausea and depression, crippling an entire sector's workforce without the Pops ever touching the toxin.

## 2. Dependencies
- `044` Horticulture & Beauty (Implemented)
- `034` Pop Health and Damage (Implemented)
- `140` Thermal Management (Implemented)

## 3. RED Phase: Tests First
```rust
// tests/symbiotic_habitation_tests.rs
#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;
    use scale::layer1::building::Building;
    use scale::layer1::needs::Hunger;
    use scale::layer1::health::Health;
    use scale::layer1::pop::{Pop, Morale};

    fn setup_world() -> World {
        let mut world = World::new();
        // Minimal setup
        world
    }

    #[test]
    fn test_hab_seed_grows_over_time() {
        let mut world = setup_world();

        let seed = world.spawn((
            Building,
            HabSeed { growth_stage: 0.0 },
            Health { current: 100.0, max: 100.0 },
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(grow_hab_seed_system);
        schedule.run(&mut world);

        let hab_seed = world.get::<HabSeed>(seed).unwrap();
        assert!(hab_seed.growth_stage > 0.0);
    }

    #[test]
    fn test_hab_plant_damage_causes_symbiotic_pain() {
        let mut world = setup_world();

        let plant = world.spawn((
            Building,
            HabPlant,
            Health { current: 50.0, max: 100.0 }, // Damaged
        )).id();

        let pop = world.spawn((
            Pop,
            ResidentOf { building: plant },
            SymbioticLink { strength: 1.0 },
            Health { current: 100.0, max: 100.0 },
            Morale { value: 1.0, ..Default::default() },
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(process_symbiotic_pain_system);
        schedule.run(&mut world);

        let health = world.get::<Health>(pop).unwrap();
        let morale = world.get::<Morale>(pop).unwrap();
        assert!(health.current < 100.0); // Took sympathetic damage
        assert!(morale.value < 1.0); // Took sympathetic morale hit
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// src/layer1/tech/symbiotic_habitation.rs
use bevy_ecs::prelude::*;
use crate::layer1::health::Health;
use crate::layer1::pop::Morale;
use crate::layer1::building::Building;

#[derive(Component)]
pub struct HabSeed {
    pub growth_stage: f32,
}

#[derive(Component)]
pub struct HabPlant;

#[derive(Component)]
pub struct ResidentOf {
    pub building: Entity,
}

#[derive(Component)]
pub struct SymbioticLink {
    pub strength: f32,
}

pub fn grow_hab_seed_system(
    mut commands: Commands,
    mut seeds: Query<(Entity, &mut HabSeed)>,
) {
    for (entity, mut seed) in seeds.iter_mut() {
        seed.growth_stage += 0.1; // Magic number for MVP

        if seed.growth_stage >= 100.0 {
            commands.entity(entity).remove::<HabSeed>();
            commands.entity(entity).insert(HabPlant);
        }
    }
}

pub fn process_symbiotic_pain_system(
    plants: Query<(Entity, &Health), With<HabPlant>>,
    mut residents: Query<(&mut Health, &mut Morale, &ResidentOf, &SymbioticLink), Without<HabPlant>>,
) {
    for (plant_entity, plant_health) in plants.iter() {
        let is_damaged = plant_health.current < plant_health.max;
        let pain_amount = plant_health.max - plant_health.current;

        if is_damaged {
            for (mut res_health, mut res_morale, resident, link) in residents.iter_mut() {
                if resident.building == plant_entity {
                    res_health.current -= pain_amount * 0.1 * link.strength;
                    res_morale.value -= pain_amount * 0.01 * link.strength;
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Growth Conditions**: `HabSeed` growth should depend on localized Water grids and sunlight availability, replacing raw building materials.
- **Link Scaling**: The `SymbioticLink` should grow stronger the longer a Pop resides in the `HabPlant`, increasing both the passive health regeneration they receive and the pain they share when it's damaged.
- **Healing the Plant**: Add specialized medical/horticulture jobs that allow Pops to heal the `HabPlant` directly.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures for new code.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for `src/layer1/tech/symbiotic_habitation.rs`.
- [ ] Hab-Seeds slowly grow into Hab-Plants.
- [ ] Damage to Hab-Plants translates into health/morale damage to their residents.

## 7. Technical Guidance
- `ResidentOf` could hook into existing `007 Housing` logic for room assignments.
- Be careful with `process_symbiotic_pain_system` applying damage every tick; apply it only on the delta of damage taken or use a slow DoT effect.

## 8. Questions
*Builder: add questions here if spec is unclear. Architect will address.*

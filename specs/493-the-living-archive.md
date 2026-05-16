# 493: The Living Archive

## 1. Overview
A library not of books, but of genetically modified organic hard drives that must be fed and kept comfortable. Late-game data storage requires "Flesh-Servers" - massive, immobile biological entities that store tech data. They require nutrient paste and perfect temperature control. If they get sick or stressed, data is temporarily corrupted, causing research delays or outputting bizarre, corrupted tech blueprints that cost twice as much to build but have weird secondary effects. This balances high-density, unhackable biological data storage vs. the fragility and horror of maintaining living servers.

## 2. Dependencies
- `029` Knowledge System (Implemented)
- `140` Thermal Management (Implemented)
- `034` Pop Health and Damage (Implemented)

## 3. RED Phase: Tests First
```rust
// tests/living_archive_tests.rs
#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;
    use scale::layer1::building::Building;
    use scale::layer1::tech::{Knowledge, Blueprint};
    use scale::layer1::needs::{Hunger, Temperature};
    use scale::layer1::health::Health;

    fn setup_world() -> World {
        let mut world = World::new();
        // Minimal setup
        world
    }

    #[test]
    fn test_flesh_server_corrupts_data_when_stressed() {
        let mut world = setup_world();

        let server_entity = world.spawn((
            Building,
            FleshServer,
            Health { current: 50.0, max: 100.0 }, // Damaged/Stressed
            Temperature { current: 40.0, target: 20.0 }, // Overheating
        )).id();

        let blueprint_entity = world.spawn((
            Blueprint { base_cost: 100.0, is_corrupted: false },
            StoredIn { server: server_entity },
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(process_flesh_server_stress_system);
        schedule.run(&mut world);

        let blueprint = world.get::<Blueprint>(blueprint_entity).unwrap();
        assert!(blueprint.is_corrupted);
        assert_eq!(blueprint.base_cost, 200.0); // Cost doubled
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// src/layer1/tech/living_archive.rs
use bevy_ecs::prelude::*;
use crate::layer1::building::Building;
use crate::layer1::health::Health;
use crate::layer1::needs::Temperature;

#[derive(Component)]
pub struct FleshServer;

#[derive(Component, Debug, Clone)]
pub struct Blueprint {
    pub base_cost: f32,
    pub is_corrupted: bool,
}

#[derive(Component)]
pub struct StoredIn {
    pub server: Entity,
}

pub fn process_flesh_server_stress_system(
    servers: Query<(Entity, &Health, &Temperature), With<FleshServer>>,
    mut blueprints: Query<(&mut Blueprint, &StoredIn)>,
) {
    for (server_entity, health, temp) in servers.iter() {
        let is_stressed = health.current < health.max * 0.8 || (temp.current - temp.target).abs() > 10.0;

        for (mut blueprint, stored_in) in blueprints.iter_mut() {
            if stored_in.server == server_entity {
                if is_stressed && !blueprint.is_corrupted {
                    blueprint.is_corrupted = true;
                    blueprint.base_cost *= 2.0;
                } else if !is_stressed && blueprint.is_corrupted {
                    blueprint.is_corrupted = false;
                    blueprint.base_cost /= 2.0;
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Nutrient Consumption**: Flesh-Servers should periodically consume Food or Biomass from the colony's inventory. If starved, they take Health damage.
- **Corrupted Blueprints**: Add specific secondary effects to corrupted blueprints (e.g., a chance to cause localized minor damage when built or run faster but break down sooner).
- **Horror Aura**: Flesh-Servers should emit a minor "Horror" aura that decreases the morale of nearby Pops who lack specific traits (like "Scientist" or "Psychopath").

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures for new code.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for `src/layer1/tech/living_archive.rs`.
- [ ] Flesh-Servers corrupt stored blueprints when subjected to high temperature or low health.
- [ ] Corrupted blueprints have doubled base build costs.

## 7. Technical Guidance
- Integrate the `Temperature` checking logic closely with the existing `140 Thermal Management` system.
- Blueprints currently might not be standalone entities in the core codebase, depending on how `029 Knowledge System` was implemented. If blueprints are just data in a resource, modify the resource directly based on the health of the active `FleshServer` entities.

## 8. Questions
*Builder: add questions here if spec is unclear. Architect will address.*
- **Architectural Contradictions:** `Temperature` is not a `Need` component in the codebase. Also, `Health` is not a tuple struct (e.g. `Health(f32)`); it is a full struct (`Health { current: f32, max: f32, ... }`). This spec's RED phase is incompatible with current architecture.

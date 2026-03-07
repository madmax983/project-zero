# 427: Mutagenic Rain

## 1. Overview
"Mutagenic Rain" is a rare weather event where glowing, toxic rain falls across the colony. Unshielded biological Pops (those outside, not under a roof) have a high chance to gain random "Mutant" traits. These traits can be beneficial (e.g., photosynthesis, thick skin) or harmful (e.g., brittle bones, extreme hunger).

This forces players to either quarantine their workforce, losing productivity, or gamble by intentionally exposing "expendable" pops to the rain in hopes of forced positive evolution.

## 2. Dependencies
- `src/layer1/weather.rs`: Global weather system tracking current events.
- `src/layer1/pop_traits.rs`: Adding and tracking traits for Pops.
- `src/layer1/terrain.rs`: Detecting if a tile has a "Roof" (indoor vs. outdoor).

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;
    use crate::layer1::weather::{WeatherState, WeatherType};
    use crate::layer1::pop_traits::{Traits, TraitType};
    use crate::layer1::terrain::{TerrainGrid, TerrainType};
    use crate::shared::math::GridPosition;

    fn setup_world() -> World {
        let mut world = World::new();
        let mut grid = TerrainGrid::new(10, 10, 2);
        // Add a roof at (5, 5, 1) over position (5, 5, 0)
        grid.set(5, 5, 1, TerrainType::Roof);
        world.insert_resource(grid);
        world.insert_resource(WeatherState { current: WeatherType::MutagenicRain, duration: 100.0 });
        world
    }

    #[test]
    fn test_indoor_pop_not_mutated() {
        let mut world = setup_world();

        let safe_pop = world.spawn((
            GridPosition { x: 5, y: 5, z: 0 }, // Under roof
            Pop,
            Traits { active: vec![] },
        )).id();

        apply_mutagenic_rain_system(&mut world);

        let traits = world.get::<Traits>(safe_pop).unwrap();
        assert!(traits.active.is_empty(), "Pop under roof should not mutate.");
    }

    #[test]
    fn test_outdoor_pop_mutates() {
        let mut world = setup_world();

        let exposed_pop = world.spawn((
            GridPosition { x: 2, y: 2, z: 0 }, // No roof above
            Pop,
            Traits { active: vec![] },
        )).id();

        apply_mutagenic_rain_system(&mut world);

        let traits = world.get::<Traits>(exposed_pop).unwrap();
        assert!(!traits.active.is_empty(), "Pop outside during mutagenic rain should mutate.");

        // Ensure trait is a mutant trait
        let is_mutant_trait = match traits.active[0] {
            TraitType::Photosynthesis | TraitType::ThickSkin | TraitType::BrittleBones | TraitType::ExtremeHunger => true,
            _ => false,
        };
        assert!(is_mutant_trait, "Gained trait must be a mutant variant.");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy_ecs::prelude::*;
use rand::Rng;
use crate::layer1::weather::{WeatherState, WeatherType};
use crate::layer1::pop_traits::{Traits, TraitType};
use crate::layer1::terrain::{TerrainGrid, TerrainType};
use crate::shared::math::GridPosition;

#[derive(Component)]
pub struct Pop;

pub fn apply_mutagenic_rain_system(world: &mut World) {
    let weather = world.resource::<WeatherState>();

    if weather.current != WeatherType::MutagenicRain {
        return;
    }

    let grid = world.resource::<TerrainGrid>();
    let mut rng = rand::thread_rng();

    let mut query = world.query::<(Entity, &GridPosition, &mut Traits, Option<&Pop>)>();

    for (_entity, pos, mut traits, pop_opt) in query.iter_mut(world) {
        if pop_opt.is_some() {
            // Check if there is a roof tile above the Pop
            let is_under_roof = grid.get(pos.x, pos.y, pos.z + 1) == Some(&TerrainType::Roof);

            if !is_under_roof {
                // High chance to mutate (e.g., 5% per tick for testing, but in reality maybe 0.1%)
                if rng.gen_bool(0.05) {
                    let possible_traits = vec![
                        TraitType::Photosynthesis,
                        TraitType::ThickSkin,
                        TraitType::BrittleBones,
                        TraitType::ExtremeHunger,
                    ];

                    let new_trait = possible_traits[rng.gen_range(0..possible_traits.len())].clone();

                    if !traits.active.contains(&new_trait) {
                        traits.active.push(new_trait);
                    }
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Randomness Injection:** Use a deterministic RNG seeded in a resource if determinism for testing/simulation replay is required.
- **Cooldowns/Caps:** Pops shouldn't mutate infinitely; add a check `if traits.active.len() > MAX_MUTATIONS` or a `Mutated` tag component to ensure they only mutate once per storm or up to a specific limit.
- **Performance:** Iterating all Pops every tick and calling `rng` can be slightly slow; perhaps run the mutation check less frequently (e.g., once every 10 simulation seconds or using a `Timer`).

## 6. Acceptance Criteria (Testable!)
- [ ] RED phase tests compile and pass.
- [ ] Pops located under `TerrainType::Roof` tiles are completely immune to mutations during the event.
- [ ] Pops located in unroofed tiles have a chance to gain random `TraitType`s corresponding to mutations.
- [ ] Mutation chance respects determinism/Rng setup if applicable to the architecture.
- [ ] Code coverage >= 85%.
- [ ] `cargo clippy -- -D warnings` passes.

## 7. Technical Guidance
- `TerrainType::Roof` detection might need to account for multi-story buildings if the grid supports deep Z-levels. Just checking `pos.z + 1` assumes roofs are always exactly 1 level up. A Raycast straight up to the map ceiling is safer.
- Create specific `TraitType` enums for the mutant traits if they do not exist yet. Ensure the trait logic hooks into their respective systems (e.g., `ExtremeHunger` drains Needs faster).

## 8. Questions
*Builder: add questions here if spec is unclear.*

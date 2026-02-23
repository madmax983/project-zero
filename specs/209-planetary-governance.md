# 209: Planetary Governance

## Overview

Planetary Governance allows players to assign high-ranking Pops (Layer 1) as **Governors** to orbital bodies (Layer 2).
A Governor applies passive bonuses to the planet's output based on their **Traits** and **Skills**.
For example, an `Intellectual` Governor boosts Research output, while a `Corrupt` Governor siphons resources.
This bridges the gap between the detailed colony simulation and the abstract system simulation, giving old or high-level Pops a retirement role.

## Dependencies

- `094` — System View Architecture (Target entities: `OrbitalBody`)
- `003` — Pop Entity (Source entities)
- `084` — Pop Traits (Modifier source)
- `051` — Pop Skills (Modifier source)
- `054` — Colony Edicts (Optional integration for policy bonuses)

## RED Phase: Tests First

Write these tests in `src/layer2/governance_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer2::governance::{Governor, PlanetStats, update_governance_system, assign_governor};
    use crate::layer1::pop::Pop;
    use crate::layer1::traits::{Trait, Traits};
    use crate::layer2::system::OrbitalBody;
    use std::collections::HashSet;

    fn setup_app() -> App {
        let mut app = App::new();
        // Setup necessary resources if any
        app
    }

    #[test]
    fn test_assign_governor() {
        let mut app = setup_app();
        let planet = app.world.spawn((OrbitalBody::default(), PlanetStats::default())).id();
        let pop = app.world.spawn(Pop).id();

        assign_governor(&mut app.world, planet, pop);

        let governor = app.world.get::<Governor>(planet).expect("Planet should have Governor component");
        assert_eq!(governor.pop_entity, pop);
    }

    #[test]
    fn test_governor_trait_bonus_industrialist() {
        let mut app = setup_app();
        app.add_systems(Update, update_governance_system);

        // Spawn Pop with Industrialist trait
        let pop = app.world.spawn((
            Pop,
            Traits(HashSet::from([Trait::Industrialist]))
        )).id();

        // Spawn Planet with base stats
        let planet = app.world.spawn((
            OrbitalBody::default(),
            PlanetStats { production_modifier: 1.0, ..Default::default() },
        )).id();

        assign_governor(&mut app.world, planet, pop);

        // Run update system
        app.update();

        let stats = app.world.get::<PlanetStats>(planet).unwrap();
        // Expect +10% production from Industrialist
        assert!(stats.production_modifier > 1.05);
    }

    #[test]
    fn test_governor_trait_penalty_corrupt() {
        let mut app = setup_app();
        app.add_systems(Update, update_governance_system);

        // Spawn Pop with Corrupt trait
        let pop = app.world.spawn((
            Pop,
            Traits(HashSet::from([Trait::Corrupt])) // Assume Corrupt trait exists or add it
        )).id();

        let planet = app.world.spawn((
            OrbitalBody::default(),
            PlanetStats { corruption: 0.0, ..Default::default() },
        )).id();

        assign_governor(&mut app.world, planet, pop);

        // Run update system multiple times
        for _ in 0..10 {
            app.update();
        }

        let stats = app.world.get::<PlanetStats>(planet).unwrap();
        // Corruption should increase over time
        assert!(stats.corruption > 0.0);
    }

    #[test]
    fn test_skill_bonus_administration() {
        let mut app = setup_app();

        // Spawn Pop with high Admin skill (assuming Skill map exists on Pop)
        // For MVP test, we can mock the skill retrieval or add a component
        // let pop = app.world.spawn((Pop, Skills::with_admin(50))).id();
        // For this test spec, we assume Skills component interaction.
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Update `Trait` Enum (`src/layer1/traits.rs`)

Add the following variants to the `Trait` enum:
- `Industrialist` (+Production)
- `Corrupt` (+Corruption, -Efficiency)
- `Charismatic` (-Unrest)
- `Bureaucrat` (+Stability)

### 2. Define Components (`src/layer2/governance.rs`)

```rust
use bevy_ecs::prelude::*;

#[derive(Component, Debug, Clone)]
pub struct Governor {
    pub pop_entity: Entity,
    pub assigned_at: f64, // Simulation time
}

#[derive(Component, Debug, Clone, Default)]
pub struct PlanetStats {
    pub production_modifier: f32,
    pub science_modifier: f32,
    pub corruption: f32,
    pub unrest_modifier: f32,
}
```

### 2. Implement Helper

```rust
pub fn assign_governor(world: &mut World, planet: Entity, pop: Entity) {
    // Remove existing governor if any (optional logic: return old to pool)
    world.entity_mut(planet).insert(Governor {
        pop_entity: pop,
        assigned_at: 0.0, // Get time from resource
    });

    // Optional: Add "Office" component to Pop to mark them as busy
}
```

### 3. Implement System

```rust
use crate::layer1::traits::{Trait, Traits};

pub fn update_governance_system(
    mut query: Query<(&Governor, &mut PlanetStats)>,
    pop_query: Query<&Traits>,
) {
    for (governor, mut stats) in query.iter_mut() {
        // Reset dynamic modifiers each tick (or accumulate, depending on design)
        // Better design: Base stats + Dynamic Modifiers.
        // For MVP, we'll just recalculate modifiers.

        stats.production_modifier = 1.0;
        stats.science_modifier = 1.0;

        if let Ok(traits) = pop_query.get(governor.pop_entity) {
            for trait_type in &traits.0 {
                match trait_type {
                    Trait::Industrialist => stats.production_modifier += 0.10,
                    Trait::Intellectual => stats.science_modifier += 0.10,
                    Trait::Corrupt => stats.corruption += 0.001, // Accumulates!
                    Trait::Charismatic => stats.unrest_modifier -= 0.05,
                    _ => {}
                }
            }
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **Event Driven**: Instead of querying Pops every tick, listen for `GovernorAssignedEvent` to calculate static modifiers (Production/Science). Only update dynamic stats (Corruption/XP) per tick.
- **UI Integration**: Add a "Governor" slot to the Planet Inspector UI (`src/layer2/ui.rs`?). Clicking it opens a list of eligible Pops.
- **Eligibility**: Pops must be Adult and not incarcerated.
- **Skill Scaling**: Add logic to scale bonuses by `Social` or `Admin` skill level, not just flat Trait bonuses.

## Acceptance Criteria

- [ ] `Governor` and `PlanetStats` components exist.
- [ ] `assign_governor` correctly sets the component.
- [ ] `update_governance_system` correctly modifies `PlanetStats` based on Pop Traits.
- [ ] Corruption accumulates over time for corrupt governors.
- [ ] Tests pass.

## Technical Guidance

- Ensure `PlanetStats` is added to `OrbitalBody` entities during system generation (`095`) or initialized lazily.
- Watch out for despawned Pops (check `world.get_entity(governor.pop_entity).is_some()` before querying).

# 1004: The Orbital Secession

## 1. Overview
A massive orbital habitat built at Layer 2 can develop its own distinct culture. If its wealth and population eclipse the planet below, and unrest is high, it can declare itself an independent nation. It will immediately embargo the planet and demand tribute. This forces players to balance the economic powerhouse of orbital structures against the danger of allowing them to become entirely self-sufficient and culturally disconnected from the homeworld.

## 2. Dependencies
- Layer 1 Morale/Unrest Mechanics (trigger conditions).
- Layer 1 Population & Wealth Tracking.
- Layer 2 Orbital Habitat mechanics.
- Faction System (spawning a new independent faction from an existing orbital asset).

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer2::orbit::{OrbitalHabitat, SecessionState};
    use crate::layer1::colony::ColonyResources;

    #[test]
    fn test_secession_triggers_on_high_wealth_and_unrest() {
        let mut app = App::new();
        app.add_systems(Update, evaluate_orbital_secession_system);

        let habitat_entity = app.world_mut().spawn((
            OrbitalHabitat { population: 5000, wealth: 100000.0 },
            Unrest { level: 90.0 }
        )).id();

        let planet_entity = app.world_mut().spawn((
            ColonyResources { population: 4000, wealth: 50000.0, ..Default::default() }
        )).id();

        app.update();

        let state = app.world().get::<SecessionState>(habitat_entity).unwrap();
        assert!(matches!(state, SecessionState::Seceded));
    }

    #[test]
    fn test_no_secession_if_homeworld_wealthier() {
        let mut app = App::new();
        app.add_systems(Update, evaluate_orbital_secession_system);

        let habitat_entity = app.world_mut().spawn((
            OrbitalHabitat { population: 5000, wealth: 10000.0 },
            Unrest { level: 90.0 }
        )).id();

        let planet_entity = app.world_mut().spawn((
            ColonyResources { population: 10000, wealth: 500000.0, ..Default::default() }
        )).id();

        app.update();

        let state = app.world().get::<SecessionState>(habitat_entity);
        assert!(state.is_none() || !matches!(state.unwrap(), SecessionState::Seceded));
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// src/layer2/orbit/secession.rs
use bevy::prelude::*;
use crate::layer1::colony::ColonyResources;

#[derive(Component)]
pub struct OrbitalHabitat {
    pub population: u32,
    pub wealth: f32,
}

#[derive(Component)]
pub struct Unrest {
    pub level: f32,
}

#[derive(Component, Debug, PartialEq)]
pub enum SecessionState {
    Loyal,
    Seceding,
    Seceded,
}

pub fn evaluate_orbital_secession_system(
    mut commands: Commands,
    mut habitat_query: Query<(Entity, &OrbitalHabitat, &Unrest)>,
    planet_query: Query<&ColonyResources>,
) {
    let planet = planet_query.iter().next().unwrap();

    for (entity, habitat, unrest) in habitat_query.iter_mut() {
        if habitat.wealth > planet.wealth && habitat.population > planet.population && unrest.level > 80.0 {
            commands.entity(entity).insert(SecessionState::Seceded);
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Currently evaluating against the first `ColonyResources` found. This assumes a single planet. If multi-planet systems are implemented, habitats must map explicitly to their parent planet via an `Orbiting` component.
- The threshold `80.0` for unrest should be configurable, perhaps extracted to a resource or constants file.
- The new secession faction should formally declare an embargo.

## 6. Acceptance Criteria (Testable!)
- [ ] Tests verifying secession when orbital wealth/pop exceeds planetary wealth/pop with high unrest pass.
- [ ] Tests verifying no secession occurs when planetary wealth/pop is higher pass.
- [ ] Test coverage ≥85% for the new module.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.

## 7. Technical Guidance
- Integrate `evaluate_orbital_secession_system` into the Layer 2 schedule.
- When an orbital structure secedes, ensure its controller faction is updated so it is no longer player-controlled.

## 8. Questions
*Builder: add questions here if spec is unclear.*

# The Founder Effect

## 1. Overview
**Layer:** Cross-layer (1 -> 2)
**Fantasy:** Your first colonists' traits echo through generations and across worlds.
**Mechanic:** Colony ships carry pops with traits. New colonies inherit trait distributions from founders. A colony founded by aggressive pops breeds aggressive culture.

## 2. Dependencies
- Trait system and Colony Ship mechanisms

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use std::collections::HashMap;

    #[test]
    fn test_colony_inherits_founder_traits() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, found_colony_system);

        let mut ship_traits = HashMap::new();
        ship_traits.insert(TraitType::Aggressive, 10);
        ship_traits.insert(TraitType::Pacifist, 1);

        let ship_entity = app.world.spawn((
            ColonyShip {
                crew_traits: ship_traits.clone(),
            },
            FoundColonyAction { target_system: 42 },
        )).id();

        // Act
        app.update();

        // Assert
        let mut found_colony = false;
        for colony in app.world.query::<&ColonyCulture>().iter(&app.world) {
            assert_eq!(colony.dominant_trait, TraitType::Aggressive);
            found_colony = true;
        }
        assert!(found_colony);
        assert!(app.world.get_entity(ship_entity).is_none()); // Ship despawned
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Component)]
pub struct ColonyShip {
    pub crew_traits: HashMap<TraitType, u32>,
}

#[derive(Component)]
pub struct FoundColonyAction {
    pub target_system: u32,
}

#[derive(Component)]
pub struct ColonyCulture {
    pub dominant_trait: TraitType,
    pub trait_distribution: HashMap<TraitType, u32>,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum TraitType {
    Aggressive,
    Pacifist,
    Industrious,
    Lazy,
}

pub fn found_colony_system(
    mut commands: Commands,
    query: Query<(Entity, &ColonyShip, &FoundColonyAction)>,
) {
    for (entity, ship, _) in query.iter() {
        let dominant_trait = ship.crew_traits
            .iter()
            .max_by_key(|&(_, count)| count)
            .map(|(t, _)| t.clone())
            .unwrap_or(TraitType::Industrious); // Fallback

        // Spawn the new colony with inherited culture
        commands.spawn((
            ColonyCulture {
                dominant_trait,
                trait_distribution: ship.crew_traits.clone(),
            },
        ));

        commands.entity(entity).despawn();
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Calculate the `dominant_trait` dynamically rather than passing it down entirely, allowing the colony to shift over time.
- Consider what happens to non-dominant traits—do they spawn minorities/factions within the new colony?
- The culture should actually affect new pop generation in the colony (e.g., higher chance to spawn Pops with the `dominant_trait`).

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Founding a colony transfers traits from the ship crew to the colony's culture.
- [ ] The most common trait on the ship becomes the dominant cultural trait of the new colony.

## 7. Technical Guidance
- Integrate with the existing `Trait` components.
- The `ColonyCulture` component should probably reside on the `Colony` root entity (Layer 2).

## 8. Questions
*Builder: add questions here if spec is unclear.*

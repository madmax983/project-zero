# Specification: Edible Architecture

## 1. Overview
When starvation strikes, the colony may resort to consuming advanced or organic building materials (like bioplastics or mycelial scaffolding). This mechanic allows the player to designate non-essential buildings to be "consumed." This destroys the building but provides an immediate infusion of food, functioning as a desperate measure to survive famine at the cost of infrastructure and morale.

## 2. Dependencies
- Layer 1 Economy (`ColonyResources`, Food)
- Layer 1 Buildings (`Building`, `BuildingType`, `Structure`)
- Layer 1 Pop Needs (`Needs.hunger`, `PopEvaluationQuery`)
- Layer 1 Utility AI (`ActionType::SatisfyHunger`)
- Layer 1 Map (`GridPosition`, `BuildingMap`, `OccupiedTiles`)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::architecture::building::{Building, BuildingType};
    use crate::layer1::economy::resources::ColonyResources;

    // Mock components
    #[derive(Component)]
    pub struct EdibleMaterial {
        pub food_yield: f32,
    }

    #[derive(Component)]
    pub struct Consumed;

    fn consume_building_system(
        mut commands: Commands,
        q_edible_buildings: Query<(Entity, &EdibleMaterial), With<Consumed>>,
        mut resources: ResMut<ColonyResources>,
    ) {
        // Implementation will go here
    }

    #[test]
    fn test_consuming_building_yields_food_and_destroys_building() {
        let mut app = App::new();
        app.insert_resource(ColonyResources { food: 10.0, ..Default::default() });
        app.add_systems(Update, consume_building_system);

        let building = app.world_mut().spawn((
            Building { building_type: BuildingType::Housing },
            EdibleMaterial { food_yield: 50.0 },
            Consumed, // Mark it for consumption
        )).id();

        app.update();

        // Food should have increased
        assert_eq!(app.world().resource::<ColonyResources>().food, 60.0);
        // Building should be despawned
        assert!(app.world().get_entity(building).is_err());
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
fn consume_building_system(
    mut commands: Commands,
    q_edible_buildings: Query<(Entity, &EdibleMaterial), With<Consumed>>,
    mut resources: ResMut<ColonyResources>,
) {
    for (entity, edible) in q_edible_buildings.iter() {
        resources.food += edible.food_yield;
        commands.entity(entity).despawn_recursive();
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Consumption Designation:** Introduce a `ConsumeTarget` marker (or interaction) so players can click and mark specific buildings.
- **Morale Penalty:** Destroying infrastructure to eat should trigger a `MoraleShock` or `AmenitiesLost` event, adding a temporary debuff to Pops' moods.
- **Toxicity Risk:** Not all building materials are perfectly safe. The conversion could randomly generate a `Toxicity` trait or trigger a disease outbreak event among the Pops.
- **Building Subsystems:** The building destruction logic needs to tie into the existing building teardown logic to correctly free up `BuildingMap` and `OccupiedTiles`.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >=85% for new code
- [ ] Designating an edible building for consumption increases ColonyResources `food`.
- [ ] The consumed building entity is removed from the ECS and the world grid.
- [ ] A morale penalty is properly recorded and applied to the Pops.

## 7. Technical Guidance
- Certain `BuildingType`s (e.g., those using `Mycelial Scaffolding` or `Bioplastics`) should be assigned the `EdibleMaterial` component during their construction.
- Tie the UI "Consume" action to inserting the `Consumed` component onto the target entity.
- Make sure to fire a `BuildingRemovedEvent` when despawning the building to ensure the map grid logic correctly updates.

## 8. Questions
*Builder: add questions here if spec is unclear.*

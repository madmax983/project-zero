# 318: The Stolen Fleet

## 1. Overview

A massive, advanced fleet from a neighboring Layer 3 empire mutinies and defects to the player's system (Layer 2). They pledge loyalty to the colony but demand an astronomical amount of Food and Fuel to maintain their ships. Their former empire immediately declares a "Punitive War." The player faces a choice: accept the fleet and face crippling logistical demands and an impending war, or reject them.

## 2. Dependencies

- `157` Ship Classes & Construction (for fleet mechanics)
- `159` Fleet Combat Resolution (for the impending war)
- `039` Trade System (to buy food/fuel)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_fleet_defection_event() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, process_defection_events);

        let defection = app.world_mut().spawn(DefectionEvent {
            fleet_size: 50,
            food_upkeep: 5000.0,
            fuel_upkeep: 2000.0,
            accepted: false,
            processed: false,
        }).id();

        // Act - Accept the fleet
        app.world_mut().get_mut::<DefectionEvent>(defection).unwrap().accepted = true;
        app.update();

        // Assert
        let defection_state = app.world().get::<DefectionEvent>(defection).unwrap();
        assert!(defection_state.processed, "Defection should be processed");

        let new_fleet = app.world_mut().query::<&Fleet>().iter(&app.world()).next();
        assert!(new_fleet.is_some(), "A new fleet should have been spawned");

        let upkeep = app.world().get::<UpkeepRequirement>(new_fleet.unwrap().entity()).unwrap();
        assert_eq!(upkeep.food, 5000.0, "Fleet upkeep should match defection demands");

        let war_declared = app.world_mut().query::<&WarDeclarationEvent>().iter(&app.world()).next();
        assert!(war_declared.is_some(), "A punitive war should have been declared");
    }

    #[test]
    fn test_fleet_upkeep_failure() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, process_fleet_upkeep);

        let fleet = app.world_mut().spawn((
            Fleet { ship_count: 50 },
            UpkeepRequirement { food: 5000.0, fuel: 2000.0 },
            Morale { current: 100.0 },
        )).id();

        app.world_mut().insert_resource(ColonyResources { food: 1000.0, fuel: 1000.0 }); // Insufficient

        // Act
        app.update();

        // Assert
        let morale = app.world().get::<Morale>(fleet).unwrap();
        assert!(morale.current < 100.0, "Fleet morale should drop due to insufficient upkeep");

        let resources = app.world().get_resource::<ColonyResources>().unwrap();
        assert_eq!(resources.food, 0.0, "Food should be drained completely");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct DefectionEvent {
    pub fleet_size: u32,
    pub food_upkeep: f32,
    pub fuel_upkeep: f32,
    pub accepted: bool,
    pub processed: bool,
}

#[derive(Component)]
pub struct Fleet {
    pub ship_count: u32,
}

#[derive(Component)]
pub struct UpkeepRequirement {
    pub food: f32,
    pub fuel: f32,
}

#[derive(Component)]
pub struct Morale {
    pub current: f32,
}

#[derive(Component)]
pub struct WarDeclarationEvent {
    pub enemy_id: u32,
}

#[derive(Resource)]
pub struct ColonyResources {
    pub food: f32,
    pub fuel: f32,
}

pub fn process_defection_events(
    mut commands: Commands,
    mut defection_query: Query<(Entity, &mut DefectionEvent)>,
) {
    for (entity, mut event) in defection_query.iter_mut() {
        if event.processed { continue; }
        if event.accepted {
            // Spawn the new fleet
            commands.spawn((
                Fleet { ship_count: event.fleet_size },
                UpkeepRequirement { food: event.food_upkeep, fuel: event.fuel_upkeep },
                Morale { current: 100.0 },
            ));

            // Declare war
            commands.spawn(WarDeclarationEvent { enemy_id: 1 }); // Hardcoded enemy ID for MVP

            event.processed = true;
        }
    }
}

pub fn process_fleet_upkeep(
    mut fleet_query: Query<(&UpkeepRequirement, &mut Morale), With<Fleet>>,
    mut resources: ResMut<ColonyResources>,
) {
    for (upkeep, mut morale) in fleet_query.iter_mut() {
        let mut satisfied = true;

        if resources.food >= upkeep.food {
            resources.food -= upkeep.food;
        } else {
            resources.food = 0.0;
            satisfied = false;
        }

        if resources.fuel >= upkeep.fuel {
            resources.fuel -= upkeep.fuel;
        } else {
            resources.fuel = 0.0;
            satisfied = false;
        }

        if !satisfied {
            morale.current -= 10.0; // Penalty for failing upkeep
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design

- **Event Despawn:** The `DefectionEvent` should be an actual `Event` in Bevy or the entity should be despawned once processed rather than checking `event.processed`.
- **Resource Deduction:** Use a standard `ColonyResources::consume` method instead of manual deduction.
- **Morale Consequences:** If fleet morale reaches 0, the fleet should disband, attack the colony, or leave.

## 6. Acceptance Criteria (Testable!)

- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Accepting a defection spawns a fleet with massive upkeep and triggers a war event.
- [ ] Failing to meet the new fleet's upkeep drains resources and lowers fleet morale.

## 7. Technical Guidance

- Place `process_defection_events` in `src/layer2/events/defection.rs` or an equivalent event management module.
- Tie the `WarDeclarationEvent` into existing Layer 3 diplomacy systems or Layer 2 combat generation.

## 8. Questions

*Builder: add questions here if spec is unclear.*

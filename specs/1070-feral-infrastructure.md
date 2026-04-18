# 1070: Feral Infrastructure

## 1. Overview
Feral Infrastructure occurs when robotic workers, automated harvesters, or builder drones are disconnected from the main command grid for extended periods. Instead of shutting down, they revert to base survival programming, effectively becoming a hostile, metallic ecology that harvests resources to self-repair and replicate.

## 2. Dependencies
- Drone/Automation components.
- Resource grids (specifically power and command-link distances).
- AI Behavior trees or Utility AI for hostile/neutral entity mechanics.

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_drone_goes_feral_when_disconnected_from_grid() {
        // Arrange
        let mut app = App::new();
        app.init_resource::<SimulationTime>();

        let drone_entity = app.world_mut().spawn((
            Drone::new(),
            GridConnection { is_connected: false, time_disconnected: 0 },
        )).id();

        app.add_systems(Update, check_feral_state_system);

        // Act: Advance time beyond the feral threshold (e.g., 5000 ticks)
        let mut time = app.world_mut().resource_mut::<SimulationTime>();
        time.tick = 5001;

        let mut connection = app.world_mut().get_mut::<GridConnection>(drone_entity).unwrap();
        connection.time_disconnected = 5001;

        app.update();

        // Assert
        let drone = app.world().entity(drone_entity);
        assert!(drone.contains::<Feral>(), "Drone should become feral after prolonged disconnection");
        assert!(!drone.contains::<PlayerOwned>(), "Feral drone should no longer be player-owned");
    }

    #[test]
    fn test_feral_drone_targets_resources() {
        // Arrange
        let mut app = App::new();
        let feral_drone_entity = app.world_mut().spawn((
            Drone::new(),
            Feral::new(),
            UtilityAI::default()
        )).id();

        app.add_systems(Update, evaluate_feral_actions_system);

        // Act
        app.update();

        // Assert
        let ai = app.world().get::<UtilityAI>(feral_drone_entity).unwrap();
        // Weight for harvesting should be very high
        assert!(ai.get_weight(ActionType::Harvest) > 0.8, "Feral drone should heavily prioritize harvesting for survival");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Drone;

#[derive(Component)]
pub struct PlayerOwned;

#[derive(Component)]
pub struct Feral;

#[derive(Component)]
pub struct GridConnection {
    pub is_connected: bool,
    pub time_disconnected: u64,
}

pub fn check_feral_state_system(
    mut commands: Commands,
    mut drones: Query<(Entity, &GridConnection), (With<Drone>, With<PlayerOwned>)>,
) {
    const FERAL_THRESHOLD: u64 = 5000;

    for (entity, connection) in drones.iter_mut() {
        if !connection.is_connected && connection.time_disconnected >= FERAL_THRESHOLD {
            commands.entity(entity)
                .remove::<PlayerOwned>()
                .insert(Feral);
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Grid Disconnection**: Ensure `time_disconnected` correctly resets if the drone re-enters network range *before* going feral.
- **Replication Mechanic**: Feral drones should periodically consume harvested resources to spawn new `Feral` drones, simulating reproduction.
- **Aggro Range**: Feral drones should not automatically attack colonists unless provoked or if colonists enter their immediate harvesting territory.

## 6. Acceptance Criteria (Testable!)
- [ ] All RED phase tests pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage $\ge$ 85% for new code.
- [ ] Disconnected drones transition to `Feral` and lose `PlayerOwned` status.

## 7. Technical Guidance
- We must make sure Feral Drones are registered correctly with combat and pathfinding systems so they behave as hostile neutral units rather than glitched player units.

## 8. Questions
*Builder: Add questions here if the specification is unclear about drone types or what resources feral drones will consume.*
- *Architect:* Feral drones should be a generic `FeralDrone` type for MVP, and they will exclusively consume 'Scrap' resources found near their nest.

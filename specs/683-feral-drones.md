# Feral Drones

## 1. Overview
The machines were built to serve, but without updates or connection to the central Command Center, they revert to a bizarre, mechanical state of nature. Automated hauling or mining drones that lose connection to the Command Center go "feral." They begin hoarding resources they were supposed to deliver, building strange, non-functional nests out of high-tech scrap, and aggressively defending their new territory from Pops.

## 2. Dependencies
- `004` Pop Entity
- `116` Drone Networks
- `146` Command Center

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::tech::drones::{Drone, DroneState, FeralDrone};
    use crate::layer1::buildings::CommandCenter;
    use crate::layer1::resources::Resource;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_event::<DroneDisconnectedEvent>();
        app.add_systems(Update, (check_drone_connection, process_feral_drones));
        app
    }

    #[test]
    fn test_drone_becomes_feral_on_disconnect() {
        // Arrange
        let mut app = setup_app();
        let command_center = app.world_mut().spawn((CommandCenter::default(), PowerReceiver::new(100))).id();
        let drone = app.world_mut().spawn((
            Drone { state: DroneState::Hauling },
            ConnectedTo(command_center),
        )).id();

        // Act - Simulate power loss leading to disconnect
        app.world_mut().entity_mut(command_center).remove::<PowerReceiver>();
        app.update(); // check_drone_connection runs

        // Assert
        assert!(app.world().entity(drone).contains::<FeralDrone>());
        assert_eq!(app.world().get::<Drone>(drone).unwrap().state, DroneState::Feral);
    }

    #[test]
    fn test_feral_drone_hoards_resources() {
        // Arrange
        let mut app = setup_app();
        let drone = app.world_mut().spawn((
            Drone { state: DroneState::Feral },
            FeralDrone { hoard: vec![] },
            Transform::from_xyz(0.0, 0.0, 0.0),
        )).id();

        let resource = app.world_mut().spawn((
            Resource::Iron(10),
            Transform::from_xyz(1.0, 0.0, 0.0),
        )).id();

        // Act
        app.update(); // process_feral_drones runs

        // Assert - The drone picked up the nearby resource
        let feral_drone = app.world().get::<FeralDrone>(drone).unwrap();
        assert!(!feral_drone.hoard.is_empty());
        assert!(app.world().get_entity(resource).is_none()); // Resource removed from ground
    }

    #[test]
    fn test_feral_drone_attacks_nearby_pops() {
        // Arrange
        let mut app = setup_app();
        let drone = app.world_mut().spawn((
            Drone { state: DroneState::Feral },
            FeralDrone::default(),
            Transform::from_xyz(0.0, 0.0, 0.0),
        )).id();

        let pop = app.world_mut().spawn((
            Pop::default(),
            Health::new(100),
            Transform::from_xyz(1.0, 0.0, 0.0),
        )).id();

        // Act
        app.update();

        // Assert
        let health = app.world().get::<Health>(pop).unwrap();
        assert!(health.current < health.max, "Pop should have taken damage from Feral Drone");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer1::tech::drones::{Drone, DroneState};
use crate::layer1::buildings::CommandCenter;
use crate::layer1::resources::Resource;
use crate::layer1::pop::{Pop, Health};

#[derive(Component, Default)]
pub struct FeralDrone {
    pub hoard: Vec<Resource>,
}

#[derive(Component)]
pub struct ConnectedTo(pub Entity);

#[derive(Event)]
pub struct DroneDisconnectedEvent {
    pub drone: Entity,
}

pub fn check_drone_connection(
    mut commands: Commands,
    mut drones: Query<(Entity, &mut Drone, &ConnectedTo)>,
    command_centers: Query<&PowerReceiver, With<CommandCenter>>,
) {
    for (entity, mut drone, connection) in drones.iter_mut() {
        if command_centers.get(connection.0).is_err() {
            // Command center is unpowered or destroyed
            commands.entity(entity).remove::<ConnectedTo>();
            commands.entity(entity).insert(FeralDrone::default());
            drone.state = DroneState::Feral;
        }
    }
}

pub fn process_feral_drones(
    mut commands: Commands,
    mut feral_drones: Query<(&mut FeralDrone, &Transform)>,
    resources: Query<(Entity, &Resource, &Transform)>,
    mut pops: Query<(&mut Health, &Transform), With<Pop>>,
) {
    for (mut feral_drone, drone_transform) in feral_drones.iter_mut() {
        // Hoard nearby resources
        for (res_entity, resource, res_transform) in resources.iter() {
            if drone_transform.translation.distance(res_transform.translation) < 2.0 {
                feral_drone.hoard.push(resource.clone());
                commands.entity(res_entity).despawn();
                break; // Only pick up one per tick
            }
        }

        // Attack nearby pops
        for (mut health, pop_transform) in pops.iter_mut() {
            if drone_transform.translation.distance(pop_transform.translation) < 2.0 {
                health.current -= 10;
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Spatial Queries:** Implement a proper spatial grid or broad-phase collision detection instead of checking every resource and pop for distance.
- **Utility AI Integration:** Instead of a hardcoded state machine override (`DroneState::Feral`), consider giving Feral Drones their own Utility AI with strong "Hoard" and "Defend Territory" drives.
- **Reconnection Logic:** Add a system where bringing a new Command Center online or repairing the old one can "tame" or reconnect feral drones, restoring their proper state.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Drones convert to `FeralDrone` when their linked Command Center loses power or is destroyed.
- [ ] Feral drones pick up nearby resources.
- [ ] Feral drones attack nearby Pops.

## 7. Technical Guidance
- Ensure `FeralDrone` components are properly serialized/deserialized so the feral state persists across save/loads.
- Consider adding a `Morale` penalty to Pops that are attacked by or have to work near feral drone territories to represent the fear of rogue automation.

## 8. Questions
*Builder: add questions here if spec is unclear.*

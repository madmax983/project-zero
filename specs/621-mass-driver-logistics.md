# 621: Mass Driver Logistics

## 1. Overview
Rockets are expensive; magnets are cheap. Transporting goods between planets via massive electromagnetic railguns (Mass Drivers). Packaged goods are fired at target planets. If they miss or the catcher fails, it becomes a kinetic bombardment event, devastating the destination.

## 2. Dependencies
- Layer 2 interplanetary map/distances.
- Layer 1 colony inventory and structural damage systems.
- Physics/Trajectory system (simple travel time calculation).

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_mass_driver_launches_package() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, mass_driver_launch_system);
        app.add_event::<LaunchEvent>();

        let driver = app.world_mut().spawn(MassDriver {
            ready_to_fire: true,
            target_planet: Entity::PLACEHOLDER,
            payload_amount: 1000,
        }).id();

        // Act
        app.world_mut().send_event(LaunchEvent { driver_id: driver });
        app.update();

        // Assert
        let driver_comp = app.world().get::<MassDriver>(driver).unwrap();
        assert!(!driver_comp.ready_to_fire, "Driver should not be ready immediately after firing");

        // Ensure the package exists in flight
        let packages: Vec<_> = app.world().query::<&InFlightPackage>().iter(app.world()).collect();
        assert_eq!(packages.len(), 1, "A package should be spawned in flight");
        assert_eq!(packages[0].amount, 1000);
    }

    #[test]
    fn test_package_arrival_catcher_success() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, package_arrival_system);

        let planet = app.world_mut().spawn(CatcherNetwork { success_rate: 1.0 }).id(); // 100% catch rate
        let package = app.world_mut().spawn(InFlightPackage {
            target_planet: planet,
            amount: 1000,
            eta_timer: Timer::from_seconds(0.0, TimerMode::Once), // Arrives instantly
        }).id();

        // Act
        app.update();

        // Assert
        assert!(app.world().get_entity(package).is_none(), "Package should be despawned on arrival");
        let catcher = app.world().get::<CatcherNetwork>(planet).unwrap();
        assert!(catcher.last_catch_successful, "Catch should be marked successful");
    }

    #[test]
    fn test_package_arrival_catcher_failure_causes_bombardment() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, package_arrival_system);
        app.add_event::<BombardmentEvent>();

        let planet = app.world_mut().spawn(CatcherNetwork { success_rate: 0.0 }).id(); // 0% catch rate
        let package = app.world_mut().spawn(InFlightPackage {
            target_planet: planet,
            amount: 1000,
            eta_timer: Timer::from_seconds(0.0, TimerMode::Once),
        }).id();

        // Act
        app.update();

        // Assert
        assert!(app.world().get_entity(package).is_none(), "Package should be despawned on arrival");

        let events = app.world().resource::<Events<BombardmentEvent>>();
        let mut reader = events.get_reader();
        let bomb_event = reader.read(events).next().unwrap();

        assert_eq!(bomb_event.target, planet, "Bombardment should hit the target planet");
        assert_eq!(bomb_event.kinetic_energy, 1000, "Damage should scale with payload amount");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct MassDriver {
    pub ready_to_fire: bool,
    pub target_planet: Entity,
    pub payload_amount: u32,
}

#[derive(Component)]
pub struct InFlightPackage {
    pub target_planet: Entity,
    pub amount: u32,
    pub eta_timer: Timer,
}

#[derive(Component)]
pub struct CatcherNetwork {
    pub success_rate: f32,
    pub last_catch_successful: bool,
}

#[derive(Event)]
pub struct LaunchEvent {
    pub driver_id: Entity,
}

#[derive(Event)]
pub struct BombardmentEvent {
    pub target: Entity,
    pub kinetic_energy: u32,
}

pub fn mass_driver_launch_system(
    mut commands: Commands,
    mut events: EventReader<LaunchEvent>,
    mut query: Query<&mut MassDriver>,
) {
    for event in events.read() {
        if let Ok(mut driver) = query.get_mut(event.driver_id) {
            if driver.ready_to_fire {
                driver.ready_to_fire = false;
                commands.spawn(InFlightPackage {
                    target_planet: driver.target_planet,
                    amount: driver.payload_amount,
                    eta_timer: Timer::from_seconds(5.0, TimerMode::Once), // minimal impl flight time
                });
            }
        }
    }
}

pub fn package_arrival_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut InFlightPackage)>,
    mut target_query: Query<&mut CatcherNetwork>,
    mut bomb_writer: EventWriter<BombardmentEvent>,
) {
    for (entity, mut package) in query.iter_mut() {
        package.eta_timer.tick(time.delta());
        if package.eta_timer.just_finished() {
            if let Ok(mut catcher) = target_query.get_mut(package.target_planet) {
                // Minimal impl: directly check against success rate
                if catcher.success_rate >= 1.0 {
                    catcher.last_catch_successful = true;
                    // In a real impl, add goods to target inventory here
                } else {
                    catcher.last_catch_successful = false;
                    bomb_writer.send(BombardmentEvent {
                        target: package.target_planet,
                        kinetic_energy: package.amount,
                    });
                }
            }
            commands.entity(entity).despawn();
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Catch RNG:** Replace `catcher.success_rate >= 1.0` with actual RNG resolution (`fastrand::f32() <= catcher.success_rate`).
- **Dynamic ETA:** Calculate `eta_timer` based on actual distance between planets in Layer 2.
- **Payload Safety:** Different items should have different damage profiles on impact (e.g., throwing steel blocks causes more damage than throwing grain).

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Mass drivers spawn an `InFlightPackage` that despawns upon ETA completion.
- [ ] Failure to catch results in a `BombardmentEvent` proportional to payload size.

## 7. Technical Guidance
- **Interruption:** Ensure packages in flight handle edge cases, such as the target planet being destroyed or conquered while the package is mid-transit.
- **Feedback Loop:** Send a UI notification back to the sender when a package arrives (whether safely caught or catastrophically crashed).

## 8. Questions
*Builder: Add questions here if spec is unclear.*

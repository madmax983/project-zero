# 1060: The Rocket Equation

## Overview

Space is immense, and fuel is heavy. This feature implements the "Rocket Equation" where ships have a hard "Delta-V" (Range) limit based on fuel capacity and mass. Extending range requires "Tanker" ships or "Refueling Depots". Stranded ships without Delta-V become distress beacons. This creates a strategic tension between Range (Safety) and Payload (War/Trade), where a victory deep in the void might leave a fleet stranded on their journey home.

## Dependencies

- None

## RED Phase: Tests First

```rust
// Define the tests that will drive implementation
// These should FAIL initially

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_ship_consumes_delta_v_on_movement() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, movement_consumes_delta_v_system);

        let ship = app.world_mut().spawn((
            Ship,
            DeltaV { current: 100.0, max: 100.0, consumption_rate: 10.0 },
            Moving { destination: Vec2::new(10.0, 0.0), speed: 1.0 },
        )).id();

        // Act
        app.update();

        // Assert
        let delta_v = app.world().get::<DeltaV>(ship).unwrap();
        assert_eq!(delta_v.current, 90.0);
    }

    #[test]
    fn test_ship_becomes_stranded_when_delta_v_depleted() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, (movement_consumes_delta_v_system, stranded_system));

        let ship = app.world_mut().spawn((
            Ship,
            DeltaV { current: 5.0, max: 100.0, consumption_rate: 10.0 },
            Moving { destination: Vec2::new(10.0, 0.0), speed: 1.0 },
        )).id();

        // Act
        app.update(); // Depletes to 0, adds Stranded
        app.update(); // Next tick handles stranded

        // Assert
        let stranded = app.world().get::<Stranded>(ship);
        assert!(stranded.is_some());

        let distress_beacon = app.world().get::<DistressBeacon>(ship);
        assert!(distress_beacon.is_some());
    }

    #[test]
    fn test_tanker_refuels_stranded_ship() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, refueling_system);

        let stranded_ship = app.world_mut().spawn((
            Ship,
            Stranded,
            DistressBeacon,
            DeltaV { current: 0.0, max: 100.0, consumption_rate: 10.0 },
            Position(Vec2::new(5.0, 5.0)),
        )).id();

        let tanker = app.world_mut().spawn((
            Ship,
            Tanker { fuel_payload: 50.0 },
            Position(Vec2::new(5.0, 5.0)),
            RefuelingTarget { target: stranded_ship },
        )).id();

        // Act
        app.update();

        // Assert
        let stranded_ship_delta_v = app.world().get::<DeltaV>(stranded_ship).unwrap();
        assert_eq!(stranded_ship_delta_v.current, 50.0);
        assert!(app.world().get::<Stranded>(stranded_ship).is_none());
        assert!(app.world().get::<DistressBeacon>(stranded_ship).is_none());

        let tanker_payload = app.world().get::<Tanker>(tanker).unwrap();
        assert_eq!(tanker_payload.fuel_payload, 0.0);
    }
}
```

## GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Ship;

#[derive(Component)]
pub struct DeltaV {
    pub current: f32,
    pub max: f32,
    pub consumption_rate: f32,
}

#[derive(Component)]
pub struct Moving {
    pub destination: Vec2,
    pub speed: f32,
}

#[derive(Component)]
pub struct Position(pub Vec2);

#[derive(Component)]
pub struct Stranded;

#[derive(Component)]
pub struct DistressBeacon;

#[derive(Component)]
pub struct Tanker {
    pub fuel_payload: f32,
}

#[derive(Component)]
pub struct RefuelingTarget {
    pub target: Entity,
}

pub fn movement_consumes_delta_v_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut DeltaV), With<Moving>>,
) {
    for (entity, mut delta_v) in query.iter_mut() {
        delta_v.current -= delta_v.consumption_rate;
        if delta_v.current <= 0.0 {
            delta_v.current = 0.0;
            commands.entity(entity).remove::<Moving>().insert(Stranded);
        }
    }
}

pub fn stranded_system(
    mut commands: Commands,
    query: Query<Entity, Added<Stranded>>,
) {
    for entity in query.iter() {
        commands.entity(entity).insert(DistressBeacon);
    }
}

pub fn refueling_system(
    mut commands: Commands,
    mut tankers: Query<(&mut Tanker, &RefuelingTarget, &Position)>,
    mut stranded: Query<(&mut DeltaV, &Position), With<Stranded>>,
) {
    for (mut tanker, target, tanker_pos) in tankers.iter_mut() {
        if let Ok((mut delta_v, stranded_pos)) = stranded.get_mut(target.target) {
            if tanker_pos.0.distance(stranded_pos.0) < 1.0 && tanker.fuel_payload > 0.0 {
                let fuel_needed = delta_v.max - delta_v.current;
                let fuel_transferred = fuel_needed.min(tanker.fuel_payload);

                delta_v.current += fuel_transferred;
                tanker.fuel_payload -= fuel_transferred;

                commands.entity(target.target).remove::<Stranded>().remove::<DistressBeacon>();
            }
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- Consider emitting a Bevy event like `StrandedEvent` when a ship depletes its Delta-V, allowing the lore engine to record stranded fleets and creating distinct "distress beacon" entities if necessary.
- Modify the `refueling_system` to handle partial refueling efficiently, ensuring that `Tanker` ships properly clear their `RefuelingTarget` when they run out of fuel.
- Consolidate positional distance checks to integrate with `crate::layer2::environment` coordinate standards.
- Re-use existing `Layer 2` fleet structures and tags if similar concepts already exist.

## Acceptance Criteria

- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Moving ships correctly consume Delta-V proportional to distance/time.
- [ ] Ships without sufficient Delta-V enter a `Stranded` state and act as `DistressBeacon`s.
- [ ] Tanker ships can refuel stranded targets and restore movement capability.

## Technical Guidance

- Place the new systems in `src/layer2/fleet/logistics.rs` or directly into `src/layer2/ship/mod.rs` if logistics isn't a large module.
- `DeltaV` should probably integrate closely with Layer 2's movement logic, updating only when actual progression toward a destination occurs.
- Be careful with exact float matching for distances in production implementation.

## Questions

*Builder: add questions here if spec is unclear.*

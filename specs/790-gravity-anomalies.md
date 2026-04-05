# Specification: Gravity Anomalies (Layer 1)

## 1. Overview
The planet exhibits localized, shifting "Gravity Pockets" that alter movement physics and entity behavior. Inside an anomaly, a Pop's movement speed may drastically increase or decrease. Furthermore, dropped items or specific entities might exhibit reverse or sideways gravity (e.g., sticking to the ceiling or walls). This forces players to adapt to unpredictable logistical hurdles and potentially exploit these flows for vertical hauling.

## 2. Dependencies
- `002-terrain-grid.md`: The anomaly must map to specific areas or tiles on the grid.
- `016-utility-ai-system.md` & `movement_system`: Pops must understand movement costs, and the movement system must respect gravity modifiers.
- Physics/Movement module: Ability to alter velocities or paths based on local field effects.

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_gravity_anomaly_modifies_movement_speed() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, apply_gravity_anomaly_speed_system);

        let anomaly_entity = app.world_mut().spawn((
            GravityAnomaly { multiplier: 0.5 }, // Slows down
            Transform::from_xyz(10.0, 10.0, 0.0),
            Radius(5.0),
        )).id();

        let pop_in_anomaly = app.world_mut().spawn((
            Pop,
            Transform::from_xyz(10.0, 10.0, 0.0),
            MovementSpeed(10.0),
        )).id();

        let pop_outside_anomaly = app.world_mut().spawn((
            Pop,
            Transform::from_xyz(100.0, 100.0, 0.0),
            MovementSpeed(10.0),
        )).id();

        // Act
        app.update();

        // Assert
        let speed_in = app.world().get::<MovementSpeed>(pop_in_anomaly).unwrap().0;
        let speed_out = app.world().get::<MovementSpeed>(pop_outside_anomaly).unwrap().0;

        assert_eq!(speed_in, 5.0, "Speed should be halved within the anomaly");
        assert_eq!(speed_out, 10.0, "Speed should remain unaffected outside the anomaly");
    }

    #[test]
    fn test_gravity_anomaly_reverses_item_fall_direction() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, process_item_gravity_system);

        let anomaly_entity = app.world_mut().spawn((
            GravityAnomaly { multiplier: -1.0 }, // Reverse gravity
            Transform::from_xyz(5.0, 5.0, 0.0),
            Radius(5.0),
        )).id();

        let item = app.world_mut().spawn((
            Item,
            Transform::from_xyz(5.0, 5.0, 0.0),
            Velocity(Vec3::new(0.0, -9.8, 0.0)), // Normally falling down
        )).id();

        // Act
        app.update();

        // Assert
        let velocity = app.world().get::<Velocity>(item).unwrap().0;
        assert!(velocity.y > 0.0, "Item velocity should be reversed (falling upwards) inside anomaly");
    }

    #[test]
    fn test_gravity_anomalies_shift_positions_over_time() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, shift_anomalies_system);
        app.insert_resource(Time::<Virtual>::from_seconds_f64(100.0)); // Simulate time jump

        let initial_pos = Vec3::new(10.0, 10.0, 0.0);
        let anomaly = app.world_mut().spawn((
            GravityAnomaly { multiplier: 2.0 },
            Transform::from_translation(initial_pos),
            DriftVector(Vec3::new(1.0, 0.0, 0.0)),
        )).id();

        // Act
        app.update();

        // Assert
        let current_pos = app.world().get::<Transform>(anomaly).unwrap().translation;
        assert_ne!(current_pos, initial_pos, "Anomaly should have drifted from its initial position over time");
        assert!(current_pos.x > initial_pos.x, "Anomaly should drift according to its vector");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Pop;

#[derive(Component)]
pub struct Item;

#[derive(Component)]
pub struct MovementSpeed(pub f32);

#[derive(Component)]
pub struct Velocity(pub Vec3);

#[derive(Component)]
pub struct Radius(pub f32);

#[derive(Component)]
pub struct GravityAnomaly {
    pub multiplier: f32,
}

#[derive(Component)]
pub struct DriftVector(pub Vec3);

pub fn apply_gravity_anomaly_speed_system(
    anomaly_query: Query<(&GravityAnomaly, &Transform, &Radius)>,
    mut target_query: Query<(&Transform, &mut MovementSpeed), With<Pop>>,
) {
    for (anomaly, a_transform, radius) in anomaly_query.iter() {
        for (t_transform, mut speed) in target_query.iter_mut() {
            if a_transform.translation.distance(t_transform.translation) <= radius.0 {
                // Minimal pass: modify speed directly
                // In refactor, this should apply a modifier rather than permanently mutating base speed
                speed.0 *= anomaly.multiplier;
            }
        }
    }
}

pub fn process_item_gravity_system(
    anomaly_query: Query<(&GravityAnomaly, &Transform, &Radius)>,
    mut item_query: Query<(&Transform, &mut Velocity), With<Item>>,
) {
    for (anomaly, a_transform, radius) in anomaly_query.iter() {
        for (i_transform, mut velocity) in item_query.iter_mut() {
            if a_transform.translation.distance(i_transform.translation) <= radius.0 {
                if anomaly.multiplier < 0.0 {
                    velocity.0.y = velocity.0.y.abs(); // Reverse fall direction minimally
                }
            }
        }
    }
}

pub fn shift_anomalies_system(
    time: Res<Time>,
    mut anomaly_query: Query<(&mut Transform, &DriftVector), With<GravityAnomaly>>,
) {
    for (mut transform, drift) in anomaly_query.iter_mut() {
        transform.translation += drift.0 * time.delta_secs();
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Refactoring Opportunities**: The current minimal implementation of `apply_gravity_anomaly_speed_system` permanently mutates `MovementSpeed`. It should instead calculate a `BaseSpeed` and apply an `ActiveModifier` derived from the anomaly so the pop regains normal speed upon exiting.
- **Code Smells**: Calculating distance using `Vec3::distance` for every pop against every anomaly every frame is O(N*M) and will cause performance issues.
- **Performance**: Implement a grid-based spatial partition. Anomalies should stamp their modifier onto the `TerrainGrid` (or a dedicated `PhysicsGrid`) cells they overlap. Pops then just check the modifier of the cell they are standing on (O(1)).
- **API Improvements**: Differentiate between "movement speed" anomalies and "directional" anomalies clearly in the data structures.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Pops moving through an anomaly have their speed modified.
- [ ] Items falling within a reverse-gravity anomaly fall upwards or sideways.
- [ ] Anomalies change position over time based on a drift vector.

## 7. Technical Guidance
- **Code Structure**: Create `src/layer1/physics/gravity_anomalies.rs`.
- **Integration Points**:
    - The pathfinding algorithm (`A*`) must be updated to read the `TerrainGrid`'s gravity modifiers so Pops prefer fast routes and avoid slow/dangerous ones.
    - Render anomalous zones visually in the Ratatui UI so the player knows where they are.
- **Gotchas**: Be careful with "falling upwards" reaching the edge of the simulation bounds. Ensure items eventually stick to ceilings or bounds rather than flying off into infinity.

## 8. Questions
*Builder: add questions here if spec is unclear.*

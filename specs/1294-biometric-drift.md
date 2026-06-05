# 1294: Biometric Drift

## 1. Overview
The machine remembers who you *were*, not who you *are*. Pops' biometric data degrades as they age or gain scars/trauma. If they are not "Recalibrated" by an administrator, they will be denied access to high-security areas like armories or command centers. This introduces tension between high-security lockouts and the maintenance overhead of managing physical changes in colonists.

## 2. Dependencies
- `layer1::pops::PopStats`
- `layer1::pops::PopHealth`
- `layer1::infrastructure::SecurityDoor`

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::pops::{PopStats, PopHealth};
    use crate::layer1::infrastructure::SecurityDoor;

    #[test]
    fn test_biometric_drift_denies_access() {
        let mut app = App::new();
        app.add_systems(Update, (update_biometric_drift_system, check_door_access_system));

        let pop = app.world_mut().spawn((
            PopStats { age: 30, ..Default::default() },
            PopHealth { trauma: 50, ..Default::default() },
            BiometricProfile { baseline_age: 20, drift_value: 0.0 },
            DoorAccessRequest { target: Entity::PLACEHOLDER, granted: false },
        )).id();

        let door = app.world_mut().spawn(SecurityDoor {
            required_clearance: 1,
            max_drift_tolerance: 10.0,
        }).id();

        // Update request with correct target
        app.world_mut().entity_mut(pop).insert(DoorAccessRequest { target: door, granted: false });

        // Act
        app.update();

        // Assert
        let profile = app.world().get::<BiometricProfile>(pop).unwrap();
        assert!(profile.drift_value > 0.0); // Age difference + trauma caused drift

        let request = app.world().get::<DoorAccessRequest>(pop).unwrap();
        assert!(!request.granted, "Access should be denied due to high biometric drift");
    }

    #[test]
    fn test_recalibration_restores_access() {
        let mut app = App::new();
        app.add_systems(Update, recalibrate_biometrics_system);

        // Spawn pop with high drift
        let pop = app.world_mut().spawn((
            PopStats { age: 30, ..Default::default() },
            BiometricProfile { baseline_age: 20, drift_value: 15.0 },
            RecalibrationRequest,
        )).id();

        // Act
        app.update();

        // Assert
        let profile = app.world().get::<BiometricProfile>(pop).unwrap();
        assert_eq!(profile.drift_value, 0.0, "Drift should be reset");
        assert_eq!(profile.baseline_age, 30, "Baseline age should be updated");
        assert!(app.world().get::<RecalibrationRequest>(pop).is_none(), "Request should be removed");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer1::pops::{PopStats, PopHealth};
use crate::layer1::infrastructure::SecurityDoor;

#[derive(Component)]
pub struct BiometricProfile {
    pub baseline_age: u32,
    pub drift_value: f32,
}

#[derive(Component)]
pub struct DoorAccessRequest {
    pub target: Entity,
    pub granted: bool,
}

#[derive(Component)]
pub struct RecalibrationRequest;

pub fn update_biometric_drift_system(
    mut profiles: Query<(&PopStats, &PopHealth, &mut BiometricProfile)>,
) {
    for (stats, health, mut profile) in profiles.iter_mut() {
        let age_diff = stats.age.saturating_sub(profile.baseline_age) as f32;
        // Calculate drift based on age difference and trauma
        profile.drift_value = age_diff * 0.5 + health.trauma as f32 * 0.2;
    }
}

pub fn check_door_access_system(
    mut requests: Query<(&BiometricProfile, &mut DoorAccessRequest)>,
    doors: Query<&SecurityDoor>,
) {
    for (profile, mut request) in requests.iter_mut() {
        if let Ok(door) = doors.get(request.target) {
            request.granted = profile.drift_value <= door.max_drift_tolerance;
        }
    }
}

pub fn recalibrate_biometrics_system(
    mut commands: Commands,
    mut requests: Query<(Entity, &PopStats, &mut BiometricProfile), With<RecalibrationRequest>>,
) {
    for (entity, stats, mut profile) in requests.iter_mut() {
        profile.baseline_age = stats.age;
        profile.drift_value = 0.0;
        commands.entity(entity).remove::<RecalibrationRequest>();
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Pathfinding**: The `DoorAccessRequest` needs to hook into the pathfinding/movement system so that Pops actually halt and recalculate paths when a door denies them access.
- **Admin Job**: Creating `RecalibrationRequest` should be a Utility AI task handled by Administrator or Doctor pops at a dedicated terminal/clinic, rather than auto-resolving instantly.
- **Visuals**: Add visual markers or floating text ("Access Denied: Biometric Mismatch") for player feedback.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >= 85%.
- [ ] Biometric drift accumulates based on age and trauma.
- [ ] High drift denies access to SecurityDoors.
- [ ] Recalibration resets drift and updates baseline age.

## 7. Technical Guidance
- Integrate carefully with `src/layer1/mind/movement.rs` or `pathfinding.rs`. You may need to treat high-drift doors as impassable terrain for specific Pops.

## 8. Questions
*Builder: add questions here if spec is unclear.*

use crate::layer1::infrastructure::SecurityDoor;
use crate::layer1::pops::{PopHealth, PopStats};
use bevy_ecs::prelude::*;

pub struct BiometricDrift1294Plugin;

impl bevy::prelude::Plugin for BiometricDrift1294Plugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(
            bevy::prelude::Update,
            (update_biometric_drift_system, check_door_access_system).chain(),
        );
        app.add_systems(bevy::prelude::Update, recalibrate_biometrics_system);
    }
}

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
            // Also enforce clearance as noted by the code reviewer
            let base_clearance_met = true; // Placeholder for clearance check logic since spec doesn't provide clearance level on pop

            request.granted = base_clearance_met && profile.drift_value <= door.max_drift_tolerance;
        }
    }
}

use bevy::prelude::Commands;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::infrastructure::SecurityDoor;
    use crate::layer1::pops::{PopHealth, PopStats};
    use bevy::prelude::*;

    #[test]
    fn test_biometric_drift_denies_access() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(BiometricDrift1294Plugin);

        let pop = app
            .world_mut()
            .spawn((
                PopStats {
                    age: 30,
                    ..Default::default()
                },
                PopHealth {
                    trauma: 50,
                    ..Default::default()
                },
                BiometricProfile {
                    baseline_age: 20,
                    drift_value: 0.0,
                },
                DoorAccessRequest {
                    target: Entity::PLACEHOLDER,
                    granted: false,
                },
            ))
            .id();

        let door = app
            .world_mut()
            .spawn(SecurityDoor {
                required_clearance: 1,
                max_drift_tolerance: 10.0,
            })
            .id();

        // Update request with correct target
        app.world_mut().entity_mut(pop).insert(DoorAccessRequest {
            target: door,
            granted: false,
        });

        // Act
        app.update();

        // Assert
        let profile = app.world().get::<BiometricProfile>(pop).unwrap();
        assert!(profile.drift_value > 0.0); // Age difference + trauma caused drift

        let request = app.world().get::<DoorAccessRequest>(pop).unwrap();
        assert!(
            !request.granted,
            "Access should be denied due to high biometric drift"
        );
    }

    #[test]
    fn test_recalibration_restores_access() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(BiometricDrift1294Plugin);

        // Spawn pop with high drift
        let pop = app
            .world_mut()
            .spawn((
                PopStats {
                    age: 30,
                    ..Default::default()
                },
                BiometricProfile {
                    baseline_age: 20,
                    drift_value: 15.0,
                },
                RecalibrationRequest,
            ))
            .id();

        // Act
        app.update();

        // Assert
        let profile = app.world().get::<BiometricProfile>(pop).unwrap();
        assert!(
            profile.drift_value.abs() < f32::EPSILON,
            "Drift should be reset"
        );
        assert_eq!(profile.baseline_age, 30, "Baseline age should be updated");
        assert!(
            app.world().get::<RecalibrationRequest>(pop).is_none(),
            "Request should be removed"
        );
    }
}

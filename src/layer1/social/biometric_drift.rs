use crate::layer1::biology::health::Health;
use crate::layer1::infrastructure::security_door::SecurityDoor;
use crate::layer1::lifecycle::Age;
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct BiometricProfile {
    pub baseline_age_ticks: u64,
    pub drift_value: f32,
}

#[derive(Component)]
pub struct DoorAccessRequest {
    pub target: Entity,
    pub granted: bool,
}

#[derive(Component)]
pub struct RecalibrationRequest;

pub fn update_biometric_drift_system(mut profiles: Query<(&Age, &Health, &mut BiometricProfile)>) {
    for (age, health, mut profile) in profiles.iter_mut() {
        let age_diff_ticks = age.ticks_alive.saturating_sub(profile.baseline_age_ticks);
        // Calculate drift based on age difference (in years for simplicity, assume 1 year = 100 ticks for scaling)
        let missing_health = (health.max - health.current).max(0.0);
        let years_diff = age_diff_ticks as f32 / 100.0;
        profile.drift_value = years_diff * 0.5 + missing_health * 0.2;
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
    mut requests: Query<(Entity, &Age, &mut BiometricProfile), With<RecalibrationRequest>>,
) {
    for (entity, age, mut profile) in requests.iter_mut() {
        profile.baseline_age_ticks = age.ticks_alive;
        profile.drift_value = 0.0;
        commands.entity(entity).remove::<RecalibrationRequest>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::App;
    use bevy_app::Update;

    #[test]
    fn test_biometric_drift_denies_access() {
        let mut app = App::new();
        // Since systems run in parallel, we need to ensure update runs before check
        // Or run them manually in order for testing

        let pop = app
            .world_mut()
            .spawn((
                Age {
                    ticks_alive: 3000,
                    ..Default::default()
                },
                Health {
                    current: 50.0,
                    max: 100.0,
                    has_rust_lung: false,
                }, // Missing 50 health
                BiometricProfile {
                    baseline_age_ticks: 2000,
                    drift_value: 0.0,
                },
                DoorAccessRequest {
                    target: Entity::PLACEHOLDER,
                    granted: true,
                }, // Start true to see if it flips
            ))
            .id();

        let door = app
            .world_mut()
            .spawn(SecurityDoor {
                required_clearance: 1,
                max_drift_tolerance: 5.0,
            })
            .id();

        app.world_mut().entity_mut(pop).insert(DoorAccessRequest {
            target: door,
            granted: true,
        });

        let mut schedule = bevy_ecs::schedule::Schedule::default();
        schedule.add_systems((
            update_biometric_drift_system,
            check_door_access_system.after(update_biometric_drift_system),
        ));
        schedule.run(app.world_mut());

        // Assert
        let profile = app.world().get::<BiometricProfile>(pop).unwrap();
        assert!(profile.drift_value > 0.0);

        let request = app.world().get::<DoorAccessRequest>(pop).unwrap();
        assert!(
            !request.granted,
            "Access should be denied due to high biometric drift"
        );
    }

    #[test]
    fn test_recalibration_restores_access() {
        let mut app = App::new();
        app.add_systems(Update, recalibrate_biometrics_system);

        let pop = app
            .world_mut()
            .spawn((
                Age {
                    ticks_alive: 3000,
                    ..Default::default()
                },
                BiometricProfile {
                    baseline_age_ticks: 2000,
                    drift_value: 15.0,
                },
                RecalibrationRequest,
            ))
            .id();

        app.update();

        let profile = app.world().get::<BiometricProfile>(pop).unwrap();
        assert_eq!(profile.drift_value, 0.0, "Drift should be reset");
        assert_eq!(
            profile.baseline_age_ticks, 3000,
            "Baseline age should be updated"
        );
        assert!(
            app.world().get::<RecalibrationRequest>(pop).is_none(),
            "Request should be removed"
        );
    }
}

#[cfg(test)]
mod tests {
    use crate::layer1::psychology::void_sickness::PopStats;
    use crate::layer1::security::{
        check_door_access_system, recalibrate_biometrics_system, update_biometric_drift_system,
        BiometricProfile, DoorAccessRequest, PopHealth, RecalibrationRequest, SecurityDoor,
    };
    use bevy::prelude::*;

    #[test]
    fn test_biometric_drift_denies_access() {
        let mut app = App::new();
        app.add_systems(
            Update,
            (update_biometric_drift_system, check_door_access_system).chain(),
        );

        let pop = app
            .world_mut()
            .spawn((
                PopStats {
                    age: 30,
                    ..Default::default()
                },
                PopHealth { trauma: 50 },
                BiometricProfile {
                    baseline_age: 20,
                    drift_value: 0.0,
                    last_update_tick: 0,
                    drift: 0.0,
                    recorded_scars: 0,
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
        app.add_systems(Update, recalibrate_biometrics_system);

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
                    last_update_tick: 0,
                    drift: 0.0,
                    recorded_scars: 0,
                },
                RecalibrationRequest,
            ))
            .id();

        // Act
        app.update();

        // Assert
        let profile = app.world().get::<BiometricProfile>(pop).unwrap();
        assert_eq!(profile.drift_value, 0.0, "Drift should be reset");
        assert_eq!(profile.baseline_age, 30, "Baseline age should be updated");
        assert!(
            app.world().get::<RecalibrationRequest>(pop).is_none(),
            "Request should be removed"
        );
    }
}

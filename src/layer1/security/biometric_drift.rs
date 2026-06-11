use bevy_ecs::prelude::*;
use crate::layer1::lifecycle::Age;

#[derive(Component, Default)]
pub struct PopHealth {
    pub trauma: u32,
}

#[derive(Component)]
pub struct SecurityDoor {
    pub required_clearance: u32,
    pub max_drift_tolerance: f32,
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
    mut profiles: Query<(&Age, &PopHealth, &mut BiometricProfile)>,
) {
    for (age, health, mut profile) in profiles.iter_mut() {
        let age_years = (age.ticks_alive / crate::layer1::balance::TICKS_PER_YEAR) as u32;
        let age_diff = age_years.saturating_sub(profile.baseline_age) as f32;
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
    mut requests: Query<(Entity, &Age, &mut BiometricProfile), With<RecalibrationRequest>>,
) {
    for (entity, age, mut profile) in requests.iter_mut() {
        let age_years = (age.ticks_alive / crate::layer1::balance::TICKS_PER_YEAR) as u32;
        profile.baseline_age = age_years;
        profile.drift_value = 0.0;
        commands.entity(entity).remove::<RecalibrationRequest>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::{App, Update};

    #[test]
    fn test_biometric_drift_denies_access() {
        let mut app = App::new();
        // Since bevy schedules run logic linearly, running the update_biometric_drift_system will apply drift before we check it!
        app.add_systems(Update, (update_biometric_drift_system, check_door_access_system).chain());

        let pop = app.world_mut().spawn((
            Age { ticks_alive: 30 * crate::layer1::balance::TICKS_PER_YEAR, stage: crate::layer1::lifecycle::LifeStage::Adult },
            PopHealth { trauma: 50 },
            BiometricProfile { baseline_age: 20, drift_value: 0.0 },
            DoorAccessRequest { target: Entity::PLACEHOLDER, granted: true }, // We will check if this turns to false
        )).id();

        let door = app.world_mut().spawn(SecurityDoor {
            required_clearance: 1,
            max_drift_tolerance: 10.0,
        }).id();

        // Update request with correct target
        app.world_mut().entity_mut(pop).insert(DoorAccessRequest { target: door, granted: true });

        // Act
        app.update();

        // Assert
        let profile = app.world().get::<BiometricProfile>(pop).unwrap();
        // age diff = 10 * 0.5 = 5.0. trauma = 50 * 0.2 = 10.0. drift = 15.0.
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
            Age { ticks_alive: 30 * crate::layer1::balance::TICKS_PER_YEAR, stage: crate::layer1::lifecycle::LifeStage::Adult },
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

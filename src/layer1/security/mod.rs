use bevy_ecs::prelude::*;

#[derive(Component, Debug, Clone, Default)]
pub struct BiometricProfile {
    /// The simulation tick when the profile was last calibrated.
    pub last_update_tick: u64,
    /// The current accumulated drift (0.0 to 1.0).
    pub drift: f32,
    /// The number of scars recorded at last calibration.
    pub recorded_scars: u32,
}

#[derive(Component, Debug, Clone)]
pub struct SecurityTerminal {
    /// Minimum clearance level required.
    pub required_clearance: u8,
    /// Multiplier for drift sensitivity (e.g., 1.0 = standard, 2.0 = strict).
    pub strictness: f32,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum AccessResult {
    Granted,
    Delayed(f32),
    DeniedDrift,
    DeniedClearance,
}

pub fn drift_accumulation_system(
    mut query: Query<(&mut BiometricProfile, Option<&crate::layer1::health::Scars>)>,
    time: Res<crate::shared::time::SimulationTime>,
) {
    for (mut profile, scars) in query.iter_mut() {
        // Time drift: 0.00001 per tick since last update
        let time_delta = time.tick.saturating_sub(profile.last_update_tick);
        let time_drift = (time_delta as f32) * 0.00001;

        // Trauma drift
        let current_scars = scars.map_or(0, |s| s.count);
        let scar_drift = if current_scars > profile.recorded_scars {
            (current_scars - profile.recorded_scars) as f32 * 0.1
        } else {
            0.0
        };

        // Total
        profile.drift = (time_drift + scar_drift).min(1.0);
    }
}

pub fn check_security_clearance(world: &World, pop: Entity, terminal: Entity) -> AccessResult {
    let Some(profile) = world.get::<BiometricProfile>(pop) else {
        return AccessResult::Granted; // No profile? Assume granted or irrelevant.
    };

    let Some(term) = world.get::<SecurityTerminal>(terminal) else {
        return AccessResult::Granted; // Not a terminal?
    };

    let effective_drift = profile.drift * term.strictness;

    if effective_drift > 0.8 {
        AccessResult::DeniedDrift
    } else if effective_drift > 0.5 {
        AccessResult::Delayed(effective_drift * 5.0) // e.g. 5 seconds * drift
    } else {
        AccessResult::Granted
    }
}

pub fn recalibrate_profile(world: &mut World, pop: Entity) {
    let current_tick = world.resource::<crate::shared::time::SimulationTime>().tick;
    // Get current scars to sync
    let current_scars = world
        .get::<crate::layer1::health::Scars>(pop)
        .map_or(0, |s| s.count);

    if let Some(mut profile) = world.get_mut::<BiometricProfile>(pop) {
        profile.drift = 0.0;
        profile.last_update_tick = current_tick;
        profile.recorded_scars = current_scars;
    }
}

#[cfg(test)]
mod biometric_drift_tests;

//! Security
//!
//! Defines security systems, policing, and threat management.

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
mod biometric_drift_tests {
    use super::*;
    use crate::layer1::health::Scars;
    use crate::layer1::lifecycle::Age;
    use crate::layer1::pop::Pop;
    use crate::shared::time::SimulationTime;

    #[test]
    fn test_drift_accumulation_over_time() {
        let mut world = World::new();
        world.insert_resource(SimulationTime {
            tick: 1000,
            speed: crate::shared::time::SimSpeed::Normal,
        });

        let pop = world
            .spawn((
                Pop,
                BiometricProfile {
                    last_update_tick: 0,
                    drift: 0.0,
                    recorded_scars: 0,
                },
                Age {
                    ticks_alive: 5000,
                    stage: crate::layer1::lifecycle::LifeStage::Adult,
                },
            ))
            .id();

        // Run drift system
        let mut schedule = Schedule::default();
        schedule.add_systems(drift_accumulation_system);
        schedule.run(&mut world);

        let profile = world.get::<BiometricProfile>(pop).unwrap();
        // 1000 ticks should cause some drift
        assert!(
            profile.drift > 0.0,
            "Drift should be greater than 0 after 1000 ticks, got {}",
            profile.drift
        );
    }

    #[test]
    fn test_scars_increase_drift() {
        let mut world = World::new();
        world.insert_resource(SimulationTime {
            tick: 0,
            speed: crate::shared::time::SimSpeed::Normal,
        });

        let pop = world
            .spawn((
                Pop,
                BiometricProfile {
                    drift: 0.0,
                    last_update_tick: 0,
                    recorded_scars: 0,
                },
                Scars { count: 0 },
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(drift_accumulation_system);

        // Initial run (0 drift)
        schedule.run(&mut world);
        let profile = world.get::<BiometricProfile>(pop).unwrap();
        assert_eq!(profile.drift, 0.0);

        // Simulate trauma (increase scars)
        if let Some(mut scars) = world.get_mut::<Scars>(pop) {
            scars.count = 2;
        }

        // Run system again
        schedule.run(&mut world);

        let profile = world.get::<BiometricProfile>(pop).unwrap();
        // 2 scars * 0.1 drift per scar = 0.2
        assert!(
            (profile.drift - 0.2).abs() < f32::EPSILON,
            "Drift should be 0.2 from 2 scars, got {}",
            profile.drift
        );
    }

    #[test]
    fn test_access_denied_high_drift() {
        let mut world = World::new();
        let pop = world
            .spawn((
                Pop,
                BiometricProfile {
                    drift: 0.9,
                    ..Default::default()
                }, // 90% Drift
            ))
            .id();

        let terminal = world
            .spawn(SecurityTerminal {
                required_clearance: 1,
                strictness: 1.0, // Strict
            })
            .id();

        // Check Access
        let result = check_security_clearance(&world, pop, terminal);
        assert_eq!(result, AccessResult::DeniedDrift);
    }

    #[test]
    fn test_access_slow_medium_drift() {
        let mut world = World::new();
        let pop = world
            .spawn((
                Pop,
                BiometricProfile {
                    drift: 0.6,
                    ..Default::default()
                }, // 60% Drift
            ))
            .id();

        let terminal = world
            .spawn(SecurityTerminal {
                required_clearance: 1,
                strictness: 1.0,
            })
            .id();

        let result = check_security_clearance(&world, pop, terminal);
        // Note: Delay duration is float, using assert matches pattern in AccessResult
        if let AccessResult::Delayed(duration) = result {
            assert!(duration > 0.0);
        } else {
            panic!("Expected Delayed access, got {:?}", result);
        }
    }

    #[test]
    fn test_recalibration_resets_drift() {
        let mut world = World::new();
        let pop = world
            .spawn((
                Pop,
                BiometricProfile {
                    drift: 0.9,
                    last_update_tick: 0,
                    recorded_scars: 0,
                },
            ))
            .id();

        world.insert_resource(SimulationTime {
            tick: 2000,
            speed: crate::shared::time::SimSpeed::Normal,
        });

        // Perform recalibration action
        recalibrate_profile(&mut world, pop);

        let profile = world.get::<BiometricProfile>(pop).unwrap();
        assert_eq!(profile.drift, 0.0);
        assert_eq!(profile.last_update_tick, 2000);
    }
}

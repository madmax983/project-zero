//! The Gravity Siphon (Nova Feature).
//!
//! # The Spark
//! We have a `Fleet` system in Layer 2 that travels between `OrbitalBody` entities using `InTransit` components.
//! On Layer 1, we simulate buildings and production.
//! What if a massive, endgame power generator on Layer 1 produced incredible amounts of energy,
//! but slowly warped the local gravity well, directly interfering with Layer 2 logistics?
//!
//! # The Feature
//! A `MicroSingularityGenerator` component can be attached to a building.
//! When active, it slowly increases a global `GravitationalAnomaly` resource.
//! As the anomaly grows, a cross-layer system `gravity_siphon_disruption_system`
//! randomly delays the `progress` of Layer 2 fleets (`InTransit`), simulating
//! navigational hazards caused by the distorted gravity well.

use crate::layer2::fleet::InTransit;
use bevy_ecs::prelude::*;
use rand::Rng;

/// A global resource tracking the total amount of gravitational distortion caused by micro-singularities.
#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct GravitationalAnomaly {
    /// The current level of distortion (0.0 to 1.0+).
    pub level: f32,
}

/// Component attached to a building that generates power but warps gravity.
#[derive(Component, Debug, Clone)]
pub struct MicroSingularityGenerator {
    /// Is the generator currently running?
    pub is_active: bool,
    /// How much anomaly is generated per tick while active.
    pub anomaly_rate: f32,
}

impl Default for MicroSingularityGenerator {
    fn default() -> Self {
        Self {
            is_active: false,
            anomaly_rate: 0.0001,
        }
    }
}

/// System that increments the global anomaly based on active generators.
pub fn process_gravity_siphon_system(
    mut commands: Commands,
    anomaly_opt: Option<ResMut<GravitationalAnomaly>>,
    generators: Query<&MicroSingularityGenerator>,
) {
    let mut anomaly = match anomaly_opt {
        Some(res) => res,
        None => {
            commands.insert_resource(GravitationalAnomaly::default());
            return; // Wait for next tick to process
        }
    };

    let mut total_increase = 0.0;
    for gen in generators.iter() {
        if gen.is_active {
            total_increase += gen.anomaly_rate;
        }
    }

    // Anomaly slowly dissipates if no generators are running,
    // but the buildup outpaces dissipation.
    if total_increase > 0.0 {
        anomaly.level += total_increase;
    } else {
        anomaly.level = (anomaly.level - 0.00005).max(0.0);
    }
}

/// Cross-layer system that delays Layer 2 fleets based on Layer 1 anomaly.
pub fn gravity_siphon_disruption_system(
    anomaly_opt: Option<Res<GravitationalAnomaly>>,
    mut fleets: Query<&mut InTransit>,
) {
    let anomaly = match anomaly_opt {
        Some(res) => res,
        None => return, // No anomaly exists yet
    };

    if anomaly.level <= 0.05 {
        return; // Anomaly too low to affect fleets.
    }

    let mut rng = rand::thread_rng();

    // The higher the anomaly, the higher the chance and severity of delay.
    let disruption_chance = (anomaly.level * 0.1).clamp(0.0, 0.8);

    for mut transit in fleets.iter_mut() {
        if rng.gen_bool(disruption_chance as f64) {
            // Apply a slight delay to the fleet's progress, simulating
            // the fleet fighting against warped space or recalculating routes.
            // Progress is normally incremented by (1.0 / duration), so reducing it
            // effectively extends the trip.
            let delay_penalty = rng.gen_range(0.001..0.005) * anomaly.level;
            transit.progress = (transit.progress - delay_penalty).max(0.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_process_gravity_siphon_increases_anomaly() {
        let mut world = World::new();
        world.insert_resource(GravitationalAnomaly::default());

        world.spawn(MicroSingularityGenerator {
            is_active: true,
            anomaly_rate: 0.1,
        });

        world.run_system_once(process_gravity_siphon_system).unwrap();

        let anomaly = world.resource::<GravitationalAnomaly>();
        assert_eq!(anomaly.level, 0.1);
    }

    #[test]
    fn test_process_gravity_siphon_decreases_when_inactive() {
        let mut world = World::new();
        world.insert_resource(GravitationalAnomaly { level: 0.1 });

        world.spawn(MicroSingularityGenerator {
            is_active: false,
            anomaly_rate: 0.1,
        });

        world.run_system_once(process_gravity_siphon_system).unwrap();

        let anomaly = world.resource::<GravitationalAnomaly>();
        assert!(anomaly.level < 0.1);
        assert!(anomaly.level > 0.0);
    }

    #[test]
    fn test_gravity_siphon_disrupts_fleets() {
        let mut world = World::new();
        // Set anomaly high enough to guarantee disruption in many iterations
        world.insert_resource(GravitationalAnomaly { level: 5.0 }); // 5.0 * 0.1 = 0.5 chance

        let origin = world.spawn_empty().id();
        let destination = world.spawn_empty().id();

        let fleet = world.spawn(InTransit {
            origin,
            destination,
            progress: 0.5,
            duration: 100.0,
        }).id();

        // Run the system many times to ensure RNG hits
        for _ in 0..100 {
            world.run_system_once(gravity_siphon_disruption_system).unwrap();
        }

        let transit = world.get::<InTransit>(fleet).unwrap();

        // Progress should be delayed from its starting 0.5
        assert!(transit.progress < 0.5);
    }
}

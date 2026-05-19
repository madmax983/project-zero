//! The Vermin Wheel Generator (Nova Feature).
//!
//! # The Spark
//! We have `VerminState` (an infestation mechanic) and `PowerSource` (the energy grid).
//! What if we allowed the colony to harness pests to produce free electricity?
//!
//! # The Feature
//! A building component `VerminWheel`. When active, it generates a `PowerSource` output
//! proportional to the `VerminState::severity`. However, operating these wheels actively
//! encourages Vermin growth, increasing severity slightly every tick, forcing the player
//! to ride the razor's edge of a colony collapse to sustain their energy grid.

use crate::layer1::energy::PowerSource;
use crate::layer1::vermin::VerminState;
use bevy_ecs::prelude::*;

/// A building component that harvests kinetic energy from Vermin.
#[derive(Component)]
pub struct VerminWheel;

/// System that connects Vermin to the Energy Grid via the VerminWheel.
pub fn vermin_wheel_system(
    mut vermin_state: ResMut<VerminState>,
    mut query: Query<&mut PowerSource, With<VerminWheel>>,
) {
    let mut total_wheels = 0;

    // Scale power output based on the severity of the vermin infestation.
    // Base output is severity * 0.5.
    let power_output = vermin_state.severity * 0.5;

    for mut power_source in query.iter_mut() {
        total_wheels += 1;
        power_source.output = power_output;
        power_source.active = power_output > 0.0;
    }

    // Operating these wheels actually ENCOURAGES vermin growth because you are
    // breeding/housing them in the wheels.
    if total_wheels > 0 && power_output > 0.0 {
        // Increase severity slightly for every active wheel
        vermin_state.severity =
            (vermin_state.severity + (0.1 * total_wheels as f32)).min(vermin_state.max_severity);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_vermin_wheel_system() {
        let mut world = World::new();

        // Insert global vermin state with severity 10.0
        world.insert_resource(VerminState {
            severity: 10.0,
            max_severity: 100.0,
            ..Default::default()
        });

        // Spawn a VerminWheel with a PowerSource
        let wheel = world.spawn((VerminWheel, PowerSource::default())).id();

        world.run_system_once(vermin_wheel_system).unwrap();

        // Check power output is scaled (10.0 * 0.5 = 5.0)
        let power = world.get::<PowerSource>(wheel).unwrap();
        assert_eq!(power.output, 5.0);
        assert!(power.active);

        // Check severity has increased due to the active wheel
        let vermin_state = world.resource::<VerminState>();
        // 10.0 + (0.1 * 1) = 10.1
        assert_eq!(vermin_state.severity, 10.1);
    }
}

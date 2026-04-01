//! The Boiling Blood / Fever Pitch Phenomenon (Nova Feature).
//!
//! # The Spark
//! We have a `StressTracker` system measuring the mental strain of the colony,
//! and we have a `TemperatureGrid` simulating physical heat diffusion and hazards.
//! What if we connect them?
//!
//! # The Feature
//! When a Pop's `accumulated_stress` crosses a critical threshold, their blood
//! literally begins to boil. They gain the `FeverPitch` condition, which causes
//! them to emit intense physical heat into the surrounding `TemperatureGrid`.
//!
//! # The Potential
//! This turns a psychological breakdown into a physical thermal hazard. A single
//! highly stressed Pop becomes a walking space heater (which might save lives
//! in winter!). But if multiple stressed Pops crowd together during a heatwave,
//! they will literally cook themselves and each other via `thermal_damage_system`.

use crate::layer1::nature::temperature::HeatSource;
use crate::layer1::pop::Pop;
use crate::layer1::stress::StressTracker;
use bevy_ecs::prelude::*;

/// The critical stress threshold at which a Pop's blood begins to boil.
const FEVER_PITCH_STRESS_THRESHOLD: f32 = 85.0;

/// The amount of physical heat a Pop emits while in a Fever Pitch.
const FEVER_PITCH_HEAT_OUTPUT: f32 = 30.0;

/// Marker component for a Pop currently experiencing a Fever Pitch.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct FeverPitch;

/// System that converts extreme psychological stress into physical thermal emission.
pub fn fever_pitch_system(
    mut commands: Commands,
    query: Query<(Entity, &StressTracker, Option<&FeverPitch>), With<Pop>>,
) {
    for (entity, stress, fever) in query.iter() {
        if stress.accumulated_stress >= FEVER_PITCH_STRESS_THRESHOLD {
            if fever.is_none() {
                // The Pop's blood boils. They become a walking heat source.
                commands.entity(entity).insert((
                    FeverPitch,
                    HeatSource {
                        output: FEVER_PITCH_HEAT_OUTPUT,
                    },
                ));
            }
        } else {
            if fever.is_some() {
                // The Pop has calmed down. The fever breaks.
                commands.entity(entity).remove::<FeverPitch>();
                commands.entity(entity).remove::<HeatSource>();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;

    fn setup_world() -> World {
        World::new()
    }

    #[test]
    fn test_fever_pitch_triggers_on_high_stress() {
        let mut world = setup_world();

        let pop = world
            .spawn((
                Pop,
                StressTracker {
                    accumulated_stress: 90.0, // Above 85.0
                },
            ))
            .id();

        world.run_system_once(fever_pitch_system).unwrap();

        // The Pop should now have FeverPitch and HeatSource
        assert!(
            world.get::<FeverPitch>(pop).is_some(),
            "FeverPitch should be added"
        );
        let heat = world
            .get::<HeatSource>(pop)
            .expect("HeatSource should be added");
        assert_eq!(heat.output, FEVER_PITCH_HEAT_OUTPUT);
    }

    #[test]
    fn test_fever_pitch_dormant_on_low_stress() {
        let mut world = setup_world();

        let pop = world
            .spawn((
                Pop,
                StressTracker {
                    accumulated_stress: 50.0, // Below 85.0
                },
            ))
            .id();

        world.run_system_once(fever_pitch_system).unwrap();

        assert!(
            world.get::<FeverPitch>(pop).is_none(),
            "FeverPitch should not be added"
        );
        assert!(
            world.get::<HeatSource>(pop).is_none(),
            "HeatSource should not be added"
        );
    }

    #[test]
    fn test_fever_pitch_breaks_when_stress_lowers() {
        let mut world = setup_world();

        let pop = world
            .spawn((
                Pop,
                StressTracker {
                    accumulated_stress: 40.0, // Stress dropped
                },
                FeverPitch, // But they still have the fever condition from before
                HeatSource {
                    output: FEVER_PITCH_HEAT_OUTPUT,
                },
            ))
            .id();

        world.run_system_once(fever_pitch_system).unwrap();

        // The fever should break
        assert!(
            world.get::<FeverPitch>(pop).is_none(),
            "FeverPitch should be removed"
        );
        assert!(
            world.get::<HeatSource>(pop).is_none(),
            "HeatSource should be removed"
        );
    }
}

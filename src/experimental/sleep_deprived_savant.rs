//! The Sleep-Deprived Savant (Nova Feature).
//!
//! # The Spark
//! We have Pops that lose `rest` and eventually collapse. But what if extreme exhaustion
//! unlocked something else for certain personalities? What if starvation of sleep led to
//! manic bursts of genius?
//!
//! # The Feature
//! Pops with the `Intellectual` or `Creative` traits who reach critical exhaustion (`rest` < 0.1)
//! enter a `FeverDream` state. In this state, their work speed is massively boosted (300%),
//! but they take continuous, slow health damage representing the physical toll of their mania.
//! When they finally rest (`rest` >= 0.3), the fever dream breaks.

use crate::layer1::health::Health;
use crate::layer1::needs::Needs;
use crate::layer1::pop::{Pop, Speed};
use crate::layer1::traits::{Trait, Traits};
use bevy_ecs::prelude::*;

/// Component indicating a Pop is in a manic, sleep-deprived state.
#[derive(Component, Debug, Clone, Copy)]
pub struct FeverDream;

const FEVER_DREAM_THRESHOLD: f32 = 0.10;
const FEVER_BREAK_THRESHOLD: f32 = 0.30;
const FEVER_DAMAGE_PER_TICK: f32 = 0.05; // Significant health drain
const FEVER_SPEED_MULTIPLIER: f32 = 3.0; // 300% speed

/// System to evaluate and apply the Fever Dream state.
#[allow(clippy::type_complexity)]
pub fn sleep_deprived_savant_system(
    mut commands: Commands,
    mut pops: Query<
        (
            Entity,
            &Needs,
            &Traits,
            Option<&FeverDream>,
            &mut Health,
            &mut Speed,
        ),
        With<Pop>,
    >,
) {
    for (entity, needs, traits, fever_dream, mut health, mut speed) in pops.iter_mut() {
        let is_susceptible = traits.has(Trait::Intellectual) || traits.has(Trait::Creative);

        if fever_dream.is_some() {
            // They are already in a fever dream. Check if they should snap out of it.
            if needs.rest >= FEVER_BREAK_THRESHOLD || !is_susceptible {
                commands.entity(entity).remove::<FeverDream>();
                // Remove the speed buff (reset to base, though usually Speed resets each tick anyway,
                // we handle it safely here by dividing out the multiplier if needed,
                // but since Speed::reset_speed_system runs every tick, we just don't re-apply it next tick.)
            } else {
                // Suffer the physical toll
                health.take_damage(FEVER_DAMAGE_PER_TICK);
                // Apply the manic speed boost
                speed.current *= FEVER_SPEED_MULTIPLIER;
            }
        } else {
            // Not in a fever dream. Should they enter one?
            if needs.rest < FEVER_DREAM_THRESHOLD && is_susceptible {
                commands.entity(entity).insert(FeverDream);
                // Initial application
                speed.current *= FEVER_SPEED_MULTIPLIER;
                health.take_damage(FEVER_DAMAGE_PER_TICK);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_enters_fever_dream() {
        let mut world = World::new();

        let traits = {
            let mut t = Traits::default();
            t.add(Trait::Intellectual);
            t
        };
        let pop = world
            .spawn((
                Pop,
                Needs {
                    rest: 0.05, // Below threshold
                    ..Default::default()
                },
                traits,
                Health {
                    current: 10.0,
                    max: 10.0,
                    conditions: Vec::new(),
                },
                Speed {
                    current: 1.0,
                    base: 1.0,
                    accumulator: 0.0,
                },
            ))
            .id();

        world.run_system_once(sleep_deprived_savant_system).unwrap();
        world.run_system_once(sleep_deprived_savant_system).unwrap(); // Run again to process command queue

        // Should have component
        assert!(world.get::<FeverDream>(pop).is_some());

        // Should have taken damage
        let health = world.get::<Health>(pop).unwrap();
        assert!(health.current < 10.0);

        // Speed should be boosted
        let speed = world.get::<Speed>(pop).unwrap();
        assert!(speed.current > 1.0);
    }

    #[test]
    fn test_breaks_fever_dream() {
        let mut world = World::new();

        let traits = {
            let mut t = Traits::default();
            t.add(Trait::Creative);
            t
        };
        let pop = world
            .spawn((
                Pop,
                Needs {
                    rest: 0.50, // Above break threshold
                    ..Default::default()
                },
                traits,
                FeverDream, // Already has it
                Health {
                    current: 10.0,
                    max: 10.0,
                    conditions: Vec::new(),
                },
                Speed {
                    current: 1.0,
                    base: 1.0,
                    accumulator: 0.0,
                },
            ))
            .id();

        world.run_system_once(sleep_deprived_savant_system).unwrap();
        world.run_system_once(sleep_deprived_savant_system).unwrap();

        // Should have lost the component
        assert!(world.get::<FeverDream>(pop).is_none());
    }

    #[test]
    fn test_normal_pop_ignores() {
        let mut world = World::new();

        let traits = Traits::default(); // Not creative/intellectual
        let pop = world
            .spawn((
                Pop,
                Needs {
                    rest: 0.05, // Exhausted
                    ..Default::default()
                },
                traits,
                Health {
                    current: 10.0,
                    max: 10.0,
                    conditions: Vec::new(),
                },
                Speed {
                    current: 1.0,
                    base: 1.0,
                    accumulator: 0.0,
                },
            ))
            .id();

        world.run_system_once(sleep_deprived_savant_system).unwrap();
        world.run_system_once(sleep_deprived_savant_system).unwrap();

        assert!(world.get::<FeverDream>(pop).is_none());
    }
}

#![allow(clippy::cast_precision_loss)]
//! The Sleep-Deprived Savant (Nova Feature).
//!
//! Implements the mechanic where Pops with high intelligence who reach critical
//! Sleep Deprivation enter a "Fever Dream" state. Their work speed triples,
//! but they take constant health damage and hallucinate.

use bevy_ecs::prelude::*;

use crate::layer1::gastronomy::WorkSpeedBuff;
use crate::layer1::health::Health;
use crate::layer1::needs::Needs;
use crate::layer1::pop::Pop;
use crate::layer1::traits::{Trait, Traits};
use crate::shared::log::MessageLog;

/// Component added to Pops experiencing a Fever Dream due to sleep deprivation.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct FeverDream {
    /// Tracks the duration the pop has been in the fever dream state.
    pub duration: u32,
}

/// Evaluates Pops to see if they should enter or exit the Fever Dream state,
type FeverDreamQuery<'a> = (
    Entity,
    &'a Needs,
    &'a Traits,
    &'a mut Health,
    Option<&'a mut FeverDream>,
);

pub fn fever_dream_system(
    mut commands: Commands,
    mut query: Query<FeverDreamQuery, With<Pop>>,
    mut log: Option<ResMut<MessageLog>>,
) {
    for (entity, needs, traits, mut health, fever_dream_opt) in query.iter_mut() {
        if traits.has(Trait::Intellectual) {
            let is_fever = fever_dream_opt.is_some();

            if needs.rest <= 0.1 && !is_fever {
                // Enter fever dream
                commands.entity(entity).insert(FeverDream::default());
                commands.entity(entity).insert(WorkSpeedBuff {
                    multiplier: 3.0,
                    duration: 5,
                });

                if let Some(ref mut l) = log {
                    l.add_colored(
                        "An Intellectual Pop has entered a sleep-deprived Fever Dream. Brilliant, but deadly.",
                        ratatui::style::Color::Magenta,
                    );
                }
            } else if is_fever {
                if needs.rest > 0.2 {
                    // Recovered enough rest to break the fever dream
                    commands.entity(entity).remove::<FeverDream>();

                    if let Some(ref mut l) = log {
                        l.add_colored(
                            "A Pop has woken from their Fever Dream.",
                            ratatui::style::Color::DarkGray,
                        );
                    }
                } else {
                    // Still in fever dream (rest <= 0.2), keep buff refreshed and apply damage
                    if let Some(mut fever_dream) = fever_dream_opt {
                        fever_dream.duration += 1;
                    }
                    health.take_damage(0.5);
                    commands.entity(entity).insert(WorkSpeedBuff {
                        multiplier: 3.0,
                        duration: 5, // Refresh duration
                    });
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> World {
        crate::setup::init_task_pools();
        let mut world = World::new();
        world.insert_resource(MessageLog::default());
        world
    }

    #[test]
    fn test_enter_fever_dream() {
        let mut world = setup();

        let mut traits = Traits::default();
        traits.add(Trait::Intellectual);

        let entity = world
            .spawn((
                Pop,
                Needs {
                    rest: 0.05,
                    hunger: 1.0,
                    leisure: 1.0,
                    hygiene: 1.0,
                    isolation: 0.0,
                },
                traits,
                Health {
                    current: 100.0,
                    max: 100.0,
                    has_rust_lung: false,
                },
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(fever_dream_system);
        schedule.run(&mut world);

        // Should have FeverDream and WorkSpeedBuff
        assert!(world.get::<FeverDream>(entity).is_some());
        let buff = world.get::<WorkSpeedBuff>(entity).unwrap();
        assert_eq!(buff.multiplier, 3.0);
    }

    #[test]
    fn test_fever_dream_damage() {
        let mut world = setup();

        let mut traits = Traits::default();
        traits.add(Trait::Intellectual);

        let entity = world
            .spawn((
                Pop,
                Needs {
                    rest: 0.05,
                    hunger: 1.0,
                    leisure: 1.0,
                    hygiene: 1.0,
                    isolation: 0.0,
                },
                traits,
                Health {
                    current: 100.0,
                    max: 100.0,
                    has_rust_lung: false,
                },
                FeverDream::default(), // Already in dream
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(fever_dream_system);
        schedule.run(&mut world);

        // Should have taken damage
        let health = world.get::<Health>(entity).unwrap();
        assert!(health.current < 100.0);

        // Duration should increment
        let dream = world.get::<FeverDream>(entity).unwrap();
        assert_eq!(dream.duration, 1);
    }

    #[test]
    fn test_exit_fever_dream() {
        let mut world = setup();

        let mut traits = Traits::default();
        traits.add(Trait::Intellectual);

        let entity = world
            .spawn((
                Pop,
                Needs {
                    rest: 0.5,
                    hunger: 1.0,
                    leisure: 1.0,
                    hygiene: 1.0,
                    isolation: 0.0,
                }, // High rest
                traits,
                Health {
                    current: 100.0,
                    max: 100.0,
                    has_rust_lung: false,
                },
                FeverDream::default(), // Had dream
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(fever_dream_system);
        schedule.run(&mut world);

        // Should lose FeverDream
        assert!(world.get::<FeverDream>(entity).is_none());
    }

    #[test]
    fn test_non_intellectual_ignored() {
        let mut world = setup();

        let traits = Traits::default(); // NO Intellectual

        let entity = world
            .spawn((
                Pop,
                Needs {
                    rest: 0.05,
                    hunger: 1.0,
                    leisure: 1.0,
                    hygiene: 1.0,
                    isolation: 0.0,
                },
                traits,
                Health {
                    current: 100.0,
                    max: 100.0,
                    has_rust_lung: false,
                },
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(fever_dream_system);
        schedule.run(&mut world);

        // Should NOT have FeverDream
        assert!(world.get::<FeverDream>(entity).is_none());
    }
}

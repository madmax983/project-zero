//! Placebo Protocols (Spec 256)
//! "Everything is under control."
//! Fake Orders/Propaganda to temporarily reduce panic or stress without solving the underlying problem.

use crate::layer1::notifications::NotificationQueue;
use crate::layer1::pop::Pop;
use crate::layer1::stress::StressTracker;
use crate::layer1::traits::{Trait, Traits};
use bevy_ecs::prelude::*;

/// Type of Placebo Protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlaceboProtocol {
    /// Reduces Panic
    FakeReinforcements,
    /// Reduces Sickness Fear
    VitaminX,
    /// Reduces Collapse Fear
    SafetyInspection,
}

/// Active Placebo instance.
#[derive(Component, Debug, Clone)]
pub struct ActivePlacebo {
    /// Protocol type.
    pub protocol: PlaceboProtocol,
    /// Duration remaining in ticks.
    pub duration: f32,
    /// Amount of stress relief applied.
    pub stress_relief: f32,
    /// Whether the placebo was revealed as fake.
    pub revealed: bool,
    /// Whether the stress relief has been applied once.
    pub applied: bool,
}

/// Applies the stress relief initially, then drains duration. If it expires naturally, it gets revealed.
pub fn placebo_tick_system(
    mut placebos: Query<(Entity, &mut ActivePlacebo)>,
    mut pops: Query<(&mut StressTracker, &Traits), With<Pop>>,
) {
    for (_, mut placebo) in placebos.iter_mut() {
        // If not applied, apply once to all non-distrustful pops.
        if !placebo.applied && !placebo.revealed {
            for (mut stress, traits) in pops.iter_mut() {
                if !traits.has(Trait::Distrustful) {
                    stress.accumulated_stress =
                        (stress.accumulated_stress - placebo.stress_relief).max(0.0);
                }
            }
            placebo.applied = true;
        }

        if placebo.duration > 0.0 {
            placebo.duration -= 1.0;
        } else if !placebo.revealed {
            placebo.revealed = true;
        }
    }
}

/// Handles betrayal logic: if revealed, applies double the stress penalty and adds the Distrustful trait.
pub fn reveal_betrayal_system(
    mut commands: Commands,
    query: Query<(Entity, &ActivePlacebo)>,
    mut pops: Query<(&mut StressTracker, &mut Traits), With<Pop>>,
    mut notification_queue: Option<ResMut<NotificationQueue>>,
    time: Option<Res<crate::shared::time::SimulationTime>>,
) {
    for (entity, placebo) in query.iter() {
        if placebo.revealed {
            for (mut stress, mut traits) in pops.iter_mut() {
                // If they weren't distrustful yet, they were affected by the placebo
                if !traits.has(Trait::Distrustful) {
                    // Penalty is returning the original relief PLUS another penalty amount,
                    // Effectively 2x the original relief.
                    stress.accumulated_stress += placebo.stress_relief * 2.0;
                    traits.add(Trait::Distrustful);
                }
            }

            // Optional Notification
            if let (Some(queue), Some(sim_time)) =
                (notification_queue.as_deref_mut(), time.as_deref())
            {
                let text = match placebo.protocol {
                    PlaceboProtocol::FakeReinforcements => {
                        "The Reinforcements were a lie! Panic Spikes!"
                    }
                    PlaceboProtocol::VitaminX => "Vitamin X was sugar! Health Anxiety Spikes!",
                    PlaceboProtocol::SafetyInspection => {
                        "The inspection was forged! Collapse Panic Spikes!"
                    }
                };
                queue.add_error(text, sim_time.tick);
            }

            commands.entity(entity).despawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{placebo_tick_system, reveal_betrayal_system, ActivePlacebo, PlaceboProtocol};
    use crate::layer1::pop::Pop;
    use crate::layer1::stress::StressTracker;
    use crate::layer1::traits::{Trait, Traits};
    use bevy_ecs::prelude::*;

    #[test]
    fn test_placebo_reduces_stress_temporarily() {
        let mut world = World::new();
        let pop = world
            .spawn((
                Pop,
                StressTracker {
                    accumulated_stress: 80.0,
                },
                Traits::default(),
            ))
            .id();

        // Issue "Fake Reinforcements"
        world.spawn(ActivePlacebo {
            protocol: PlaceboProtocol::FakeReinforcements,
            duration: 10.0,
            stress_relief: 20.0,
            revealed: false,
            applied: false,
        });

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(placebo_tick_system);
        schedule.run(&mut world);

        let stress = world.get::<StressTracker>(pop).unwrap();
        assert!(stress.accumulated_stress <= 60.0 + f32::EPSILON);
    }

    #[test]
    fn test_betrayal_spikes_stress_and_adds_trait() {
        let mut world = World::new();
        let pop = world
            .spawn((
                Pop,
                StressTracker {
                    accumulated_stress: 60.0,
                }, // Reduced level
                Traits::default(),
            ))
            .id();

        // Expire/Fail a placebo
        world.spawn(ActivePlacebo {
            protocol: PlaceboProtocol::FakeReinforcements,
            duration: 0.0, // Expired
            stress_relief: 20.0,
            revealed: true, // Failed
            applied: true,
        });

        let mut schedule = Schedule::default();
        schedule.add_systems(reveal_betrayal_system);
        schedule.run(&mut world);

        let stress = world.get::<StressTracker>(pop).unwrap();
        // Stress should return to original (80) PLUS penalty (e.g. +40) = 100
        assert!(stress.accumulated_stress >= 90.0);

        let traits = world.get::<Traits>(pop).unwrap();
        assert!(traits.has(Trait::Distrustful));
    }

    #[test]
    fn test_distrustful_pops_ignore_placebos() {
        let mut world = World::new();
        let mut pop_traits = Traits::default();
        pop_traits.add(Trait::Distrustful);

        let pop = world
            .spawn((
                Pop,
                StressTracker {
                    accumulated_stress: 80.0,
                },
                pop_traits,
            ))
            .id();

        world.spawn(ActivePlacebo {
            protocol: PlaceboProtocol::VitaminX,
            duration: 10.0,
            stress_relief: 20.0,
            revealed: false,
            applied: false,
        });

        let mut schedule = Schedule::default();
        schedule.add_systems(placebo_tick_system);
        schedule.run(&mut world);

        let stress = world.get::<StressTracker>(pop).unwrap();
        // Since they are distrustful, stress shouldn't have changed.
        assert!((stress.accumulated_stress - 80.0).abs() < f32::EPSILON);
    }
}

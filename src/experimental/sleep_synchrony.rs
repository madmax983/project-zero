#![allow(clippy::type_complexity)]
//! Sleep Synchrony (Nova Feature).
//!
//! # The Spark
//! We have a `SatisfyRest` action, a `DreamJournal`, and a grid system.
//!
//! # The Feature
//! Pops sleeping in close proximity synchronize their dreams. They regenerate their `rest` need much faster ("Restful Synergy").
//! However, if any pop in the synchronized block experiences a nightmare, the terror cascades through the network, instantly waking everyone up in a state of mass panic (Dazing breakdown).
//!
//! # The Potential
//! Forces players to weigh the massive efficiency of high-density barracks against the catastrophic risk of a single stressed pop triggering a colony-wide psychological collapse.
//!
//! # Risk
//! Low. Self-contained in `src/experimental/sleep_synchrony.rs` behind the `nova` feature flag.

use crate::layer1::core::map::GridPosition;
use crate::layer1::mind::utility_types::{ActionType, PopAction};
use crate::layer1::needs::Needs;
use crate::layer1::pop::Pop;
use crate::layer1::psychology::dreams::{DreamJournal, DreamtThisSleep};
use crate::layer1::psychology::stress::{Breakdown, BreakdownType};
use crate::shared::log::MessageLog;
use bevy_ecs::prelude::*;

/// System that handles sleep synchrony between proximate sleeping pops.
pub fn sleep_synchrony_system(
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            &GridPosition,
            &mut PopAction,
            &mut Needs,
            Option<&DreamJournal>,
            Option<&DreamtThisSleep>,
        ),
        With<Pop>,
    >,
    mut log: Option<ResMut<MessageLog>>,
) {
    // Collect all sleeping pops
    let mut sleeping_pops = Vec::new();
    for (entity, pos, action, _, journal_opt, dreamt_opt) in query.iter() {
        if action.current == ActionType::SatisfyRest {
            // Check if they had a nightmare recently
            let mut has_nightmare = false;
            if dreamt_opt.is_some() {
                if let Some(journal) = journal_opt {
                    if let Some(dream) = &journal.last_dream {
                        if dream.is_nightmare {
                            has_nightmare = true;
                        }
                    }
                }
            }
            sleeping_pops.push((entity, *pos, has_nightmare));
        }
    }

    let mut nightmare_cascades = Vec::new();

    // Evaluate synchrony blocks (distance <= 3)
    for (entity, pos, mut action, mut needs, journal_opt, dreamt_opt) in query.iter_mut() {
        if action.current == ActionType::SatisfyRest {
            let mut connected_peers = 0;

            // Determine if *this* pop has a nightmare
            let mut has_nightmare = false;
            if dreamt_opt.is_some() {
                if let Some(journal) = journal_opt {
                    if let Some(dream) = &journal.last_dream {
                        if dream.is_nightmare {
                            has_nightmare = true;
                        }
                    }
                }
            }

            let mut connected_nightmare = has_nightmare;

            for (other_entity, other_pos, other_has_nightmare) in &sleeping_pops {
                if *other_entity != entity && pos.distance_chebyshev(*other_pos) <= 3 {
                    connected_peers += 1;
                    if *other_has_nightmare {
                        connected_nightmare = true;
                    }
                }
            }

            if connected_peers >= 2 {
                // At least a group of 3 (self + 2)
                if connected_nightmare {
                    // Cascade!
                    action.current = ActionType::Idle;
                    needs.rest = 0.1; // Exhausted from panic
                    needs.leisure = 0.0; // Stressed
                    commands.entity(entity).insert(Breakdown {
                        breakdown_type: BreakdownType::Dazing,
                        duration_remaining: 100,
                    });
                    nightmare_cascades.push(entity);
                } else {
                    // Synergy buff: Faster rest regeneration
                    needs.rest = (needs.rest + 0.01 * connected_peers as f32).clamp(0.0, 1.0);
                }
            }
        }
    }

    if !nightmare_cascades.is_empty() {
        if let Some(ref mut l) = log {
            l.add_colored(
                format!(
                    "A nightmare cascaded through a synchronized sleeping block! {} pops woke up in a panic.",
                    nightmare_cascades.len()
                ),
                ratatui::style::Color::Red,
            );
        }
    }
}

pub fn register(schedule: &mut Schedule) {
    schedule.add_systems(sleep_synchrony_system);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::psychology::dreams::Dream;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_sleep_synchrony_buff() {
        let mut world = World::new();

        // Spawn 3 pops sleeping close together
        let mut entities = Vec::new();
        for i in 0..3 {
            entities.push(
                world
                    .spawn((
                        Pop,
                        GridPosition { x: 5, y: 5 + i },
                        PopAction {
                            current: ActionType::SatisfyRest,
                            ..Default::default()
                        },
                        Needs {
                            rest: 0.5,
                            ..Default::default()
                        },
                    ))
                    .id(),
            );
        }

        world.run_system_once(sleep_synchrony_system).unwrap();

        for e in entities {
            let needs = world.get::<Needs>(e).unwrap();
            assert!(
                needs.rest > 0.5,
                "Rest should regenerate faster due to synchrony"
            );
        }
    }

    #[test]
    fn test_sleep_synchrony_nightmare_cascade() {
        let mut world = World::new();

        // Pop 1 has a nightmare
        let p1 = world
            .spawn((
                Pop,
                GridPosition { x: 5, y: 5 },
                PopAction {
                    current: ActionType::SatisfyRest,
                    ..Default::default()
                },
                Needs {
                    rest: 0.5,
                    leisure: 0.5,
                    ..Default::default()
                },
                DreamJournal {
                    last_dream: Some(Dream {
                        content: "bad dream".to_string(),
                        tick: 0,
                        impact: -0.1,
                        is_nightmare: true,
                    }),
                    history: vec![],
                },
                DreamtThisSleep,
            ))
            .id();

        // Pop 2 and 3 are sleeping nearby
        let p2 = world
            .spawn((
                Pop,
                GridPosition { x: 5, y: 6 },
                PopAction {
                    current: ActionType::SatisfyRest,
                    ..Default::default()
                },
                Needs {
                    rest: 0.5,
                    leisure: 0.5,
                    ..Default::default()
                },
            ))
            .id();

        let p3 = world
            .spawn((
                Pop,
                GridPosition { x: 5, y: 7 },
                PopAction {
                    current: ActionType::SatisfyRest,
                    ..Default::default()
                },
                Needs {
                    rest: 0.5,
                    leisure: 0.5,
                    ..Default::default()
                },
            ))
            .id();

        world.run_system_once(sleep_synchrony_system).unwrap();

        // All should wake up in a panic
        for e in [p1, p2, p3] {
            let action = world.get::<PopAction>(e).unwrap();
            let needs = world.get::<Needs>(e).unwrap();
            let breakdown = world.get::<Breakdown>(e);

            assert_eq!(action.current, ActionType::Idle);
            assert!(breakdown.is_some());
            assert_eq!(needs.rest, 0.1);
        }
    }
}

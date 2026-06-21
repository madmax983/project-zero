//! Acoustic Hallucinations (Nova Feature).
//!
//! # The Spark
//! We have a `NoiseSource` system for `NoiseMap`, `MentalBreakType`, and `Morale`.
//! What if extreme, persistent noise drives Pops crazy, not just through stress, but by inducing hallucinations?
//!
//! # The Feature
//! Pops in areas of extreme noise (e.g. near a `FeralChoir` or loud machinery) have a chance to experience `AcousticHallucinations`.
//! When hallucinating, their `ActionType` is forced into `Daze`, simulating their disorientation and terror, while dropping their `leisure` need.
//! This turns heavy industrial areas or biological noise hazards into psychological minefields if not properly insulated or suppressed with `SonicSuppression`.

use crate::layer1::acoustic::NoiseMap;
use crate::layer1::map::GridPosition;
use crate::layer1::mind::utility_types::{ActionType, PopAction};
use crate::layer1::needs::Needs;
use crate::layer1::pop::Pop;
use bevy_ecs::prelude::*;
use rand::Rng;

const HALLUCINATION_NOISE_THRESHOLD: f32 = 0.8;
const HALLUCINATION_CHANCE: f64 = 0.05;

#[derive(Component)]
pub struct Hallucinating {
    pub duration_remaining: u32,
}

pub fn acoustic_hallucinations_system(
    mut commands: Commands,
    acoustic_grid: Option<Res<NoiseMap>>,
    mut pops: Query<
        (
            Entity,
            &GridPosition,
            &mut Needs,
            Option<&mut Hallucinating>,
        ),
        With<Pop>,
    >,
    mut log: Option<ResMut<crate::shared::log::MessageLog>>,
) {
    if let Some(grid) = acoustic_grid {
        let mut rng = rand::thread_rng();

        for (entity, pos, mut needs, hallucinating_opt) in pops.iter_mut() {
            if let Some(mut hallucinating) = hallucinating_opt {
                if hallucinating.duration_remaining > 0 {
                    hallucinating.duration_remaining -= 1;
                } else {
                    commands.entity(entity).remove::<Hallucinating>();
                }
                continue; // Already hallucinating
            }

            if pos.x < 0 || pos.y < 0 {
                continue;
            }

            let noise_level = grid.get(pos.x, pos.y);

            if noise_level > HALLUCINATION_NOISE_THRESHOLD && rng.gen_bool(HALLUCINATION_CHANCE) {
                commands.entity(entity).insert(Hallucinating {
                    duration_remaining: 100,
                });

                needs.leisure = (needs.leisure - 0.2).max(0.0);

                if let Some(ref mut l) = log {
                    l.add_colored(
                        format!("A pop is experiencing acoustic hallucinations at ({}, {}) due to extreme noise!", pos.x, pos.y),
                        ratatui::style::Color::Magenta,
                    );
                }
            }
        }
    }
}

pub fn apply_hallucination_daze_system(mut pops: Query<&mut PopAction, With<Hallucinating>>) {
    for mut action in pops.iter_mut() {
        action.current = ActionType::Daze;
    }
}

pub fn register(schedule: &mut Schedule) {
    schedule.add_systems((
        acoustic_hallucinations_system,
        apply_hallucination_daze_system.after(acoustic_hallucinations_system),
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_acoustic_hallucinations() {
        let mut world = World::new();

        let mut grid = NoiseMap::new(10, 10);
        grid.set(5, 5, 1.0); // Extreme noise
        world.insert_resource(grid);

        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 5, y: 5 },
                Needs {
                    leisure: 1.0,
                    ..Default::default()
                },
                PopAction {
                    current: ActionType::Idle,
                    ..Default::default()
                },
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems((
            acoustic_hallucinations_system,
            apply_hallucination_daze_system.after(acoustic_hallucinations_system),
        ));

        // Run until hallucination triggers
        let mut triggered = false;
        for _ in 0..1000 {
            schedule.run(&mut world);
            if world.get::<Hallucinating>(pop).is_some() {
                triggered = true;
                break;
            }
        }

        assert!(
            triggered,
            "Pop should eventually hallucinate in extreme noise"
        );

        let needs = world.get::<Needs>(pop).unwrap();
        assert!(
            needs.leisure < 1.0,
            "Leisure should drop when hallucinating"
        );

        let action = world.get::<PopAction>(pop).unwrap();
        assert_eq!(
            action.current,
            ActionType::Daze,
            "Action should be forced to Daze"
        );
    }
}

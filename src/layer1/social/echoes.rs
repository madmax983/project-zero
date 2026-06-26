use crate::layer1::map::GridPosition;
use crate::layer1::skills::{SkillType, Skills};
use crate::layer1::stress::StressTracker;
use bevy_ecs::prelude::*;
#[cfg(test)]
use bevy_ecs::system::RunSystemOnce;
use bevy_time::{Time, Timer, TimerMode};

#[derive(Component)]
pub struct EchoSource {
    pub spawn_chance: f32, // e.g. 0.01 per tick
}

#[derive(Component)]
pub struct Echo {
    pub radius: f32,
    pub timer: Timer,
}

pub fn echo_spawn_system(mut commands: Commands, query: Query<(&EchoSource, &GridPosition)>) {
    for (source, pos) in query.iter() {
        if source.spawn_chance >= 1.0 {
            commands.spawn((
                Echo {
                    radius: 3.0,
                    timer: Timer::from_seconds(60.0, TimerMode::Once),
                },
                *pos,
            ));
        }
    }
}

use crate::layer1::mind::utility_types::{ActionType, PopAction};
use crate::layer1::psychology::traits::{Trait, Traits};

#[allow(clippy::type_complexity)]
pub fn echo_aura_system(
    echoes: Query<(&Echo, &GridPosition)>,
    mut pops: Query<(
        &GridPosition,
        &mut Skills,
        &mut StressTracker,
        Option<&PopAction>,
        Option<&mut Traits>,
    )>,
) {
    for (pop_pos, mut skills, mut stress, action, mut traits) in pops.iter_mut() {
        for (echo, echo_pos) in echoes.iter() {
            let dx = (pop_pos.x as f32 - echo_pos.x as f32).abs();
            let dy = (pop_pos.y as f32 - echo_pos.y as f32).abs();
            let dist = (dx * dx + dy * dy).sqrt();

            if dist <= echo.radius {
                // Only grant XP if researching
                if let Some(act) = action {
                    if act.current == ActionType::Research {
                        skills
                            .xp
                            .entry(SkillType::Engineering)
                            .and_modify(|x| *x += 10.0)
                            .or_insert(10.0);
                    }
                }

                // Stress is passive
                stress.accumulated_stress += 5.0;

                // Insanity cult logic
                if stress.accumulated_stress > 80.0 {
                    if let Some(ref mut t) = traits {
                        if !t.has(Trait::EngineCultist) {
                            t.add(Trait::EngineCultist);
                        }
                    }
                }
            }
        }
    }
}

pub fn echo_despawn_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Echo)>,
) {
    for (entity, mut echo) in query.iter_mut() {
        echo.timer.tick(time.delta());
        if echo.timer.finished() {
            commands.entity(entity).despawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::map::GridPosition;
    use crate::layer1::skills::{SkillType, Skills};
    use crate::layer1::stress::StressTracker;
    use bevy_app::App;
    use bevy_time::TimePlugin;
    use core::time::Duration;

    #[test]
    fn test_echo_spawns_from_source() {
        let mut world = World::new();
        // Create an EchoSource
        let _source = world
            .spawn((
                EchoSource { spawn_chance: 1.0 },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(echo_spawn_system);
        schedule.run(&mut world);

        let mut echo_query = world.query::<&Echo>();
        assert_eq!(
            echo_query.iter(&world).count(),
            1,
            "An Echo should have spawned from the source."
        );
    }

    #[test]
    fn test_pop_near_echo_gains_bonus_xp_and_stress() {
        let mut world = World::new();
        // Spawn an Echo
        world.spawn((
            Echo {
                radius: 2.0,
                timer: Timer::from_seconds(60.0, TimerMode::Once),
            },
            GridPosition { x: 5, y: 5 },
        ));

        // Spawn a Pop nearby
        let action = PopAction {
            current: ActionType::Research,
            ..Default::default()
        };

        let pop = world
            .spawn((
                GridPosition { x: 6, y: 5 },
                Skills::default(),
                StressTracker::default(),
                action,
                Traits::default(),
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(echo_aura_system);

        // Run a few ticks
        for _ in 0..5 {
            schedule.run(&mut world);
        }

        let skills = world.get::<Skills>(pop).unwrap();
        let stress = world.get::<StressTracker>(pop).unwrap();

        assert!(
            skills.get_xp(SkillType::Engineering) > 0.0,
            "Pop should gain XP from being near the Echo."
        );
        assert!(
            stress.accumulated_stress > 0.0,
            "Pop should gain stress from being near the Echo."
        );
    }

    #[test]
    fn test_pop_near_echo_gains_stress_but_not_xp_if_not_researching() {
        let mut world = World::new();
        world.spawn((
            Echo {
                radius: 2.0,
                timer: Timer::from_seconds(60.0, TimerMode::Once),
            },
            GridPosition { x: 5, y: 5 },
        ));

        let action = PopAction {
            current: ActionType::Idle,
            ..Default::default()
        };

        let pop = world
            .spawn((
                GridPosition { x: 6, y: 5 },
                Skills::default(),
                StressTracker::default(),
                action,
                Traits::default(),
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(echo_aura_system);

        schedule.run(&mut world);

        let skills = world.get::<Skills>(pop).unwrap();
        let stress = world.get::<StressTracker>(pop).unwrap();

        assert_eq!(
            skills.get_xp(SkillType::Engineering),
            0.0,
            "Pop should NOT gain XP because they are not researching."
        );
        assert!(
            stress.accumulated_stress > 0.0,
            "Pop should STILL gain stress from being near the Echo."
        );
    }

    #[test]
    fn test_pop_near_echo_gains_cultist_trait_on_high_stress() {
        let mut world = World::new();
        world.spawn((
            Echo {
                radius: 2.0,
                timer: Timer::from_seconds(60.0, TimerMode::Once),
            },
            GridPosition { x: 5, y: 5 },
        ));

        let pop = world
            .spawn((
                GridPosition { x: 6, y: 5 },
                Skills::default(),
                StressTracker {
                    accumulated_stress: 79.0,
                },
                PopAction::default(),
                Traits::default(),
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(echo_aura_system);

        schedule.run(&mut world);

        let stress = world.get::<StressTracker>(pop).unwrap();
        let traits = world.get::<Traits>(pop).unwrap();

        assert!(
            stress.accumulated_stress > 80.0,
            "Pop should gain stress to push them over the threshold."
        );
        assert!(
            traits.has(Trait::EngineCultist),
            "Pop should have gained the EngineCultist trait."
        );
    }

    #[test]
    fn test_pop_far_from_echo_unaffected() {
        let mut world = World::new();
        // Spawn an Echo
        world.spawn((
            Echo {
                radius: 2.0,
                timer: Timer::from_seconds(60.0, TimerMode::Once),
            },
            GridPosition { x: 5, y: 5 },
        ));

        // Spawn a Pop far away
        let pop = world
            .spawn((
                GridPosition { x: 20, y: 20 },
                Skills::default(),
                StressTracker::default(),
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(echo_aura_system);
        schedule.run(&mut world);

        let skills = world.get::<Skills>(pop).unwrap();
        let stress = world.get::<StressTracker>(pop).unwrap();

        assert_eq!(
            skills.get_xp(SkillType::Engineering),
            0.0,
            "Pop far away should not gain XP."
        );
        assert_eq!(
            stress.accumulated_stress, 0.0,
            "Pop far away should not gain stress."
        );
    }

    #[test]
    fn test_echo_despawns_after_timer() {
        let mut app = App::new();
        app.add_plugins(TimePlugin);
        app.add_systems(bevy_app::Update, echo_despawn_system);

        let echo_entity = app
            .world_mut()
            .spawn((
                Echo {
                    radius: 3.0,
                    timer: Timer::from_seconds(1.0, TimerMode::Once),
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        // Setup time delta manually and run schedule
        let mut time = app.world_mut().resource_mut::<Time>();
        time.advance_by(Duration::from_secs_f32(1.5));

        // Use schedule directly to ensure delta is used by the system
        app.world_mut()
            .run_system_once(echo_despawn_system)
            .unwrap();

        assert!(
            app.world().get_entity(echo_entity).is_err(),
            "Echo should have despawned after timer finished."
        );
    }
}

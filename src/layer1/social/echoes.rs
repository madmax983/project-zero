use crate::layer1::core::map::GridPosition;
use crate::layer1::psychology::stress::StressTracker;
use crate::layer1::skills::{SkillType, Skills};
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct EchoSource {
    pub spawn_chance: f32, // e.g. 0.01 per tick
}

#[derive(Component)]
pub struct Echo {
    pub radius: f32,
}

pub fn echo_spawn_system(mut commands: Commands, query: Query<(&EchoSource, &GridPosition)>) {
    for (source, pos) in query.iter() {
        if source.spawn_chance >= 1.0 {
            commands.spawn((Echo { radius: 3.0 }, *pos));
        }
    }
}

pub fn echo_aura_system(
    echoes: Query<(&Echo, &GridPosition)>,
    mut pops: Query<(&GridPosition, &mut Skills, &mut StressTracker)>,
) {
    for (pop_pos, mut skills, mut stress) in pops.iter_mut() {
        for (echo, echo_pos) in echoes.iter() {
            let dx = (pop_pos.x as f32 - echo_pos.x as f32).abs();
            let dy = (pop_pos.y as f32 - echo_pos.y as f32).abs();
            let dist = (dx * dx + dy * dy).sqrt();

            if dist <= echo.radius {
                let xp_entry = skills.xp.entry(SkillType::Engineering).or_insert(0.0);
                *xp_entry += 10.0;
                stress.accumulated_stress += 5.0;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::core::map::GridPosition;
    use crate::layer1::psychology::stress::StressTracker;
    use crate::layer1::skills::{SkillType, Skills};

    #[test]
    fn test_echo_spawns_from_source() {
        let mut world = World::new();
        // Create an EchoSource
        world.spawn((
            EchoSource { spawn_chance: 1.0 },
            GridPosition { x: 5, y: 5 },
        ));

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
        world.spawn((Echo { radius: 2.0 }, GridPosition { x: 5, y: 5 }));

        // Spawn a Pop nearby
        let pop = world
            .spawn((
                GridPosition { x: 6, y: 5 },
                Skills::default(),
                StressTracker::default(),
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
            *skills.xp.get(&SkillType::Engineering).unwrap_or(&0.0) > 0.0,
            "Pop should gain XP from being near the Echo."
        );
        assert!(
            stress.accumulated_stress > 0.0,
            "Pop should gain stress from being near the Echo."
        );
    }

    #[test]
    fn test_pop_far_from_echo_unaffected() {
        let mut world = World::new();
        // Spawn an Echo
        world.spawn((Echo { radius: 2.0 }, GridPosition { x: 5, y: 5 }));

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
            *skills.xp.get(&SkillType::Engineering).unwrap_or(&0.0),
            0.0,
            "Pop far away should not gain XP."
        );
        assert_eq!(
            stress.accumulated_stress, 0.0,
            "Pop far away should not gain stress."
        );
    }
}

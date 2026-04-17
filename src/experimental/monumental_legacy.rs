//! Monumental Legacy (Nova Feature).
//!
//! # The Spark
//! Pops gain skills over their lifetime, making them efficient workers. But what happens
//! when a master artisan or legendary miner reaches the peak of their craft? What if they
//! could leave a lasting, physical manifestation of their expertise behind?
//!
//! # The Feature
//! Pops that reach Level 10 in a specific `SkillType` have a rare chance to enter a
//! `MonumentalTrance`. In this trance, they will physically construct a `LegacyMonument`
//! entity at their current position. The `LegacyMonument` acts as a beacon, projecting
//! a `LegacyAura` that increases the work speed of any nearby Pops performing jobs
//! related to that specific skill.

use crate::layer1::map::GridPosition;
use crate::layer1::pop::Pop;
use crate::layer1::skills::{SkillType, Skills};
use bevy_ecs::prelude::*;
use rand::Rng;

/// Indicates a pop is in a trance and about to build a monument.
#[derive(Component, Debug)]
pub struct MonumentalTrance {
    pub skill: SkillType,
    pub duration: u32,
}

/// A constructed monument that buffs nearby workers.
#[derive(Component, Debug)]
pub struct LegacyMonument {
    pub skill: SkillType,
}

const MONUMENT_RADIUS: u32 = 10;
const TRANCE_CHANCE: f64 = 0.05;

#[allow(clippy::type_complexity)]
pub fn monumental_trance_system(
    mut commands: Commands,
    pops: Query<(Entity, &Skills), (With<Pop>, Without<MonumentalTrance>)>,
) {
    let mut rng = rand::thread_rng();

    for (entity, skills) in pops.iter() {
        for (skill, xp) in skills.xp.iter() {
            // Level 10 requires 10,000 XP
            if *xp >= 10000.0 && rng.gen_bool(TRANCE_CHANCE) {
                commands.entity(entity).insert(MonumentalTrance {
                    skill: *skill,
                    duration: 10, // Takes 10 ticks to build
                });
                break; // Only enter trance for one skill at a time
            }
        }
    }
}

pub fn construct_monument_system(
    mut commands: Commands,
    mut pops: Query<(Entity, &mut MonumentalTrance, &GridPosition), With<Pop>>,
) {
    for (entity, mut trance, pos) in pops.iter_mut() {
        if trance.duration > 0 {
            trance.duration -= 1;
        } else {
            // Construct the monument
            commands.spawn((
                LegacyMonument {
                    skill: trance.skill,
                },
                *pos,
            ));

            // Remove the trance
            commands.entity(entity).remove::<MonumentalTrance>();
        }
    }
}

pub fn legacy_aura_system(
    monuments: Query<(&GridPosition, &LegacyMonument)>,
    mut workers: Query<(
        &GridPosition,
        &mut crate::layer1::pop::Speed,
        &crate::layer1::pop::Job,
    )>,
) {
    // Helper to map AssignmentType to SkillType
    let maps_to_skill = |job_type: crate::layer1::utility_types::AssignmentType,
                         skill: SkillType|
     -> bool {
        match (job_type, skill) {
            (crate::layer1::utility_types::AssignmentType::DeepMining, SkillType::Mining) => true,
            (crate::layer1::utility_types::AssignmentType::FarmWorker, SkillType::Farming) => true,
            // Expand as more mappings are formally defined, but this serves the core mechanic
            _ => false,
        }
    };

    for (worker_pos, mut speed, job) in workers.iter_mut() {
        let mut buff_applied = false;
        for (monument_pos, monument) in monuments.iter() {
            if worker_pos.distance_chebyshev(*monument_pos) <= MONUMENT_RADIUS
                && maps_to_skill(job.job_type, monument.skill)
            {
                buff_applied = true;
                break;
            }
        }

        // Use a buff modifier. The base reset happens in reset_speed_system
        if buff_applied {
            speed.current *= 1.2; // Apply a 20% buff based on the current speed
        }
    }
}

pub fn register(schedule: &mut Schedule) {
    schedule.add_systems((
        monumental_trance_system,
        construct_monument_system,
        legacy_aura_system,
    ));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::pop::{Job, Speed};
    use crate::layer1::utility_types::AssignmentType;

    #[test]
    fn test_master_pop_enters_trance() {
        let mut world = World::new();

        let mut skills = Skills::default();
        // 10,000 XP = level 10
        skills.add_xp(SkillType::Mining, 10000.0);

        let pop = world.spawn((Pop, skills)).id();

        // Run until trance triggers (due to RNG)
        let mut triggered = false;
        for _ in 0..100 {
            let mut schedule = Schedule::default();
            schedule.add_systems(monumental_trance_system);
            schedule.run(&mut world);
schedule.run(&mut world);

            if world.get::<MonumentalTrance>(pop).is_some() {
                triggered = true;
                break;
            }
        }

        assert!(triggered, "Master pop should eventually enter trance");
    }

    #[test]
    fn test_apprentice_pop_does_not_enter_trance() {
        let mut world = World::new();

        let mut skills = Skills::default();
        skills.add_xp(SkillType::Mining, 100.0); // Level 1

        let pop = world.spawn((Pop, skills)).id();

        for _ in 0..100 {
            let mut schedule = Schedule::default();
            schedule.add_systems(monumental_trance_system);
            schedule.run(&mut world);
schedule.run(&mut world);
        }

        assert!(
            world.get::<MonumentalTrance>(pop).is_none(),
            "Apprentice pop should never enter trance"
        );
    }

    #[test]
    fn test_trance_creates_monument() {
        let mut world = World::new();

        let pop = world
            .spawn((
                Pop,
                MonumentalTrance {
                    skill: SkillType::Mining,
                    duration: 0, // Will finish immediately
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(construct_monument_system);
        schedule.run(&mut world);

        assert!(
            world.get::<MonumentalTrance>(pop).is_none(),
            "Trance should be removed"
        );

        let mut query = world.query::<(&LegacyMonument, &GridPosition)>();
        let mut found = false;
        for (monument, pos) in query.iter(&world) {
            if monument.skill == SkillType::Mining && pos.x == 5 && pos.y == 5 {
                found = true;
            }
        }

        assert!(found, "Monument should be created at pop's location");
    }

    #[test]
    fn test_monument_buffs_nearby_workers() {
        let mut world = World::new();

        // Spawn a monument for mining
        world.spawn((
            LegacyMonument {
                skill: SkillType::Mining,
            },
            GridPosition { x: 0, y: 0 },
        ));

        // Spawn a nearby miner
        let miner = world
            .spawn((
                GridPosition { x: 2, y: 2 }, // In range
                Speed::default(),
                Job {
                    workplace: Entity::from_raw(0), // Dummy
                    job_type: AssignmentType::DeepMining,
                },
            ))
            .id();

        // Spawn a far miner
        let far_miner = world
            .spawn((
                GridPosition { x: 20, y: 20 }, // Out of range
                Speed::default(),
                Job {
                    workplace: Entity::from_raw(0),
                    job_type: AssignmentType::DeepMining,
                },
            ))
            .id();

        // Spawn a nearby farmer
        let farmer = world
            .spawn((
                GridPosition { x: 2, y: 2 }, // In range, wrong job
                Speed::default(),
                Job {
                    workplace: Entity::from_raw(0),
                    job_type: AssignmentType::FarmWorker,
                },
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(legacy_aura_system);
        schedule.run(&mut world);
schedule.run(&mut world);

        let miner_speed = world.get::<Speed>(miner).unwrap().current;
        let far_miner_speed = world.get::<Speed>(far_miner).unwrap().current;
        let farmer_speed = world.get::<Speed>(farmer).unwrap().current;

        assert!(
            miner_speed > 1.0,
            "Nearby miner should be buffed (got {})",
            miner_speed
        );
        assert!(
            (far_miner_speed - 1.0).abs() < f32::EPSILON,
            "Far miner should not be buffed"
        );
        assert!(
            (farmer_speed - 1.0).abs() < f32::EPSILON,
            "Nearby farmer should not be buffed"
        );
    }
}

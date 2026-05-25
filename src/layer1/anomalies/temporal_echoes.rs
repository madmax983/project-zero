use crate::layer1::architecture::building::Building;
use crate::layer1::map::GridPosition;
use crate::layer1::pop::Pop;
use crate::layer1::psychology::stress::StressTracker;
use crate::layer1::skills::{SkillType, Skills};
use bevy_ecs::prelude::*;
use rand::Rng;

#[derive(Component)]
pub struct ChronoAnomaly {
    pub radius: f32,
}

#[derive(Component)]
pub struct BuildingAge {
    pub ticks: u32,
}

pub fn process_temporal_echo_system(
    anomalies: Query<(&ChronoAnomaly, &GridPosition)>,
    mut pops: Query<(&mut Skills, &mut StressTracker, &GridPosition), With<Pop>>,
) {
    let mut rng = rand::thread_rng();
    for (anomaly, anomaly_pos) in anomalies.iter() {
        for (mut skills, mut stress, pop_pos) in pops.iter_mut() {
            if (anomaly_pos.distance_manhattan(*pop_pos) as f32) <= anomaly.radius {
                let roll = rng.gen::<f32>();
                if roll > 0.5 {
                    let current_xp = skills
                        .xp
                        .get(&SkillType::Engineering)
                        .copied()
                        .unwrap_or(0.0);
                    skills.xp.insert(SkillType::Engineering, current_xp + 5.0);
                } else {
                    stress.accumulated_stress += 20.0;
                }
            }
        }
    }
}

pub fn process_temporal_decay_system(
    anomalies: Query<(&ChronoAnomaly, &GridPosition)>,
    mut buildings: Query<(&mut BuildingAge, &GridPosition), With<Building>>,
) {
    for (anomaly, anomaly_pos) in anomalies.iter() {
        for (mut age, building_pos) in buildings.iter_mut() {
            if (anomaly_pos.distance_manhattan(*building_pos) as f32) <= anomaly.radius {
                age.ticks += 1000;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;

    fn setup_world() -> World {
        World::new()
    }

    #[test]
    fn test_pop_enters_anomaly_gains_skill_or_stress() {
        let mut world = setup_world();

        let anomaly_pos = GridPosition { x: 50, y: 50 };
        world.spawn((ChronoAnomaly { radius: 2.0 }, anomaly_pos));

        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 50, y: 50 },
                Skills {
                    xp: std::collections::HashMap::new(),
                },
                StressTracker {
                    accumulated_stress: 0.0,
                },
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(process_temporal_echo_system);
        schedule.run(&mut world);

        let skills = world.get::<Skills>(pop).unwrap();
        let stress = world.get::<StressTracker>(pop).unwrap();

        let engineering_xp = skills
            .xp
            .get(&SkillType::Engineering)
            .copied()
            .unwrap_or(0.0);
        assert!(engineering_xp > 0.0 || stress.accumulated_stress > 0.0);
    }

    #[test]
    fn test_building_in_anomaly_ages() {
        let mut world = setup_world();

        let anomaly_pos = GridPosition { x: 50, y: 50 };
        world.spawn((ChronoAnomaly { radius: 2.0 }, anomaly_pos));

        let building = world
            .spawn((
                Building {
                    building_type: crate::layer1::architecture::building::BuildingType::Housing,
                },
                GridPosition { x: 50, y: 50 },
                BuildingAge { ticks: 100 },
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(process_temporal_decay_system);
        schedule.run(&mut world);

        let age = world.get::<BuildingAge>(building).unwrap();
        assert!(age.ticks > 100);
    }
}

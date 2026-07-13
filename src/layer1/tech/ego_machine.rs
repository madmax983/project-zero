use crate::layer1::map::GridPosition;
use crate::layer1::skills::{SkillType, Skills};
use crate::layer1::social::unrest::Unrest;

#[derive(bevy_ecs::prelude::Component, Debug, Clone, Default)]
pub struct EgoMachine {
    pub active: bool,
}

#[derive(bevy_ecs::prelude::Component, Debug, Clone, Default)]
pub struct EgoStat {
    pub value: f32,
}

pub fn process_ego_machine(
    mut pop_query: bevy_ecs::system::Query<(&mut Skills, &mut EgoStat, &GridPosition)>,
    machine_query: bevy_ecs::system::Query<(&EgoMachine, &GridPosition)>,
) {
    for (machine, machine_pos) in machine_query.iter() {
        if !machine.active {
            continue;
        }

        for (mut skills, mut ego, pop_pos) in pop_query.iter_mut() {
            if machine_pos == pop_pos {
                skills.add_xp(SkillType::Crafting, 0.1);
                ego.value += 0.1;
            }
        }
    }
}

pub fn process_ego_social_friction(
    pop_query: bevy_ecs::system::Query<(&EgoStat, &GridPosition)>,
    mut unrest: bevy_ecs::system::ResMut<Unrest>,
) {
    // To avoid O(N^2) complexity with a simple double loop,
    // we could use a spatial hash, but given the spec, a simple N^2 is fine for the minimal implementation.

    // Collect all high ego pops
    let mut high_ego_pops = Vec::new();
    let mut low_ego_pops = Vec::new();

    for (ego, pos) in pop_query.iter() {
        if ego.value > 50.0 {
            high_ego_pops.push(pos);
        } else if ego.value < 30.0 {
            low_ego_pops.push(pos);
        }
    }

    for high_pos in &high_ego_pops {
        for low_pos in &low_ego_pops {
            if high_pos == low_pos {
                unrest.level += 0.05;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::{App, Update};

    use crate::layer1::execution::general_work::get_status_modifiers;

    #[test]
    fn test_ego_machine_grants_xp_and_ego() {
        let mut app = App::new();
        app.add_systems(Update, process_ego_machine);

        let pos = GridPosition { x: 5, y: 5 };

        app.world_mut().spawn((EgoMachine { active: true }, pos));

        let pop = app
            .world_mut()
            .spawn((
                crate::layer1::skills::Skills::default(),
                EgoStat { value: 0.0 },
                pos,
            ))
            .id();

        app.update();

        let skills = app.world().get::<Skills>(pop).unwrap();
        let ego = app.world().get::<EgoStat>(pop).unwrap();

        assert!(skills.get_xp(SkillType::Crafting) > 0.0, "Should gain XP");
        assert!(ego.value > 0.0, "Pop should gain Ego from Ego Machine");
    }

    #[test]
    fn test_high_ego_increases_unrest_with_low_ego() {
        let mut app = App::new();
        app.add_systems(Update, process_ego_social_friction);

        let pos = GridPosition { x: 5, y: 5 };

        app.world_mut().spawn((EgoStat { value: 80.0 }, pos));
        app.world_mut().spawn((EgoStat { value: 10.0 }, pos));

        app.world_mut()
            .insert_resource(crate::layer1::social::unrest::Unrest {
                level: 0.0,
                modifiers: vec![],
            });

        app.update();

        let unrest = app.world().get_resource::<Unrest>().unwrap();
        assert!(
            unrest.level > 0.0,
            "Unrest should increase due to ego friction"
        );
        assert!(unrest.level > 0.0);
    }

    #[test]
    fn test_high_ego_refuses_menial_jobs() {
        let mut world = bevy_ecs::world::World::new();

        let high_ego_pop = world
            .spawn((
                EgoStat { value: 80.0 },
                crate::layer1::pop::Job {
                    workplace: bevy_ecs::entity::Entity::PLACEHOLDER,
                    job_type: crate::layer1::utility_types::AssignmentType::FarmWorker,
                },
            ))
            .id();

        let low_ego_pop = world
            .spawn((
                EgoStat { value: 10.0 },
                crate::layer1::pop::Job {
                    workplace: bevy_ecs::entity::Entity::PLACEHOLDER,
                    job_type: crate::layer1::utility_types::AssignmentType::FarmWorker,
                },
            ))
            .id();

        let high_ego_mod = get_status_modifiers(&world, high_ego_pop);
        let low_ego_mod = get_status_modifiers(&world, low_ego_pop);

        assert_eq!(high_ego_mod, 0.0, "High ego pops should refuse menial jobs");
        assert_eq!(
            low_ego_mod, 1.0,
            "Low ego pops should accept menial jobs normally"
        );
    }
}

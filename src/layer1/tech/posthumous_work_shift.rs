use crate::layer1::actions::AssignedTo;
use crate::layer1::map::GridPosition;
use crate::layer1::pop::{PopDied, Speed};
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct DigitalGhost;

pub fn posthumous_work_shift_system(
    mut commands: Commands,
    mut events: EventReader<PopDied>,
    query: Query<(&AssignedTo, &GridPosition)>,
) {
    for event in events.read() {
        if let Ok((assigned_to, grid_position)) = query.get(event.entity) {
            commands.spawn((
                DigitalGhost,
                AssignedTo {
                    entity: assigned_to.entity,
                    assignment_type: assigned_to.assignment_type,
                },
                *grid_position,
                Speed::default(),
            ));
        }
    }
}

pub fn digital_ghost_decay_system(
    mut commands: Commands,
    query: Query<(Entity, Option<&AssignedTo>), With<DigitalGhost>>,
) {
    for (entity, assigned_to_opt) in query.iter() {
        if assigned_to_opt.is_none() {
            commands.entity(entity).despawn();
        }
    }
}

pub fn haunted_workplace_system(
    ghosts: Query<&GridPosition, With<DigitalGhost>>,
    mut pops: Query<
        (&GridPosition, &mut crate::layer1::morale::Morale),
        With<crate::layer1::pop::Pop>,
    >,
) {
    for ghost_pos in ghosts.iter() {
        for (pop_pos, mut morale) in pops.iter_mut() {
            let dx = (ghost_pos.x - pop_pos.x).abs();
            let dy = (ghost_pos.y - pop_pos.y).abs();
            if dx <= 2 && dy <= 2 {
                let has_modifier = morale
                    .modifiers
                    .iter()
                    .any(|m| m.label == "Haunted Workplace");
                if !has_modifier {
                    morale.modifiers.push(crate::layer1::morale::MoodModifier {
                        label: "Haunted Workplace".to_string(),
                        value: -0.2,
                        duration: 10,
                    });
                } else {
                    for m in morale.modifiers.iter_mut() {
                        if m.label == "Haunted Workplace" {
                            m.duration = 10;
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::actions::{AssignedTo, AssignmentType};
    use crate::layer1::map::GridPosition;
    use crate::layer1::morale::Morale;
    use crate::layer1::pop::{Pop, PopDied};

    #[test]
    fn test_pop_death_spawns_digital_ghost_during_active_shift() {
        let mut world = World::new();
        let workplace = world.spawn_empty().id();
        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 5, y: 5 },
                AssignedTo {
                    entity: workplace,
                    assignment_type: AssignmentType::FarmWorker,
                },
            ))
            .id();

        world.insert_resource(Events::<PopDied>::default());

        world.send_event(PopDied {
            entity: pop,
            name: "Test Pop".to_string(),
            tick: 1,
            reason: "Mock Death".to_string(),
        });

        let mut schedule = Schedule::default();
        schedule.add_systems(posthumous_work_shift_system);
        schedule.run(&mut world);

        let mut query = world
            .query_filtered::<(&DigitalGhost, &AssignedTo, &GridPosition), With<DigitalGhost>>();
        let ghosts: Vec<_> = query.iter(&world).collect();
        assert_eq!(
            ghosts.len(),
            1,
            "Expected exactly 1 DigitalGhost to be spawned"
        );
        assert_eq!(ghosts[0].1.entity, workplace);
        assert_eq!(ghosts[0].1.assignment_type, AssignmentType::FarmWorker);
        assert_eq!(ghosts[0].2.x, 5);
        assert_eq!(ghosts[0].2.y, 5);
    }

    #[test]
    fn test_digital_ghost_completes_current_shift_then_decays() {
        let mut world = World::new();
        let ghost = world.spawn((DigitalGhost,)).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(digital_ghost_decay_system);
        schedule.run(&mut world);

        assert!(
            world.get_entity(ghost).is_err(),
            "Ghost should have despawned after completing the shift"
        );
    }

    #[test]
    fn test_digital_ghost_reduces_colony_morale_when_working() {
        let mut world = World::new();
        world.spawn((DigitalGhost, GridPosition { x: 10, y: 10 }));

        let living_pop = world
            .spawn((
                Pop,
                Morale::default(),
                GridPosition { x: 10, y: 11 }, // Adjacent
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(haunted_workplace_system);
        schedule.run(&mut world);

        let morale = world.get::<Morale>(living_pop).unwrap();
        assert!(
            morale
                .modifiers
                .iter()
                .any(|m| m.label == "Haunted Workplace"),
            "Living pop should have 'Haunted Workplace' modifier"
        );
    }

    #[test]
    fn test_digital_ghost_cannot_be_assigned_new_tasks() {
        let mut world = World::new();
        let ghost = world.spawn((DigitalGhost,)).id();

        // Adding necessary resources to avoid panic during Utility AI
        world.insert_resource(crate::layer1::utility_types::UtilityConfig::default());
        world.insert_resource(crate::layer1::resources::ColonyResources::default());
        world.insert_resource(crate::layer1::temperature::TemperatureGrid::new(1, 1, 0.0));
        world.insert_resource(crate::layer1::factions::Factions::default());
        world.insert_resource(crate::layer1::zone::ZoneGrid::new(1, 1));
        world.insert_resource(crate::layer1::day_night::DayNightCycle::default());
        world.insert_resource(crate::layer1::taboo::TabooState::default());

        crate::layer1::utility_ai::evaluate_actions_system(&mut world);

        assert!(
            world.get::<AssignedTo>(ghost).is_none(),
            "Idle Ghost should not be assigned new tasks by Utility AI"
        );
    }
}

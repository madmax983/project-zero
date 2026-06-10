#[cfg(test)]
mod integration_tests {
    use bevy_app::prelude::*;
    use bevy_ecs::prelude::*;
    use scale::layer1::social::inherited_grudges::{
        prevent_grudge_work_system, transfer_grudges_on_death_system, Grudge, GrudgeList, Lineage,
    };
    use scale::layer1::pop::PopDied;
    use scale::layer1::actions::{AssignedTo, AssignmentType};

    #[test]
    fn test_inherited_grudges_bridge() {
        let mut app = App::new();
        app.add_event::<PopDied>();

        app.add_systems(
            Update,
            (transfer_grudges_on_death_system, prevent_grudge_work_system).chain(),
        );

        let target = app.world_mut().spawn_empty().id();
        let workplace = app.world_mut().spawn_empty().id();

        app.world_mut().entity_mut(target).insert(AssignedTo {
            entity: workplace,
            assignment_type: AssignmentType::FarmWorker,
        });

        let parent = app
            .world_mut()
            .spawn(GrudgeList(vec![Grudge {
                target_entity: target,
                intensity: 100.0,
                origin_reason: "Feud".to_string(),
            }]))
            .id();

        let child = app
            .world_mut()
            .spawn((
                Lineage {
                    parent_entity: Some(parent),
                },
                GrudgeList(vec![]),
                AssignedTo {
                    entity: workplace,
                    assignment_type: AssignmentType::FarmWorker,
                },
            ))
            .id();

        app.world_mut().send_event(PopDied {
            entity: parent,
            name: "Parent".to_string(),
            tick: 0,
            reason: "Old Age".to_string(),
        });

        app.update();

        // The child should inherit the grudge
        let child_grudges = app.world().get::<GrudgeList>(child).unwrap();
        assert_eq!(child_grudges.0.len(), 1);

        // Since the child inherited the grudge against target, and both are assigned to workplace,
        // the child's assignment should be removed.
        assert!(app.world().get::<AssignedTo>(child).is_none());
    }
}

use crate::layer1::economy::resources::ColonyResources;
use crate::layer1::map::GridPosition;
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct HarvesterMeteor {
    pub stolen_amount: f32,
    pub launch_timer: f32,
    pub has_absorbed: bool,
}

#[derive(Component)]
pub struct ImpactZone;

pub fn harvester_meteor_absorption_system(
    mut meteors: Query<(&mut HarvesterMeteor, &GridPosition)>,
    mut resources: ResMut<ColonyResources>,
) {
    for (mut meteor, _) in meteors.iter_mut() {
        if meteor.has_absorbed {
            continue;
        }

        let mut total_stolen = 0.0;
        let drain = |amount: &mut f32| -> f32 {
            let stolen = *amount;
            *amount = 0.0;
            stolen
        };

        total_stolen += drain(&mut resources.metal);
        total_stolen += drain(&mut resources.ore);
        total_stolen += drain(&mut resources.scrap);
        total_stolen += drain(&mut resources.hyper_alloys);
        total_stolen += drain(&mut resources.hyper_valuable);

        meteor.stolen_amount += total_stolen;
        meteor.has_absorbed = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spawn_test_world() -> World {
        World::new()
    }

    #[test]
    fn test_meteor_absorbs_resources() {
        let mut world = spawn_test_world();

        let meteor_entity = world.spawn_empty().id();
        world.entity_mut(meteor_entity).insert((
            HarvesterMeteor {
                stolen_amount: 0.0,
                launch_timer: 180.0,
                has_absorbed: false,
            },
            GridPosition { x: 5, y: 5 },
        ));

        let resources = ColonyResources {
            metal: 500.0,
            wood: 200.0, // Should not be stolen
            ..Default::default()
        };
        world.insert_resource(resources);

        // Act
        let mut schedule = bevy_ecs::schedule::Schedule::default();
        schedule.add_systems(harvester_meteor_absorption_system);
        schedule.run(&mut world);

        // Assert
        let res = world.resource::<ColonyResources>();
        let meteor = world.get::<HarvesterMeteor>(meteor_entity).unwrap();

        assert_eq!(
            res.metal, 0.0,
            "Colony should be drained of metal by the meteor"
        );
        assert_eq!(
            res.wood, 200.0,
            "Colony should not be drained of wood by the meteor"
        );

        assert_eq!(
            meteor.stolen_amount, 500.0,
            "Meteor should absorb the colony's valuable resources"
        );
        assert!(
            meteor.has_absorbed,
            "Meteor should be marked as having absorbed resources"
        );
    }

    #[test]
    fn test_meteor_does_not_absorb_twice() {
        let mut world = spawn_test_world();

        let meteor_entity = world.spawn_empty().id();
        world.entity_mut(meteor_entity).insert((
            HarvesterMeteor {
                stolen_amount: 0.0,
                launch_timer: 180.0,
                has_absorbed: false,
            },
            GridPosition { x: 5, y: 5 },
        ));

        let resources = ColonyResources {
            metal: 500.0,
            ..Default::default()
        };
        world.insert_resource(resources);

        // Act 1
        let mut schedule = bevy_ecs::schedule::Schedule::default();
        schedule.add_systems(harvester_meteor_absorption_system);
        schedule.run(&mut world);

        // Act 2 (Give more resources, meteor shouldn't absorb again)
        world.resource_mut::<ColonyResources>().metal = 300.0;
        schedule.run(&mut world);

        // Assert
        let res = world.resource::<ColonyResources>();
        let meteor = world.get::<HarvesterMeteor>(meteor_entity).unwrap();

        assert_eq!(
            res.metal, 300.0,
            "Colony should keep the new metal because meteor has already absorbed"
        );
        assert_eq!(
            meteor.stolen_amount, 500.0,
            "Meteor should only have the initially absorbed resources"
        );
    }
}

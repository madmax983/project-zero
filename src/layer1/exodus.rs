use crate::layer1::building::Building;
use crate::layer1::resources::ColonyResources;
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct ArkShipProject {
    pub progress: u32,
    pub target: u32,
}

#[derive(Component)]
pub struct Cannibalizable {
    pub yield_amount: u32,
}

pub fn build_ark_system(
    mut ark_projects: Query<&mut ArkShipProject>,
    mut stockpiles: ResMut<ColonyResources>,
) {
    for mut ark in ark_projects.iter_mut() {
        // Take up to 100 parts per tick
        if stockpiles.scrap >= 100.0 {
            stockpiles.scrap -= 100.0;
            ark.progress += 100;
        } else if stockpiles.scrap >= 1.0 {
            let taken_floor = stockpiles.scrap.floor();
            stockpiles.scrap -= taken_floor;
            ark.progress += taken_floor as u32;
        }
    }
}

pub fn cannibalize_infrastructure_system(
    mut commands: Commands,
    buildings: Query<(Entity, &Cannibalizable), With<Building>>,
    mut stockpiles: ResMut<ColonyResources>,
) {
    for (entity, can) in buildings.iter() {
        stockpiles.scrap += can.yield_amount as f32;
        commands.entity(entity).despawn();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::building::BuildingType;

    #[test]
    fn test_ark_ship_construction_progress() {
        // Arrange
        let mut world = World::new();
        let ark = world
            .spawn(ArkShipProject {
                progress: 0,
                target: 1000,
            })
            .id();
        let res = ColonyResources {
            scrap: 100.0,
            ..Default::default()
        };
        world.insert_resource(res);

        // Act
        let mut schedule = Schedule::default();
        schedule.add_systems(build_ark_system);
        schedule.run(&mut world);

        // Assert
        let progress = world.get::<ArkShipProject>(ark).expect("Component should exist or System should run").progress;
        assert_eq!(
            progress, 100,
            "Ark should have consumed resources and progressed"
        );
        let remaining_resources = world.resource::<ColonyResources>().scrap;
        assert_eq!(remaining_resources, 0.0, "Resources should be consumed");
    }

    #[test]
    fn test_cannibalize_building() {
        // Arrange
        let mut world = World::new();
        let building = world
            .spawn((
                Building {
                    building_type: BuildingType::Housing,
                },
                Cannibalizable { yield_amount: 50 },
            ))
            .id();
        let res = ColonyResources {
            scrap: 0.0,
            ..Default::default()
        };
        world.insert_resource(res);

        // Act
        let mut schedule = Schedule::default();
        schedule.add_systems(cannibalize_infrastructure_system);
        schedule.run(&mut world);

        // Assert
        assert!(
            world.get::<Building>(building).is_none(),
            "Building should be dismantled"
        );
        let remaining_resources = world.resource::<ColonyResources>().scrap;
        assert_eq!(
            remaining_resources, 50.0,
            "Yielded resources should go to stockpile"
        );
    }
}

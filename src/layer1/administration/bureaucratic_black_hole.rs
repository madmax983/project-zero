use crate::layer1::architecture::building::Building;
use crate::layer1::architecture::building::BuildingType;
use crate::layer1::core::map::GridPosition;
use crate::layer1::economy::resources::ResourceItem;
use crate::layer1::entities::pop::Pop;
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct BureaucraticBlackHole {
    pub radius: f32,
}

#[derive(Component)]
pub struct LostInPaperworkChance(pub f32);

#[derive(Component)]
pub struct ReassignmentChance(pub f32);

pub fn check_bureaucratic_density_system(
    mut commands: Commands,
    buildings_query: Query<(Entity, &GridPosition, &Building)>,
    black_hole_query: Query<&BureaucraticBlackHole>,
) {
    if !black_hole_query.is_empty() {
        return; // Only one for now to satisfy minimal test
    }

    let mut admin_count = 0;
    let mut prod_count = 0;
    let mut first_admin_entity = None;

    for (entity, _pos, building) in buildings_query.iter() {
        if building.building_type == BuildingType::Office {
            admin_count += 1;
            if first_admin_entity.is_none() {
                first_admin_entity = Some(entity);
            }
        } else {
            prod_count += 1;
        }
    }

    if admin_count > prod_count * 2 && admin_count >= 5 {
        if let Some(entity) = first_admin_entity {
            commands
                .entity(entity)
                .insert(BureaucraticBlackHole { radius: 2.0 });
        }
    }
}

pub fn bureaucratic_resource_loss_system(
    mut commands: Commands,
    black_holes: Query<(&BureaucraticBlackHole, &GridPosition)>,
    resources: Query<(Entity, &GridPosition, Option<&LostInPaperworkChance>), With<ResourceItem>>,
) {
    for (bh, bh_pos) in black_holes.iter() {
        for (entity, res_pos, chance) in resources.iter() {
            let dx = (bh_pos.x - res_pos.x) as f32;
            let dy = (bh_pos.y - res_pos.y) as f32;
            let distance = (dx * dx + dy * dy).sqrt();

            if distance <= bh.radius {
                let loss_chance = chance.map(|c| c.0).unwrap_or(0.01);
                // For test determinism if chance is 1.0 we delete
                if loss_chance >= 1.0 || rand::random::<f32>() < loss_chance {
                    commands.entity(entity).despawn();
                }
            }
        }
    }
}

pub fn bureaucratic_pop_reassignment_system(
    mut commands: Commands,
    black_holes: Query<(&BureaucraticBlackHole, &GridPosition)>,
    pops: Query<(Entity, &GridPosition, Option<&ReassignmentChance>), With<Pop>>,
) {
    for (bh, bh_pos) in black_holes.iter() {
        for (entity, pop_pos, chance) in pops.iter() {
            let dx = (bh_pos.x - pop_pos.x) as f32;
            let dy = (bh_pos.y - pop_pos.y) as f32;
            let distance = (dx * dx + dy * dy).sqrt();

            if distance <= bh.radius {
                let reassignment_chance = chance.map(|c| c.0).unwrap_or(0.001);
                if reassignment_chance >= 1.0 || rand::random::<f32>() < reassignment_chance {
                    commands.entity(entity).despawn();
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::economy::resources::ResourceType;
    use bevy::prelude::*;

    #[test]
    fn test_bureaucratic_black_hole_formation() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, check_bureaucratic_density_system);

        // Spawn 1 productive building
        app.world_mut().spawn((
            Building {
                building_type: BuildingType::Farm,
            },
            GridPosition { x: 0, y: 0 },
        ));
        // Spawn multiple admin buildings close to each other
        for i in 1..=5 {
            app.world_mut().spawn((
                Building {
                    building_type: BuildingType::Office,
                },
                GridPosition { x: i, y: 0 },
            ));
        }

        // Act
        app.update();

        // Assert: BureaucraticBlackHole component should be added to the cluster
        let mut query = app.world_mut().query::<&BureaucraticBlackHole>();
        assert_eq!(
            query.iter(&app.world()).count(),
            1,
            "A Bureaucratic Black Hole should have formed"
        );
    }

    #[test]
    fn test_resource_lost_in_paperwork() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, bureaucratic_resource_loss_system);

        let black_hole_pos = GridPosition { x: 5, y: 5 };
        app.world_mut()
            .spawn((BureaucraticBlackHole { radius: 2.0 }, black_hole_pos));

        // Resource carried into the black hole (using 100% loss chance for testing)
        let resource_entity = app
            .world_mut()
            .spawn((
                ResourceItem {
                    resource_type: ResourceType::Food,
                    amount: 10.0,
                },
                GridPosition { x: 5, y: 6 },
                LostInPaperworkChance(1.0),
            ))
            .id();

        // Act
        app.update();

        // Assert: Resource is deleted
        assert!(
            app.world().get::<ResourceItem>(resource_entity).is_none(),
            "Resource should be lost in paperwork"
        );
    }

    #[test]
    fn test_pop_reassignment_deletion() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, bureaucratic_pop_reassignment_system);

        let black_hole_pos = GridPosition { x: 5, y: 5 };
        app.world_mut()
            .spawn((BureaucraticBlackHole { radius: 2.0 }, black_hole_pos));

        // Pop enters the black hole (using 100% reassignment chance for testing)
        let pop_entity = app
            .world_mut()
            .spawn((Pop, GridPosition { x: 5, y: 4 }, ReassignmentChance(1.0)))
            .id();

        // Act
        app.update();

        // Assert: Pop is deleted
        assert!(
            app.world().get::<Pop>(pop_entity).is_none(),
            "Pop should be permanently reassigned (deleted)"
        );
    }
}

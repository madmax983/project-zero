use bevy_ecs::prelude::*;

use crate::layer1::architecture::building::{Building, BuildingMap, OccupiedTiles};
use crate::layer1::events::BuildingRemovedEvent;
use crate::layer1::map::GridPosition;
use crate::layer1::morale::{MoodModifier, Morale};
use crate::layer1::pop::Pop;

#[derive(Component)]
pub struct EdibleMaterial {
    pub food_yield: f32,
}

#[derive(Component)]
pub struct Consumed;

pub fn execute_consume(world: &mut World, designation_entity: Entity) -> bool {
    let Some(designation_pos) = world.get::<GridPosition>(designation_entity).copied() else {
        return false;
    };

    let mut target_building = None;
    if let Some(map) = world.get_resource::<BuildingMap>() {
        if let Some(&entity) = map.0.get(&(designation_pos.x, designation_pos.y)) {
            target_building = Some(entity);
        }
    }

    let Some(building_entity) = target_building else {
        return false;
    };

    if world.get::<EdibleMaterial>(building_entity).is_none() {
        return false; // Can only consume edible buildings
    }

    world.entity_mut(building_entity).insert(Consumed);
    world.despawn(designation_entity);

    true
}

#[allow(clippy::too_many_arguments)]
pub fn consume_building_system(
    mut commands: Commands,
    q_edible_buildings: Query<(Entity, &EdibleMaterial), With<Consumed>>,
    mut building_map: Option<ResMut<BuildingMap>>,
    mut occupied_tiles: Option<ResMut<OccupiedTiles>>,
    mut removed_events: EventWriter<BuildingRemovedEvent>,
    q_building: Query<(&Building, &GridPosition)>,
    mut q_morale: Query<&mut Morale, With<Pop>>,
) {
    for (entity, edible) in q_edible_buildings.iter() {
        if let Ok((building, pos)) = q_building.get(entity) {
            removed_events.send(BuildingRemovedEvent {
                entity,
                position: *pos,
                building_type: building.building_type,
            });
            if let Some(map) = building_map.as_deref_mut() {
                map.0.remove(&(pos.x, pos.y));
            }
            if let Some(occupied) = occupied_tiles.as_deref_mut() {
                occupied.0.remove(&(pos.x, pos.y));
            }
        }

        // resources.food += edible.food_yield; // we spawn items instead now
        if let Ok((_b, pos)) = q_building.get(entity) {
            for _ in 0..(edible.food_yield as u32) {
                commands.spawn((
                    crate::layer1::economy::items::Item {
                        item_type: crate::layer1::economy::items::ItemType::Potato,
                    },
                    *pos,
                ));
            }
        }
        commands.entity(entity).despawn();

        for mut morale in q_morale.iter_mut() {
            morale.add_modifier(MoodModifier {
                label: "Ate the architecture".to_string(),
                value: -0.15,
                duration: 500, // arbitrary duration
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::architecture::building::{Building, BuildingType};
    use bevy::prelude::*;

    use crate::layer1::economy::resources::ColonyResources;

    #[test]
    fn test_execute_consume_success() {
        let mut app = App::new();
        app.insert_resource(ColonyResources {
            food: 10.0,
            ..Default::default()
        });

        let building = app
            .world_mut()
            .spawn((
                Building {
                    building_type: BuildingType::Housing,
                },
                GridPosition { x: 5, y: 5 },
                EdibleMaterial { food_yield: 50.0 },
            ))
            .id();

        app.world_mut().insert_resource(BuildingMap({
            let mut map = bevy::utils::HashMap::new();
            map.insert((5, 5), building);
            map
        }));

        let designation = app.world_mut().spawn((GridPosition { x: 5, y: 5 },)).id();

        let success = execute_consume(app.world_mut(), designation);
        assert!(success);
        assert!(app.world().get::<Consumed>(building).is_some());
        assert!(app.world().get_entity(designation).is_err());
    }

    #[test]
    fn test_execute_consume_fail_no_pos() {
        let mut app = App::new();
        let designation = app.world_mut().spawn_empty().id();
        let success = execute_consume(app.world_mut(), designation);
        assert!(!success);
    }

    #[test]
    fn test_execute_consume_fail_no_building() {
        let mut app = App::new();
        let designation = app.world_mut().spawn(GridPosition { x: 5, y: 5 }).id();
        let success = execute_consume(app.world_mut(), designation);
        assert!(!success);
    }

    #[test]
    fn test_execute_consume_fail_not_edible() {
        let mut app = App::new();
        let building = app
            .world_mut()
            .spawn((
                Building {
                    building_type: BuildingType::Housing,
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        app.world_mut().insert_resource(BuildingMap({
            let mut map = bevy::utils::HashMap::new();
            map.insert((5, 5), building);
            map
        }));

        let designation = app.world_mut().spawn((GridPosition { x: 5, y: 5 },)).id();

        let success = execute_consume(app.world_mut(), designation);
        assert!(!success);
    }

    #[test]
    fn test_consume_building_yields_food() {
        let mut app = App::new();
        app.insert_resource(ColonyResources {
            food: 10.0,
            ..Default::default()
        });
        app.add_event::<BuildingRemovedEvent>();
        app.add_systems(Update, consume_building_system);

        let building = app
            .world_mut()
            .spawn((
                Building {
                    building_type: BuildingType::Housing,
                },
                GridPosition { x: 5, y: 5 },
                EdibleMaterial { food_yield: 50.0 },
                Consumed, // Mark it for consumption
            ))
            .id();

        app.update();

        // Building should be despawned
        assert!(app.world().get_entity(building).is_err());

        let mut found_food = false;
        for item in app
            .world_mut()
            .query::<&crate::layer1::economy::items::Item>()
            .iter(app.world())
        {
            if item.item_type == crate::layer1::economy::items::ItemType::Potato {
                found_food = true;
                break;
            }
        }
        assert!(
            found_food,
            "Consuming the building should spawn food items."
        );
    }
}

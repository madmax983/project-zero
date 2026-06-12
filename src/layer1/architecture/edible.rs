use bevy_ecs::prelude::*;

use crate::layer1::architecture::building::{Building, BuildingMap, OccupiedTiles};
use crate::layer1::economy::resources::ColonyResources;
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

pub fn consume_building_system(
    mut commands: Commands,
    q_edible_buildings: Query<(Entity, &EdibleMaterial), With<Consumed>>,
    mut resources: ResMut<ColonyResources>,
    mut building_map: Option<ResMut<BuildingMap>>,
    mut occupied_tiles: Option<ResMut<OccupiedTiles>>,
    mut removed_events: EventWriter<BuildingRemovedEvent>,
    q_building: Query<(&Building, &GridPosition)>,
    mut q_morale: Query<&mut Morale, With<Pop>>,
) {
    for (entity, edible) in q_edible_buildings.iter() {
        resources.food += edible.food_yield;

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
    use crate::layer1::economy::resources::ColonyResources;
    use bevy::prelude::*;

    #[test]
    fn test_consuming_building_yields_food_and_destroys_building() {
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

        // Food should have increased
        assert_eq!(app.world().resource::<ColonyResources>().food, 60.0);
        // Building should be despawned
        assert!(app.world().get_entity(building).is_err());
    }
}

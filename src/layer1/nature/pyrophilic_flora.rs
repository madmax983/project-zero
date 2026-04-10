use crate::layer1::economy::items::{Item, ItemType};
use crate::layer1::map::GridPosition;
use crate::layer1::nature::fire::{Fire, Flammable};
use bevy::utils::HashSet;
use bevy_ecs::prelude::*;

#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub enum Plant {
    PyrophilicFlora,
}

#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub enum GrowthStage {
    Seedling,
    Growing,
    Mature,
}

#[derive(Component, Debug, Clone)]
pub struct Harvestable {
    pub default_yield: u32,
}

#[derive(Event, Debug, Clone)]
pub struct HarvestEvent {
    pub target: Entity,
}

pub fn pyrophilic_ignition_harvest_system(
    mut commands: Commands,
    plants: Query<(Entity, &Plant, &GrowthStage, &GridPosition)>,
    fires: Query<&GridPosition, With<Fire>>,
) {
    let mut fire_positions = HashSet::default();
    for pos in fires.iter() {
        fire_positions.insert(*pos);
    }

    for (entity, plant, stage, plant_pos) in plants.iter() {
        if matches!(plant, Plant::PyrophilicFlora) && *stage == GrowthStage::Mature {
            // If there's a fire on the exact same tile
            if fire_positions.contains(plant_pos) {
                // Destroy plant
                commands.entity(entity).despawn();
                // Spawn yield
                commands.spawn((
                    Item {
                        item_type: ItemType::PyrophilicFruit,
                    },
                    *plant_pos,
                    Flammable::default(),
                ));
                // Add BurntSeeds as well (spec mentions "yield items (BurntSeeds, PyrophilicFruit)")
                commands.spawn((
                    Item {
                        item_type: ItemType::BurntSeeds,
                    },
                    *plant_pos,
                    Flammable::default(),
                ));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_pyrophilic_crop_does_not_drop_yield_on_standard_harvest() {
        let mut world = World::new();
        let plant_entity = world
            .spawn((
                Plant::PyrophilicFlora,
                GrowthStage::Mature,
                Harvestable { default_yield: 0 },
            ))
            .id();

        world.insert_resource(Events::<HarvestEvent>::default());
        let mut events = world.get_resource_mut::<Events<HarvestEvent>>().unwrap();
        events.send(HarvestEvent {
            target: plant_entity,
        });

        // Assert no items spawned
        assert_eq!(world.query::<&Item>().iter(&world).count(), 0);
    }

    #[test]
    fn test_pyrophilic_crop_drops_yield_when_on_fire() {
        let mut world = World::new();
        let pos = GridPosition { x: 10, y: 10 };
        let plant_entity = world
            .spawn((Plant::PyrophilicFlora, GrowthStage::Mature, pos))
            .id();

        world.spawn((Fire::default(), pos)); // Ignite the tile

        world
            .run_system_once(pyrophilic_ignition_harvest_system)
            .unwrap();

        // Assert plant is destroyed/consumed and yield items are spawned at pos
        let mut query = world.query::<(&Item, &GridPosition)>();
        let has_yield = query
            .iter(&world)
            .any(|(item, p)| *p == pos && item.item_type == ItemType::PyrophilicFruit);
        assert!(has_yield);
        assert!(world.get::<Plant>(plant_entity).is_none()); // Plant destroyed
    }
}

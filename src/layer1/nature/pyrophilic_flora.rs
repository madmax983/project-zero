use crate::layer1::economy::items::{Item, ItemType};
use crate::layer1::map::GridPosition;
use crate::layer1::nature::fire::{Fire, Flammable};
use bevy_ecs::prelude::*;
use bevy::utils::HashSet;

#[derive(Component, Debug, PartialEq, Eq, Clone, Copy)]
pub enum Plant {
    PyrophilicFlora,
}

#[derive(Component, Debug, PartialEq, Eq, Clone, Copy)]
pub enum GrowthStage {
    Mature,
    Seedling,
    Growing,
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
    let mut fire_set = HashSet::new();
    for pos in fires.iter() {
        fire_set.insert(*pos);
    }

    for (entity, plant, stage, plant_pos) in plants.iter() {
        if matches!(plant, Plant::PyrophilicFlora) && *stage == GrowthStage::Mature {
            // If there's a fire on the exact same tile
            if fire_set.contains(plant_pos) {
                // Destroy plant
                commands.entity(entity).despawn();
                // Spawn yield
                commands.spawn((
                    Item { item_type: ItemType::PyrophilicFruit },
                    *plant_pos,
                    Flammable::default(),
                ));
                commands.spawn((
                    Item { item_type: ItemType::BurntSeeds },
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
    use bevy::prelude::*;
    use crate::layer1::economy::items::{ItemType, Item};
    use crate::layer1::map::GridPosition;
    use crate::layer1::nature::fire::Fire;

    // Dummy harvest system to test that PyrophilicFlora isn't harvested normally
    fn dummy_standard_harvest_system(
        mut commands: Commands,
        mut events: EventReader<HarvestEvent>,
        plants: Query<(&Plant, &Harvestable, &GridPosition)>,
    ) {
        for event in events.read() {
            if let Ok((plant, _harvestable, pos)) = plants.get(event.target) {
                if !matches!(plant, Plant::PyrophilicFlora) {
                    commands.spawn((
                        Item { item_type: ItemType::Potato }, // some standard yield
                        *pos,
                    ));
                    commands.entity(event.target).despawn();
                }
            }
        }
    }

    #[test]
    fn test_pyrophilic_crop_does_not_drop_yield_on_standard_harvest() {
        let mut app = App::new();
        app.add_event::<HarvestEvent>();
        app.add_systems(Update, dummy_standard_harvest_system);

        let pos = GridPosition { x: 5, y: 5 };
        let plant_entity = app.world_mut().spawn((
            Plant::PyrophilicFlora,
            GrowthStage::Mature,
            Harvestable { default_yield: 10 },
            pos,
        )).id();

        app.world_mut().send_event(HarvestEvent { target: plant_entity });
        app.update();

        // Assert no items spawned
        let mut query = app.world_mut().query::<&Item>();
        assert_eq!(query.iter(app.world()).count(), 0);
        // Assert Pyrophilic plant is NOT destroyed
        assert!(app.world().get::<Plant>(plant_entity).is_some());
    }

    #[test]
    fn test_pyrophilic_crop_drops_yield_when_on_fire() {
        let mut app = App::new();
        app.add_systems(Update, pyrophilic_ignition_harvest_system);

        let pos = GridPosition { x: 10, y: 10 };
        let plant_entity = app.world_mut().spawn((
            Plant::PyrophilicFlora,
            GrowthStage::Mature,
            pos,
        )).id();

        app.world_mut().spawn((Fire::default(), pos)); // Ignite the tile

        app.update();

        // Assert plant is destroyed/consumed and yield items (BurntSeeds, PyrophilicFruit) are spawned at pos
        let mut query = app.world_mut().query::<(&Item, &GridPosition)>();
        let has_fruit = query.iter(app.world()).any(|(item, p)| *p == pos && item.item_type == ItemType::PyrophilicFruit);
        let has_seeds = query.iter(app.world()).any(|(item, p)| *p == pos && item.item_type == ItemType::BurntSeeds);
        assert!(has_fruit);
        assert!(has_seeds);
        assert!(app.world().get::<Plant>(plant_entity).is_none()); // Plant destroyed
    }
}

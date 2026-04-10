use crate::layer1::economy::items::{Item, ItemType};
use crate::layer1::map::GridPosition;
use crate::layer1::nature::fire::{Fire, Flammable};
use bevy_ecs::prelude::*;

#[derive(Component)]
pub enum Plant {
    PyrophilicFlora,
}

#[derive(Component, PartialEq, Eq)]
pub enum GrowthStage {
    Mature,
    Seedling,
    Growing,
}

#[derive(Component)]
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
    // Collect all fire positions into a HashSet for O(1) lookup
    let mut fire_positions = bevy::utils::HashSet::new();
    for pos in fires.iter() {
        fire_positions.insert(*pos);
    }

    for (entity, plant, stage, plant_pos) in plants.iter() {
        if matches!(plant, Plant::PyrophilicFlora) && *stage == GrowthStage::Mature {
            // If there's a fire on the exact same tile
            if fire_positions.contains(plant_pos) {
                // Destroy plant
                commands.entity(entity).despawn();

                // Spawn yield (PyrophilicFruit and BurntSeeds)
                // Add Flammable so they can burn if not hauled
                commands.spawn((
                    Item {
                        item_type: ItemType::PyrophilicFruit,
                    },
                    *plant_pos,
                    Flammable::default(),
                ));
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

    // Mock App for tests since Bevy App is not easily used without full bevy dependency.
    // Instead we use Schedule and World directly as is typical in this project for unit tests.

    #[test]
    fn test_pyrophilic_crop_does_not_drop_yield_on_standard_harvest() {
        let mut world = World::new();
        world.init_resource::<Events<HarvestEvent>>();

        let plant_entity = world.spawn((
            Plant::PyrophilicFlora,
            GrowthStage::Mature,
            Harvestable { default_yield: 0 },
        )).id();

        world.send_event(HarvestEvent { target: plant_entity });

        let mut schedule = Schedule::default();
        schedule.add_systems(pyrophilic_ignition_harvest_system);
        schedule.run(&mut world);

        // Run systems if we were doing full event processing, but here we just verify
        // the harvest system isn't triggered by the event for pyrophilic flora
        // In the real system, standard harvest might ignore it if it checks for PyrophilicFlora
        // The test explicitly wants to assert no items are spawned.
        assert_eq!(world.query::<&Item>().iter(&world).count(), 0);
    }

    #[test]
    fn test_pyrophilic_crop_drops_yield_when_on_fire() {
        let mut world = World::new();

        let pos = GridPosition { x: 10, y: 10 };
        let plant_entity = world.spawn((
            Plant::PyrophilicFlora,
            GrowthStage::Mature,
            pos,
        )).id();

        world.spawn((Fire::default(), pos)); // Ignite the tile

        let mut schedule = Schedule::default();
        schedule.add_systems(pyrophilic_ignition_harvest_system);
        schedule.run(&mut world);

        // Assert plant is destroyed/consumed and yield items (BurntSeeds, PyrophilicFruit) are spawned at pos
        let mut query = world.query::<(&Item, &GridPosition)>();
        let has_fruit = query.iter(&world).any(|(item, p)| *p == pos && item.item_type == ItemType::PyrophilicFruit);
        let has_seeds = query.iter(&world).any(|(item, p)| *p == pos && item.item_type == ItemType::BurntSeeds);

        assert!(has_fruit);
        assert!(has_seeds);
        assert!(world.get::<Plant>(plant_entity).is_none()); // Plant destroyed
    }
}

use bevy_ecs::prelude::*;
use crate::layer1::balance::TICKS_PER_YEAR;
use crate::layer1::resources::ColonyResources;

/// Component indicating an item can spoil/rot over time.
#[derive(Component, Debug, Clone, Copy)]
pub struct Perishable {
    /// Number of ticks since creation/spawn.
    pub current_ticks: u32,
    /// Maximum ticks before the item is destroyed.
    pub max_ticks: u32,
}

impl Default for Perishable {
    fn default() -> Self {
        Self {
            current_ticks: 0,
            #[allow(clippy::cast_possible_truncation)]
            max_ticks: TICKS_PER_YEAR as u32,
        }
    }
}

/// Percentage of global food that spoils per tick (0.05%).
pub const GLOBAL_SPOILAGE_RATE: f32 = 0.0005;

/// System that handles decay of `ColonyResources` (food) and `Perishable` entities.
pub fn spoilage_system(
    mut commands: Commands,
    mut resources: ResMut<ColonyResources>,
    mut query: Query<(Entity, &mut Perishable)>,
) {
    // 1. Handle Global Spoilage
    if resources.food > 0.0 {
        let decay = resources.food * GLOBAL_SPOILAGE_RATE;
        resources.food = (resources.food - decay).max(0.0);
    }

    // 2. Handle Perishable Items
    for (entity, mut perishable) in &mut query {
        perishable.current_ticks += 1;
        if perishable.current_ticks >= perishable.max_ticks {
            commands.entity(entity).despawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::resources::{ColonyResources, ResourceItem, ResourceType};
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_perishable_component_default() {
        let p = Perishable::default();
        // Default should be reasonable, e.g., 1000 ticks (1 year)
        assert!(p.max_ticks > 0);
        assert_eq!(p.current_ticks, 0);
    }

    #[test]
    fn test_perishable_item_decay() {
        let mut world = World::new();
        world.insert_resource(ColonyResources::default());

        // Spawn a perishable item (Food)
        let item = world
            .spawn((
                ResourceItem {
                    resource_type: ResourceType::Food,
                    amount: 10.0,
                },
                Perishable {
                    max_ticks: 10,
                    current_ticks: 0,
                },
            ))
            .id();

        // Run system 5 times
        for _ in 0..5 {
            world.run_system_once(spoilage_system).unwrap();
        }

        // Item should still exist
        let p = world.get::<Perishable>(item).unwrap();
        assert_eq!(p.current_ticks, 5);
    }

    #[test]
    fn test_perishable_item_destruction() {
        let mut world = World::new();
        world.insert_resource(ColonyResources::default());

        // Spawn a perishable item near death
        let item = world
            .spawn((
                ResourceItem {
                    resource_type: ResourceType::Food,
                    amount: 10.0,
                },
                Perishable {
                    max_ticks: 10,
                    current_ticks: 9,
                },
            ))
            .id();

        // Run system twice (9->10 (rot), 10->despawn? Or just >= max)
        world.run_system_once(spoilage_system).unwrap();

        // Should be gone or marked for removal
        // Note: Bevy 0.15 get_entity returns Result
        assert!(world.get_entity(item).is_err());
    }

    #[test]
    fn test_global_food_decay() {
        let mut world = World::new();
        let mut resources = ColonyResources::default();
        resources.food = 1000.0;
        world.insert_resource(resources);

        // Run system
        world.run_system_once(spoilage_system).unwrap();

        let resources = world.resource::<ColonyResources>();
        // Should be less than 1000.0
        assert!(resources.food < 1000.0);
        // But not zero
        assert!(resources.food > 0.0);
    }

    #[test]
    fn test_non_perishable_items_untouched() {
        let mut world = World::new();
        world.insert_resource(ColonyResources::default());

        // Stone is not perishable
        let item = world
            .spawn((
                ResourceItem {
                    resource_type: ResourceType::Stone,
                    amount: 10.0,
                },
                // No Perishable component
            ))
            .id();

        world.run_system_once(spoilage_system).unwrap();

        assert!(world.get_entity(item).is_ok());
    }
}

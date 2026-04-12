use crate::layer1::inventory::Inventory;
use crate::layer1::map::GridPosition;
use crate::layer1::LightMap;
use bevy_ecs::prelude::*;

/// Component for items that decay when exposed to light.
#[derive(Component, Debug, Clone)]
pub struct Photophobic {
    /// HP lost per tick at max light intensity (1.0).
    pub decay_rate: f32,
    /// Current structural integrity of the item.
    pub current_hp: f32,
    /// Maximum structural integrity.
    pub max_hp: f32,
}

/// Component indicating the entity is physically located within another entity (container).
///
/// Used for items inside an [`Inventory`] to track their position relative to the container.
#[derive(Component, Debug, Clone, Copy)]
pub struct Parent(pub Entity);

/// System that applies decay to [`Photophobic`] items based on light levels.
///
/// Handles items on the ground (using [`GridPosition`]) and items in containers (using [`Parent`]).
/// If an item's HP reaches 0, it is destroyed and removed from the container's inventory.
pub fn photophobic_decay_system(
    mut commands: Commands,
    light_map: Res<LightMap>,
    mut query: Query<(
        Entity,
        &mut Photophobic,
        Option<&GridPosition>,
        Option<&Parent>,
    )>,
    mut parents: Query<(&GridPosition, Option<&mut Inventory>), Without<Photophobic>>,
) {
    for (entity, mut photo, pos, parent) in query.iter_mut() {
        let mut target_pos = None;
        let mut parent_inventory = None;

        if let Some(p) = pos {
            target_pos = Some(*p);
        } else if let Some(parent) = parent {
            if let Ok((p_pos, p_inv)) = parents.get_mut(parent.0) {
                target_pos = Some(*p_pos);
                parent_inventory = p_inv;
            }
        }

        if let Some(pos) = target_pos {
            // Cast GridPosition (i32) to u32 for LightMap
            if pos.x >= 0 && pos.y >= 0 {
                let light = light_map.get(pos.x as u32, pos.y as u32);
                if light > 0.0 {
                    let decay = photo.decay_rate * light;
                    photo.current_hp -= decay;

                    if photo.current_hp <= 0.0 {
                        // Cleanup
                        if let Some(mut inv) = parent_inventory {
                            // Remove from inventory
                            // InventoryItem has optional entity field.
                            if let Some(idx) =
                                inv.items.iter().position(|i| i.entity == Some(entity))
                            {
                                inv.items.remove(idx);
                            }
                        }
                        commands.entity(entity).despawn();
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::layer1::environment::photophobic::{photophobic_decay_system, Parent, Photophobic};
    use crate::layer1::inventory::{Inventory, InventoryItem};
    use crate::layer1::items::{Item, ItemType};
    use crate::layer1::map::GridPosition;
    use crate::layer1::LightMap;
    use bevy_ecs::prelude::*;

    #[test]
    fn test_photophobic_component_initialization() {
        let p = Photophobic {
            decay_rate: 10.0,
            current_hp: 100.0,
            max_hp: 100.0,
        };
        assert_eq!(p.current_hp, 100.0);
    }

    #[test]
    fn test_decay_in_light() {
        let mut world = World::new();
        let mut light_map = LightMap::new(10, 10);
        light_map.set(5, 5, 1.0); // Full light
        world.insert_resource(light_map);

        let item = world
            .spawn((
                Item {
                    item_type: ItemType::ShadowCrystal,
                },
                Photophobic {
                    decay_rate: 10.0,
                    current_hp: 100.0,
                    max_hp: 100.0,
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(photophobic_decay_system);
        schedule.run(&mut world);

        let p = world.get::<Photophobic>(item).unwrap();
        assert!(p.current_hp < 100.0, "Should decay in light");
        // Expected: 100 - (1.0 * 10.0) = 90
        assert!((p.current_hp - 90.0).abs() < 0.01);
    }

    #[test]
    fn test_no_decay_in_darkness() {
        let mut world = World::new();
        let mut light_map = LightMap::new(10, 10);
        light_map.set(5, 5, 0.0); // Darkness
        world.insert_resource(light_map);

        let item = world
            .spawn((
                Item {
                    item_type: ItemType::ShadowCrystal,
                },
                Photophobic {
                    decay_rate: 10.0,
                    current_hp: 100.0,
                    max_hp: 100.0,
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(photophobic_decay_system);
        schedule.run(&mut world);

        let p = world.get::<Photophobic>(item).unwrap();
        assert_eq!(p.current_hp, 100.0, "Should not decay in darkness");
    }

    #[test]
    fn test_decay_in_inventory() {
        let mut world = World::new();
        let mut light_map = LightMap::new(10, 10);
        light_map.set(5, 5, 1.0); // Light at storage location
        world.insert_resource(light_map);

        // Storage Building (e.g. Stockpile)
        let storage = world
            .spawn((
                GridPosition { x: 5, y: 5 },
                Inventory::default(), // Needs Inventory for cleanup check, though not strictly required for decay
            ))
            .id();

        // Item inside inventory (child)
        let item = world
            .spawn((
                Item {
                    item_type: ItemType::ShadowCrystal,
                },
                Photophobic {
                    decay_rate: 10.0,
                    current_hp: 100.0,
                    max_hp: 100.0,
                },
                Parent(storage),
            ))
            .id();

        // We also need to add it to Inventory for consistency,
        // though the system checks Parent for position regardless.
        // But the system tries to remove it from Inventory if it dies.
        // For simple decay test, it should work even if not in Inventory struct.

        let mut schedule = Schedule::default();
        schedule.add_systems(photophobic_decay_system);
        schedule.run(&mut world);

        let p = world.get::<Photophobic>(item).unwrap();
        assert!(p.current_hp < 100.0, "Should decay if storage is lit");
    }

    #[test]
    fn test_destruction_at_zero_hp() {
        let mut world = World::new();
        let mut light_map = LightMap::new(10, 10);
        light_map.set(5, 5, 1.0);
        world.insert_resource(light_map);

        let item = world
            .spawn((
                Item {
                    item_type: ItemType::ShadowCrystal,
                },
                Photophobic {
                    decay_rate: 100.0, // Instant kill
                    current_hp: 10.0,
                    max_hp: 100.0,
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(photophobic_decay_system);
        schedule.run(&mut world);

        assert!(world.get_entity(item).is_err(), "Item should be destroyed");
    }

    #[test]
    fn test_destruction_removes_from_inventory() {
        let mut world = World::new();
        let mut light_map = LightMap::new(10, 10);
        light_map.set(5, 5, 1.0);
        world.insert_resource(light_map);

        // Storage
        let storage = world
            .spawn((GridPosition { x: 5, y: 5 }, Inventory::default()))
            .id();

        // Item
        let item = world
            .spawn((
                Item {
                    item_type: ItemType::ShadowCrystal,
                },
                Photophobic {
                    decay_rate: 100.0,
                    current_hp: 10.0,
                    max_hp: 100.0,
                },
                Parent(storage),
            ))
            .id();

        // Add to inventory
        world
            .get_mut::<Inventory>(storage)
            .unwrap()
            .add(InventoryItem {
                item_type: ItemType::ShadowCrystal,
                entity: Some(item),
            });

        let mut schedule = Schedule::default();
        schedule.add_systems(photophobic_decay_system);
        schedule.run(&mut world);

        assert!(world.get_entity(item).is_err(), "Item should be destroyed");

        let inv = world.get::<Inventory>(storage).unwrap();
        assert!(inv.items.is_empty(), "Inventory should be empty");
    }

    #[test]
    fn test_hauling_preserves_photophobic() {
        use crate::layer1::hauling::haul_system;
        use crate::layer1::items::CarryingItem;
        use crate::layer1::pop::Pop;
        use crate::layer1::utility_ai::{ActionType, PopAction};

        let mut world = World::new();
        // Setup Resources needed for hauling
        world.insert_resource(crate::layer1::resources::ColonyResources::default());
        world.insert_resource(crate::layer1::factions::Factions::default());
        world.insert_resource(crate::shared::log::MessageLog::default());
        // Hauling checks ZoneGrid resource optionally
        world.insert_resource(crate::layer1::zone::ZoneGrid::new(10, 10));

        // Pop
        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 0, y: 0 },
                PopAction {
                    current: ActionType::Haul,
                    ..Default::default()
                },
            ))
            .id();

        // Photophobic Item
        let item = world
            .spawn((
                Item {
                    item_type: ItemType::ShadowCrystal,
                },
                Photophobic {
                    decay_rate: 10.0,
                    current_hp: 100.0,
                    max_hp: 100.0,
                },
                GridPosition { x: 0, y: 0 }, // At pop location
            ))
            .id();

        // Stockpile
        let stockpile = world
            .spawn((
                crate::layer1::stockpile::Stockpile::default(),
                Inventory::default(),
                GridPosition { x: 5, y: 0 },
            ))
            .id();

        // 1. Pickup (simulate arrival)
        world
            .entity_mut(pop)
            .insert(crate::layer1::execution::AtTarget);
        // haul_system will try to find item at current pos
        haul_system(&mut world);

        // Verify pickup
        assert!(
            world.get::<CarryingItem>(pop).is_some(),
            "Pop should pick up item"
        );
        assert_eq!(world.get::<CarryingItem>(pop).unwrap().0, item);
        assert!(
            world.get_entity(item).is_ok(),
            "Item should exist while carried"
        );

        // 2. Dropoff
        // Move pop to stockpile
        *world.get_mut::<GridPosition>(pop).unwrap() = GridPosition { x: 5, y: 0 };
        world
            .entity_mut(pop)
            .insert(crate::layer1::execution::AtTarget);

        haul_system(&mut world);

        // Verify dropoff
        // Item entity should STILL exist
        assert!(
            world.get_entity(item).is_ok(),
            "Item should be preserved after dropoff"
        );

        // Item should have Parent(stockpile)
        let parent = world.get::<Parent>(item).expect("Item should have Parent");
        assert_eq!(parent.0, stockpile);

        // Inventory should contain item with entity ref
        let inv = world.get::<Inventory>(stockpile).unwrap();
        assert_eq!(inv.items.len(), 1);
        assert_eq!(inv.items[0].entity, Some(item));
    }
}

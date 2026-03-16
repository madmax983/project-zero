use bevy::prelude::DespawnRecursiveExt;
use bevy_ecs::prelude::*;
use crate::layer1::building::Building;
use crate::layer1::inventory::{Inventory, InventoryItem};
use crate::layer1::items::ItemType;
use crate::layer1::map::GridPosition;
use rand::Rng;

#[derive(Component, Default)]
pub struct Nanoforge {
    pub active_recipe: Option<ItemType>,
    pub breach_risk: f32, // 0.0 to 1.0
}

#[derive(Component)]
pub struct GreyGoo {
    pub replication_progress: f32, // 0.0 to 1.0, 1.0 triggers spread
}

#[derive(Event)]
pub struct ContainmentBreachEvent {
    pub source_entity: Entity,
    pub position: GridPosition,
}

pub fn nanite_fabrication_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Nanoforge, &mut Inventory, Option<&GridPosition>)>,
    mut breach_events: EventWriter<ContainmentBreachEvent>,
    mut chronicle: Option<ResMut<crate::layer1::chronicle::Chronicle>>,
    time: Option<Res<crate::shared::time::SimulationTime>>,
) {
    let mut rng = rand::thread_rng();

    for (entity, mut forge, mut inventory, pos_opt) in query.iter_mut() {
        // Roll for breach
        if rng.gen::<f32>() < forge.breach_risk {
            let pos = pos_opt.copied().unwrap_or(GridPosition { x: 0, y: 0 }); // Fallback for tests
            breach_events.send(ContainmentBreachEvent {
                source_entity: entity,
                position: pos,
            });

            if let Some(chronicle) = chronicle.as_mut() {
                if let Some(time) = time.as_ref() {
                    chronicle.add_event(
                        time.tick,
                        format!("A Nanoforge containment breach occurred at {:?}", pos),
                        crate::layer1::chronicle::EventImportance::Major,
                    );
                }
            }

            commands.entity(entity).despawn_recursive();
            commands.spawn((
                GreyGoo { replication_progress: 0.0 },
                pos,
            ));

            continue;
        }

        // Production risk increase
        forge.breach_risk += 0.001; // Increase risk each tick it runs

        // Instant Production Logic
        if let Some(recipe) = forge.active_recipe {
            let mut none_index = None;
            for (i, item) in inventory.items.iter().enumerate() {
                if matches!(item.item_type, ItemType::None) {
                    none_index = Some(i);
                    break;
                }
            }

            if let Some(index) = none_index {
                inventory.items.remove(index);
                inventory.add(InventoryItem { item_type: recipe, entity: None });
            }
        }
    }
}

pub fn grey_goo_replication_system(
    mut commands: Commands,
    mut goo_query: Query<(Entity, &mut GreyGoo, &GridPosition)>,
    target_query: Query<(Entity, &GridPosition, Option<&Building>)>, // Simplified target finding
) {
    for (_goo_entity, mut goo, goo_pos) in goo_query.iter_mut() {
        if goo.replication_progress >= 1.0 {
            goo.replication_progress = 0.0; // Reset progress after replication

            // Find an adjacent target to consume
            for (target_entity, target_pos, building_opt) in target_query.iter() {
                // Check adjacency (simplistic orthogonal check)
                if (goo_pos.x - target_pos.x).abs() + (goo_pos.y - target_pos.y).abs() == 1
                    && building_opt.is_some() {
                    // Consume!
                    commands.entity(target_entity).despawn_recursive();
                    commands.spawn((
                        GreyGoo { replication_progress: 0.0 },
                        *target_pos,
                    ));
                    break; // Only consume one per tick per goo tile
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::{App, Update};
    use crate::layer1::BuildingType;

    #[test]
    fn test_nanoforge_instant_production() {
        let mut app = App::new();
        app.add_event::<ContainmentBreachEvent>();
        app.insert_resource(crate::layer1::chronicle::Chronicle::default());
        app.insert_resource(crate::shared::time::SimulationTime::default());
        app.add_systems(Update, nanite_fabrication_system);

        // Arrange: Create a nanoforge with input materials
        let mut inventory = Inventory::default();
        use crate::layer1::inventory::InventoryItem;
        for _ in 0..10 {
            inventory.add(InventoryItem { item_type: ItemType::None, entity: None });
        }

        let forge_entity = app.world_mut().spawn((
            Building { building_type: BuildingType::Nanoforge },
            Nanoforge {
                active_recipe: Some(ItemType::Tool),
                breach_risk: 0.0,
                ..Default::default()
            },
            inventory,
        )).id();

        // Act: Run the system
        app.update();

        // Assert: Check that production happened instantly without labor
        let inventory_after = app.world().get::<Inventory>(forge_entity).unwrap();
        assert!(inventory_after.items.iter().any(|item| matches!(item.item_type, ItemType::Tool)));
        assert!(inventory_after.items.iter().filter(|item| matches!(item.item_type, ItemType::None)).count() < 10);
    }

    #[test]
    fn test_nanoforge_containment_breach_event() {
        let mut app = App::new();
        app.add_event::<ContainmentBreachEvent>();
        app.insert_resource(crate::layer1::chronicle::Chronicle::default());
        app.insert_resource(crate::shared::time::SimulationTime::default());
        app.add_systems(Update, nanite_fabrication_system);

        // Arrange: Create a nanoforge with 100% breach risk
        let forge_entity = app.world_mut().spawn((
            Building { building_type: BuildingType::Nanoforge },
            Nanoforge {
                active_recipe: Some(ItemType::Tool),
                breach_risk: 1.0, // Guaranteed breach
                ..Default::default()
            },
            Inventory::default(), // Even with empty inventory, let's say risk check happens
        )).id();

        // Act
        app.update();

        // Assert: Event should be fired
        let events = app.world().resource::<Events<ContainmentBreachEvent>>();
        let mut cursor = events.get_cursor();
        let breach_events: Vec<_> = cursor.read(events).collect();

        assert_eq!(breach_events.len(), 1);
        assert_eq!(breach_events[0].source_entity, forge_entity);
    }

    #[test]
    fn test_grey_goo_replication() {
        let mut app = App::new();
        app.add_systems(Update, grey_goo_replication_system);

        // Arrange: Spawn Grey Goo adjacent to a consumable building
        let pos_goo = GridPosition { x: 10, y: 10 };
        let pos_target = GridPosition { x: 10, y: 11 };

        let target_entity = app.world_mut().spawn((
            Building { building_type: BuildingType::Stockpile },
            pos_target,
        )).id();

        app.world_mut().spawn((
            GreyGoo { replication_progress: 1.0 }, // Ready to replicate
            pos_goo,
        ));

        // Act: Run the replication system
        app.update();

        // Assert: The target building should be destroyed and replaced/spawned as Grey Goo
        assert!(app.world().get_entity(target_entity).is_err() || app.world().get::<Building>(target_entity).is_none());

        // Check if new Grey Goo exists at the target position
        let mut new_goo_found = false;
        for (_goo, pos) in app.world_mut().query::<(&GreyGoo, &GridPosition)>().iter(app.world()) {
            if pos.x == 10 && pos.y == 11 {
                new_goo_found = true;
                break;
            }
        }
        assert!(new_goo_found, "Grey Goo failed to replicate to adjacent tile.");
    }
}

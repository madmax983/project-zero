// src/layer1/tech/nanite_fabrication.rs

use bevy::prelude::*;
use crate::layer1::items::ItemType;
use crate::layer1::map::GridPosition;
use crate::layer1::building::BuildingMap;
use rand::Rng;
use crate::layer1::inventory::Inventory;

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
    mut breach_events: EventWriter<ContainmentBreachEvent>,
    mut query: Query<(Entity, &Nanoforge, &mut Inventory, Option<&GridPosition>)>,
) {
    let mut rng = rand::thread_rng();

    for (entity, forge, mut inventory, pos_opt) in query.iter_mut() {
        // Roll for breach
        if rng.gen::<f32>() < forge.breach_risk {
            let pos = pos_opt.copied().unwrap_or(GridPosition { x: 0, y: 0 }); // Fallback for tests
            breach_events.send(ContainmentBreachEvent {
                source_entity: entity,
                position: pos,
            });

            // Optionally, transform forge itself into Grey Goo here or let another system handle it
            continue;
        }

        // Instant Production Logic
        if let Some(recipe) = forge.active_recipe {
            // Simplified recipe check: 10 Energy + 5 RawMass = 1 item
            // In full implementation, link to a proper Recipe system
            let energy_needed = 10;
            let mass_needed = 5;

            if inventory.items.iter().filter(|i| i.item_type == ItemType::Tool).count() >= energy_needed &&
               inventory.items.iter().filter(|i| i.item_type == ItemType::Clothing).count() >= mass_needed {

                for _ in 0..energy_needed { if let Some(idx) = inventory.items.iter().position(|i| i.item_type == ItemType::Tool) { inventory.items.remove(idx); } }
                for _ in 0..mass_needed { if let Some(idx) = inventory.items.iter().position(|i| i.item_type == ItemType::Clothing) { inventory.items.remove(idx); } }
                use crate::layer1::inventory::InventoryItem;
                inventory.add(InventoryItem { item_type: recipe, entity: None });
            }
        }
    }
}

pub fn handle_containment_breach_system(
    mut commands: Commands,
    mut events: EventReader<ContainmentBreachEvent>,
) {
    for event in events.read() {
        if let Some(entity_cmd) = commands.get_entity(event.source_entity) {
            entity_cmd.despawn_recursive();
        }
        commands.spawn((
            GreyGoo { replication_progress: 0.0 },
            event.position,
        ));
    }
}

pub fn increment_grey_goo_progress_system(
    mut query: Query<&mut GreyGoo>,
) {
    for mut goo in query.iter_mut() {
        goo.replication_progress += 0.05; // Arbitrary speed
    }
}

pub fn grey_goo_replication_system(
    mut commands: Commands,
    mut goo_query: Query<(Entity, &mut GreyGoo, &GridPosition)>,
    building_map: Res<BuildingMap>,
) {
    for (_goo_entity, mut goo, goo_pos) in goo_query.iter_mut() {
        if goo.replication_progress >= 1.0 {
            goo.replication_progress = 0.0; // Reset progress after replication

            // Directions to check: up, down, left, right
            let directions = [
                (0, 1), (0, -1), (1, 0), (-1, 0)
            ];

            for (dx, dy) in directions.iter() {
                let target_x = goo_pos.x + dx;
                let target_y = goo_pos.y + dy;

                if let Some(&target_entity) = building_map.0.get(&(target_x, target_y)) {
                    // We found a building!
                    commands.entity(target_entity).despawn_recursive();
                    commands.spawn((
                        GreyGoo { replication_progress: 0.0 },
                        GridPosition { x: target_x, y: target_y },
                    ));
                    break; // Only consume one per tick per goo tile
                }
            }
        }
    }
}

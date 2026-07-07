use crate::layer1::items::ItemType as Item;
use crate::layer1::map::GridPosition as GridPos;

use crate::layer1::spoilage::Perishable as Spoilage;
use crate::layer1::terrain::{TerrainGrid as GridMap, TerrainType};
use bevy_ecs::prelude::*;
use rand::Rng;

#[derive(Event)]
pub struct OrbitalDropEvent {
    pub target: GridPos,
    pub items: Vec<Item>,
    pub scatter_radius: i32,
}

#[derive(Component)]
pub struct DroppedCrate {
    pub item: Item,
}

pub fn process_orbital_drops(
    mut events: EventReader<OrbitalDropEvent>,
    mut commands: Commands,
    mut grid: ResMut<GridMap>,
) {
    let mut rng = rand::thread_rng();

    for drop in events.read() {
        for item in &drop.items {
            // Apply simple random scatter
            let dx = rng.gen_range(-drop.scatter_radius..=drop.scatter_radius);
            let dy = rng.gen_range(-drop.scatter_radius..=drop.scatter_radius);

            let final_x = (drop.target.x + dx).clamp(0, grid.width as i32 - 1);
            let final_y = (drop.target.y + dy).clamp(0, grid.height as i32 - 1);
            let final_pos = GridPos {
                x: final_x,
                y: final_y,
            };

            if final_x < 0 || final_y < 0 {
                continue;
            }

            // Ensure items do not drop onto impenetrable terrain like solid rock walls
            let terrain = if final_x >= 0 && final_y >= 0 {
                grid.get(final_x as usize, final_y as usize)
            } else {
                None
            }
            .unwrap_or(TerrainType::Grass);
            if matches!(terrain, TerrainType::Rock) {
                continue; // Cannot drop on solid rock walls, skip
            }

            let mut entity = commands.spawn((
                DroppedCrate { item: *item },
                final_pos,
                crate::layer1::items::Item { item_type: *item },
            ));

            // Apply spoilage if perishable
            if matches!(
                item,
                Item::Potato
                    | Item::Wheat
                    | Item::Rice
                    | Item::Corn
                    | Item::Soy
                    | Item::Meat
                    | Item::Fish
                    | Item::Fruit
                    | Item::LuxuryMeal
                    | Item::AlienMeatA
                    | Item::AlienMeatB
                    | Item::GlowMushroom
                    | Item::MysteryMeal
            ) {
                entity.insert(Spoilage {
                    max_ticks: 100,
                    current_ticks: 0,
                }); // Use proper initialization
            }

            // Damage terrain if item is heavy (assuming heavy is Scrap metal)
            if matches!(item, Item::Scrap) && final_x >= 0 && final_y >= 0 {
                grid.set(final_x as usize, final_y as usize, TerrainType::Dirt);
                // Assuming Dirt is closest to Crater we have right now
            }
        }
    }
}

/// Bridges OrbitalDropEvent (Logistics) to crate::layer1::core::chronicle::AddChronicleEvent (Chronicle).
pub fn orbital_drop_chronicle_bridge(
    mut drop_events: EventReader<OrbitalDropEvent>,
    mut chronicle_events: EventWriter<crate::layer1::core::chronicle::AddChronicleEvent>,
) {
    for event in drop_events.read() {
        chronicle_events.send(crate::layer1::core::chronicle::AddChronicleEvent {
            text: format!(
                "Orbital Drop at ({}, {}): {} items scattered within {} tiles.",
                event.target.x,
                event.target.y,
                event.items.len(),
                event.scatter_radius
            ),
            importance: crate::layer1::core::chronicle::EventImportance::Major,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::items::ItemType as Item;
    use crate::layer1::map::GridPosition as GridPos;
    use crate::layer1::spoilage::Perishable as Spoilage;
    use crate::layer1::terrain::TerrainGrid as GridMap;
    use bevy::prelude::*;

    #[test]
    fn test_orbital_drop_spawns_scattered_items() {
        let mut app = App::new();
        app.add_event::<OrbitalDropEvent>();
        app.add_systems(Update, process_orbital_drops);
        app.world_mut().insert_resource(GridMap {
            width: 50,
            height: 50,
            tiles: vec![TerrainType::Grass; 2500],
        });

        let target_pos = GridPos { x: 25, y: 25 };
        let drop_event = OrbitalDropEvent {
            target: target_pos,
            items: vec![Item::Potato, Item::Scrap],
            scatter_radius: 5,
        };

        app.world_mut().send_event(drop_event);
        app.update();

        let dropped_items = app
            .world_mut()
            .query::<(&DroppedCrate, &GridPos)>()
            .iter(app.world())
            .count();
        assert_eq!(dropped_items, 2, "Both items should spawn");

        for (_item, pos) in app
            .world_mut()
            .query::<(&DroppedCrate, &GridPos)>()
            .iter(app.world())
        {
            let distance_x = pos.x.abs_diff(target_pos.x).min(i32::MAX as u32) as i32;
            let distance_y = pos.y.abs_diff(target_pos.y).min(i32::MAX as u32) as i32;
            assert!(
                distance_x <= 5 && distance_y <= 5,
                "Items must land within scatter radius"
            );
        }
    }

    #[test]
    fn test_dropped_items_have_entropy() {
        let mut app = App::new();
        app.add_event::<OrbitalDropEvent>();
        app.add_systems(Update, process_orbital_drops);
        app.world_mut().insert_resource(GridMap {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Grass; 100],
        });

        let drop_event = OrbitalDropEvent {
            target: GridPos { x: 5, y: 5 },
            items: vec![Item::Potato],
            scatter_radius: 1,
        };

        app.world_mut().send_event(drop_event);
        app.update();

        let has_spoilage = app
            .world_mut()
            .query::<&Spoilage>()
            .iter(app.world())
            .count()
            > 0;
        assert!(
            has_spoilage,
            "Dropped perishable items must start degrading"
        );
    }

    #[test]
    fn test_drop_damages_terrain() {
        let mut app = App::new();
        app.add_event::<OrbitalDropEvent>();
        app.add_systems(Update, process_orbital_drops);
        let mut grid = GridMap {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Grass; 100],
        };
        let target_pos = GridPos { x: 5, y: 5 };
        if target_pos.x >= 0 && target_pos.y >= 0 {
            grid.set(
                target_pos.x as usize,
                target_pos.y as usize,
                TerrainType::Grass,
            );
        }
        app.world_mut().insert_resource(grid);

        let drop_event = OrbitalDropEvent {
            target: target_pos,
            items: vec![Item::Scrap], // Heavy drop
            scatter_radius: 0,        // Direct hit
        };

        app.world_mut().send_event(drop_event);
        app.update();

        let grid = app.world().resource::<GridMap>();
        assert!(
            matches!(
                if target_pos.x >= 0 && target_pos.y >= 0 {
                    grid.get(target_pos.x as usize, target_pos.y as usize)
                } else {
                    None
                }
                .unwrap(),
                TerrainType::Dirt
            ),
            "Heavy drop should crater the terrain"
        );
    }
}

use crate::layer1::inventory::Inventory;
use crate::layer1::items::{Item, ItemType};
use crate::layer1::map::GridPosition;
use crate::layer1::temperature::TemperatureGrid;
use crate::shared::time::SimulationTime;
use bevy_ecs::prelude::*;
use std::collections::HashMap;

/// Component for stones that have geodetic sentience.
#[derive(Component, Default)]
pub struct LivingStone {
    /// The last tick this stone moved.
    pub last_move_tick: u64,
}

/// Component for the Stone Golem entity formed by fusing Living Stones.
#[derive(Component)]
pub struct StoneGolem {
    /// Current health points.
    pub hp: i32,
    /// Maximum health points.
    pub max_hp: i32,
}

/// Event emitted when a Stone Golem is formed.
#[derive(Event, Debug, Clone)]
pub struct GolemFormedEvent {
    /// The location where the Golem was formed.
    pub position: GridPosition,
}

/// The number of Living Stones required to form a Golem.
pub const GOLEM_THRESHOLD: usize = 5;
/// The interval (in ticks) between Living Stone movements.
pub const MOVE_INTERVAL: u64 = 100; // Ticks

/// System that updates `LivingStone` positions.
///
/// Stones migrate towards heat sources and each other.
type StonePosQuery<'w, 's> =
    Query<'w, 's, (Entity, &'static GridPosition), (With<LivingStone>, With<Item>)>;
type MutStoneQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static mut GridPosition,
        &'static mut LivingStone,
        &'static Item,
    ),
>;

pub fn update_living_stone_system(
    _commands: Commands,
    time: Res<SimulationTime>,
    mut queries: ParamSet<(StonePosQuery, MutStoneQuery)>,
    temp_grid: Res<TemperatureGrid>,
) {
    // 1. Collect other stone positions to avoid borrow issues
    let stone_positions: Vec<(Entity, GridPosition)> =
        queries.p0().iter().map(|(e, p)| (e, *p)).collect();

    // 2. Iterate and Update
    for (entity, mut pos, mut living, item) in queries.p1().iter_mut() {
        if time.tick < living.last_move_tick + MOVE_INTERVAL {
            continue;
        }

        if item.item_type != ItemType::LivingStone {
            continue;
        }

        living.last_move_tick = time.tick;

        // Logic: Find nearest other stone or heat
        let current_temp = temp_grid.get(pos.x as usize, pos.y as usize);

        let mut best_move = None;
        let mut best_score = -1000.0; // Higher is better

        // Check neighbors
        for dx in -1..=1 {
            for dy in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let nx = pos.x + dx;
                let ny = pos.y + dy;

                if nx < 0 || ny < 0 {
                    continue;
                } // Basic bounds check, assuming infinite + for now or clamped by grid

                // Score based on Heat
                let t = temp_grid.get(nx as usize, ny as usize);
                let heat_score = if t > current_temp {
                    t - current_temp
                } else {
                    0.0
                };

                // Score based on Attraction (Distance to nearest other stone)
                let mut min_dist = 1000.0;
                for (other_e, other_pos) in &stone_positions {
                    if *other_e == entity {
                        continue;
                    }
                    let dist = ((nx - other_pos.x).abs() + (ny - other_pos.y).abs()) as f32;
                    if dist < min_dist {
                        min_dist = dist;
                    }
                }

                // Attraction score: closer is better.
                // If min_dist is 0 (on top of another stone), score is high.
                // We want to minimize distance.
                // Let's arbitrarily weight heat vs attraction.
                // Spec says "migrate towards each other OR heat sources".
                // Let's normalize. Heat diff can be 0-100+. Dist can be 0-100+.
                // Attraction score = -min_dist.

                let attraction_score = if min_dist >= 1000.0 {
                    0.0
                } else {
                    -min_dist * 2.0
                }; // Weight attraction heavily

                let total_score = heat_score + attraction_score;

                if total_score > best_score {
                    best_score = total_score;
                    best_move = Some((nx, ny));
                }
            }
        }

        // If current position (dx=0, dy=0) is better than any neighbor, we stay.
        // Current score calculation:
        {
            let mut min_dist = 1000.0;
            for (other_e, other_pos) in &stone_positions {
                if *other_e == entity {
                    continue;
                }
                let dist = ((pos.x - other_pos.x).abs() + (pos.y - other_pos.y).abs()) as f32;
                if dist < min_dist {
                    min_dist = dist;
                }
            }
            // Heat diff is 0 because t == current_temp
            let current_heat_score = 0.0;

            // Fix: If there are no other stones, min_dist is 1000.0. In the neighbor loop it would also be 1000.0.
            // If min_dist is 1000.0, we shouldn't heavily penalize.
            let attraction_score = if min_dist >= 1000.0 {
                0.0
            } else {
                -min_dist * 2.0
            };

            let current_score = current_heat_score + attraction_score;
            if current_score >= best_score {
                best_move = None;
            }
        }

        if let Some((nx, ny)) = best_move {
            pos.x = nx;
            pos.y = ny;
        }
    }
}

/// System that checks for Golem formation conditions.
///
/// If enough Living Stones gather at the same location (on ground or in inventory),
/// they fuse into a `StoneGolem`.
pub fn form_golem_system(
    mut commands: Commands,
    query: Query<(Entity, &GridPosition, &Item), With<LivingStone>>,
    mut inventories: Query<(Entity, &GridPosition, &mut Inventory)>,
    mut log: Option<ResMut<crate::shared::log::MessageLog>>,
    mut golem_events: EventWriter<GolemFormedEvent>,
) {
    // Map of Position -> Count of Stones
    // We need to track entities to despawn them if they are on ground
    // For inventory items, we track (InventoryEntity, Index)

    struct StoneLocation {
        ground_entities: Vec<Entity>,
        inventory_entries: Vec<(Entity, usize)>, // (InventoryEntity, Index in items vec)
    }

    let mut map: HashMap<(i32, i32), StoneLocation> = HashMap::new();

    // 1. Check Ground Items
    for (e, pos, item) in query.iter() {
        if item.item_type == ItemType::LivingStone {
            map.entry((pos.x, pos.y))
                .or_insert(StoneLocation {
                    ground_entities: Vec::new(),
                    inventory_entries: Vec::new(),
                })
                .ground_entities
                .push(e);
        }
    }

    // 2. Check Inventories
    for (inv_entity, pos, inventory) in inventories.iter() {
        for (i, item) in inventory.items.iter().enumerate() {
            if item.item_type == ItemType::LivingStone {
                map.entry((pos.x, pos.y))
                    .or_insert(StoneLocation {
                        ground_entities: Vec::new(),
                        inventory_entries: Vec::new(),
                    })
                    .inventory_entries
                    .push((inv_entity, i));
            }
        }
    }

    // 3. Form Golems
    for ((x, y), loc) in map {
        let total_count = loc.ground_entities.len() + loc.inventory_entries.len();

        if total_count >= GOLEM_THRESHOLD {
            // Consume Ground Entities
            for e in loc.ground_entities {
                commands.entity(e).despawn();
            }

            // Consume Inventory Items
            // We need to be careful about indices shifting if we remove multiple from same inventory.
            // Best approach: Group by inventory entity, sort indices descending, remove.
            let mut inv_removals: HashMap<Entity, Vec<usize>> = HashMap::new();
            for (inv_e, idx) in loc.inventory_entries {
                inv_removals.entry(inv_e).or_default().push(idx);
            }

            for (inv_e, mut indices) in inv_removals {
                indices.sort_unstable_by(|a, b| b.cmp(a)); // Descending
                if let Ok((_, _, mut inventory)) = inventories.get_mut(inv_e) {
                    for idx in indices {
                        if idx < inventory.items.len() {
                            inventory.items.remove(idx);
                        }
                    }
                }
            }

            // Spawn Golem
            use crate::layer1::fauna::{Fauna, FaunaState, FaunaType};
            use crate::layer1::health::Health;

            commands.spawn((
                StoneGolem {
                    hp: 100,
                    max_hp: 100,
                },
                GridPosition { x, y },
                Fauna {
                    fauna_type: FaunaType::Wolf, // Placeholder, maybe add StoneGolem type to FaunaType later
                    state: FaunaState::Wander,
                    detection_range: 10.0,
                    attack_cooldown: 0,
                    target: None,
                },
                Health {
                    current: 100.0,
                    max: 100.0,
                    has_rust_lung: false,
                },
                // Visuals
                crate::layer1::particles::Particle {
                    char: 'G',
                    color: ratatui::style::Color::Gray,
                    lifetime: 0, // Persistent
                },
            ));

            golem_events.send(GolemFormedEvent {
                position: GridPosition { x, y },
            });

            if let Some(log) = log.as_mut() {
                log.add("The stones vibrate and fuse... A Golem is born!");
            }
        }
    }
}

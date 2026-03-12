use crate::layer1::lighting::LightMap;
use crate::layer1::map::GridPosition;
use crate::layer1::trade::{Merchant, TradeDeal};
use bevy_ecs::prelude::*;

use crate::layer1::resources::ResourceType;
use crate::layer1::inventory::Inventory;
use crate::layer1::items::ItemType;
use crate::layer1::memory_core::MemoryCoreData;
use crate::layer1::skills::Skills;
use crate::layer1::terrain::TerrainGrid;
use crate::shared::time::SimulationTime;
use rand::Rng;

#[derive(Component)]
pub struct ShadowTrader {
    pub merchant: Merchant,
}

/// System to execute a trade for an item (like Memory Core) specifically from a Shadow Trader
pub fn execute_shadow_item_trade(
    world: &mut World,
    trader_entity: Entity,
    buyer_entity: Entity,
    cost_resource: ResourceType,
    cost_amount: f32,
    item_to_buy: ItemType,
) -> bool {
    let mut resources = world.resource_mut::<crate::layer1::resources::ColonyResources>();

    if !resources.can_afford(&crate::layer1::resources::ColonyResources {
        food: if cost_resource == ResourceType::Food { cost_amount } else { 0.0 },
        wood: if cost_resource == ResourceType::Wood { cost_amount } else { 0.0 },
        stone: if cost_resource == ResourceType::Stone { cost_amount } else { 0.0 },
        metal: if cost_resource == ResourceType::Metal { cost_amount } else { 0.0 },
        ore: if cost_resource == ResourceType::Ore { cost_amount } else { 0.0 },
        planks: if cost_resource == ResourceType::Planks { cost_amount } else { 0.0 },
        blocks: if cost_resource == ResourceType::Blocks { cost_amount } else { 0.0 },
        waste: if cost_resource == ResourceType::Waste { cost_amount } else { 0.0 },
        rations: if cost_resource == ResourceType::Rations { cost_amount } else { 0.0 },
        fuel: if cost_resource == ResourceType::Fuel { cost_amount } else { 0.0 },
        alcohol: if cost_resource == ResourceType::Alcohol { cost_amount } else { 0.0 },
        scrap: if cost_resource == ResourceType::Scrap { cost_amount } else { 0.0 },
        tools: if cost_resource == ResourceType::Tools { cost_amount } else { 0.0 },
        building_permits: if cost_resource == ResourceType::BuildingPermit { cost_amount } else { 0.0 },
        credits: 0.0,
        ..crate::layer1::resources::ColonyResources::zeroed()
    }) {
        return false;
    }

    // Deduct
    resources.consume(cost_resource, cost_amount);

    // Transfer item
    let mut trader_inv_opt = None;
    if let Some(inv) = world.get::<Inventory>(trader_entity) {
        trader_inv_opt = Some(inv.clone());
    }

    if let Some(mut trader_inv) = trader_inv_opt {
        if let Some(idx) = trader_inv.items.iter().position(|i| i.item_type == item_to_buy) {
            let item = trader_inv.items.remove(idx);

            // Re-apply modified inventory
            world.entity_mut(trader_entity).insert(trader_inv);

            if let Some(mut buyer_inv) = world.get_mut::<Inventory>(buyer_entity) {
                buyer_inv.add(item);
                return true;
            }
        }
    }

    false
}

pub fn can_spawn_trader(world: &World, pos: GridPosition) -> bool {
    if pos.x < 0 || pos.y < 0 {
        return false;
    }
    let x = pos.x as u32;
    let y = pos.y as u32;

    // Must be in darkness
    if let Some(light_map) = world.get_resource::<LightMap>() {
        if light_map.get(x, y) > 0.1 {
            return false;
        }
    } else {
        return false;
    }

    // Must be pathable (walkable)
    if let Some(terrain) = world.get_resource::<TerrainGrid>() {
        if let Some(tile) = terrain.get(x as usize, y as usize) {
            if !tile.is_walkable() {
                return false;
            }
        } else {
            return false;
        }
    } else {
        return false;
    }

    true
}

pub fn check_spawn_conditions(
    pos: GridPosition,
    light_map: &LightMap,
    terrain: &TerrainGrid,
) -> bool {
    if pos.x < 0 || pos.y < 0 {
        return false;
    }
    let x = pos.x as u32;
    let y = pos.y as u32;

    if light_map.get(x, y) > 0.1 {
        return false;
    }

    if let Some(tile) = terrain.get(x as usize, y as usize) {
        if !tile.is_walkable() {
            return false;
        }
    } else {
        return false;
    }

    true
}

/// A resource to track the cooldown between shadow trader spawns
#[derive(Resource, Default)]
pub struct ShadowMarketManager {
    pub cooldown: u64,
}

pub fn spawn_shadow_trader_system(
    mut commands: Commands,
    mut manager: ResMut<ShadowMarketManager>,
    time: Res<SimulationTime>,
    light_map: Res<LightMap>,
    terrain: Res<TerrainGrid>,
    query: Query<&ShadowTrader>,
) {
    // Only one trader at a time
    if !query.is_empty() {
        return;
    }

    if time.tick < manager.cooldown {
        return;
    }

    // Try to find a dark spot
    let mut rng = rand::thread_rng();

    // Fast probabilistic search
    for _ in 0..50 {
        let rx = rng.gen_range(0..light_map.width);
        let ry = rng.gen_range(0..light_map.height);

        let pos = GridPosition {
            x: rx as i32,
            y: ry as i32,
        };
        if check_spawn_conditions(pos, &light_map, &terrain) {
            let deals = vec![TradeDeal {
                cost_resource: ResourceType::Food,
                cost_amount: 10.0,
                give_resource: ResourceType::Alcohol, // Contraband
                give_amount: 5.0,
            }];

            let mut trader_inv = Inventory::default();

            if rng.gen_bool(0.2) {
                // 20% chance to stock a Memory Core (as a physical item)
                let core_data = commands.spawn(MemoryCoreData {
                    skills: Skills::default(), // MVP: generic skills for spawned core
                    traits: vec![],
                }).id();

                trader_inv.add(crate::layer1::inventory::InventoryItem {
                    item_type: ItemType::MemoryCore(core_data),
                    entity: Some(core_data),
                });
            }

            commands.spawn((
                ShadowTrader {
                    merchant: Merchant {
                        name: "Shadow Trader".to_string(),
                        arrival_tick: time.tick,
                        departure_tick: time.tick + 500,
                        deals,
                    },
                },
                trader_inv,
                pos,
            ));

            manager.cooldown = time.tick + 1000;
            return;
        }
    }
}

pub fn despawn_in_light_system(
    mut commands: Commands,
    query: Query<(Entity, &GridPosition), With<ShadowTrader>>,
    light_map: Res<LightMap>,
) {
    for (entity, pos) in query.iter() {
        if pos.x < 0 || pos.y < 0 {
            continue; // Skip out-of-bounds negative coordinates
        }

        let x = pos.x as u32;
        let y = pos.y as u32;
        if light_map.get(x, y) > 0.1 {
            // "The shadows flee!"
            commands.entity(entity).despawn();
            // TODO: Emit visual smoke effect
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::terrain::TerrainType;

    #[test]
    fn test_spawn_only_in_darkness() {
        let mut world = World::new();
        let mut light_map = LightMap::new(10, 10);
        let mut terrain = TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Grass; 100],
        };

        // (5,5) is Lit
        light_map.set(5, 5, 1.0);
        // (0,0) is Dark
        light_map.set(0, 0, 0.0);
        // (1,1) is Dark but Unwalkable
        light_map.set(1, 1, 0.0);
        terrain.set(1, 1, TerrainType::Rock);

        world.insert_resource(light_map);
        world.insert_resource(terrain);

        assert!(
            !can_spawn_trader(&world, GridPosition { x: 5, y: 5 }),
            "Should not spawn in light"
        );
        assert!(
            can_spawn_trader(&world, GridPosition { x: 0, y: 0 }),
            "Should spawn in darkness on grass"
        );
        assert!(
            !can_spawn_trader(&world, GridPosition { x: 1, y: 1 }),
            "Should not spawn on unwalkable tile"
        );
    }

    #[test]
    fn test_despawn_when_lit() {
        let mut world = World::new();
        let light_map = LightMap::new(10, 10);
        world.insert_resource(light_map);

        // Spawn trader in dark
        let trader = world
            .spawn((
                ShadowTrader {
                    merchant: Merchant {
                        name: "".to_string(),
                        arrival_tick: 0,
                        departure_tick: 0,
                        deals: vec![],
                    },
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        // 1. Verify safe in dark
        let mut schedule = Schedule::default();
        schedule.add_systems(despawn_in_light_system);
        schedule.run(&mut world);
        assert!(world.get_entity(trader).is_ok());

        // 2. Turn on lights
        let mut light_map = world.resource_mut::<LightMap>();
        light_map.set(5, 5, 1.0);

        // 3. Run system
        schedule.run(&mut world);

        // 4. Verify despawn
        assert!(
            world.get_entity(trader).is_err(),
            "Trader should flee light"
        );
    }

    #[test]
    fn test_spawn_shadow_trader_system() {
        let mut world = World::new();
        let mut light_map = LightMap::new(10, 10);
        // Map is fully dark
        light_map.tiles.fill(0.0);
        let terrain = TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Grass; 100],
        };

        world.insert_resource(light_map);
        world.insert_resource(terrain);
        world.insert_resource(SimulationTime {
            tick: 100,
            ..Default::default()
        });
        world.insert_resource(ShadowMarketManager { cooldown: 0 });

        let mut schedule = Schedule::default();
        schedule.add_systems(spawn_shadow_trader_system);

        // Ensure no trader yet
        assert_eq!(world.query::<&ShadowTrader>().iter(&world).count(), 0);

        // Run system to spawn
        schedule.run(&mut world);

        // A trader should have been spawned
        let traders: Vec<_> = world.query::<&ShadowTrader>().iter(&world).collect();
        assert_eq!(traders.len(), 1);

        // Check cooldown was set
        assert!(world.resource::<ShadowMarketManager>().cooldown > 100);

        // Run again, should not spawn another due to cooldown/presence
        schedule.run(&mut world);
        let traders_after: Vec<_> = world.query::<&ShadowTrader>().iter(&world).collect();
        assert_eq!(traders_after.len(), 1);
    }
}

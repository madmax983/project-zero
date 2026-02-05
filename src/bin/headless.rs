//! Headless game runner with semantic input/output.
//!
//! Run with: `cargo run --bin headless`
//!
//! Commands:
//!   tick [N]       - Advance N ticks (default 1)
//!   status         - Show colony resources and pop count
//!   pops           - Show detailed pop states
//!   map [x] [y]    - Show visual terrain around position
//!   scan [x] [y] [r] - Semantic terrain output (parseable)
//!   terrain <x> <y> - Get single tile info
//!   buildings      - List all buildings with positions
//!   build <type> <x> <y> - Build: farm, housing, stockpile
//!   mine <x> <y>   - Designate rock for mining
//!   chop <x> <y>   - Designate tree for chopping
//!   designations   - List all active designations
//!   find <terrain> [count] - Find terrain coordinates
//!   help           - Show this help
//!   quit           - Exit

use bevy_ecs::prelude::*;
use scale::layer1::{
    BuildMode, BuildingTracker, BuildingType, Chronicle, ChronicleUiState, ColonyMemory,
    ColonyResources, Designation, DesignationMode, DesignationType, Farm, GridPosition, Housing,
    MovementTarget, Needs, OccupiedTiles, Pop, PopAction, Stockpile, TerrainGrid, TerrainType,
    UtilityConfig, Viewport, arrival_handler_system, check_milestones_system,
    clean_dead_residents_system, clean_dead_workers_system, cleanup_previous_assignment_system,
    consume_food_system, decay_needs_system, evaluate_actions_system, generate_terrain,
    initial_chronicle_event, kill_starving_entities_system, movement_system,
    process_start_plan_system, produce_food_system, restore_rest_in_housing_system,
    spawn_initial_pops, track_plan_outcomes_system, try_designate, try_place_building,
    update_action_timer_system, update_resource_caps_system, work_execution_system,
};
use scale::shared::log::MessageLog;
use scale::shared::selection::Selection;
use scale::shared::state::GameState;
use scale::shared::time::SimulationTime;
use std::io::{self, BufRead, Write};

fn main() {
    let mut world = setup_world();

    println!("=== SCALE Headless Mode ===");
    println!("Type 'help' for commands, 'quit' to exit.\n");
    print_status(&mut world);
    println!();

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    loop {
        print!("> ");
        stdout.flush().unwrap();

        let mut input = String::new();
        if stdin.lock().read_line(&mut input).is_err() {
            break;
        }

        let input = input.trim();
        if input.is_empty() {
            continue;
        }

        let parts: Vec<&str> = input.split_whitespace().collect();
        let command = parts[0].to_lowercase();

        match command.as_str() {
            "quit" | "exit" | "q" => {
                println!("Goodbye!");
                break;
            }
            "help" | "h" | "?" => print_help(),
            "status" | "s" => print_status(&mut world),
            "pops" | "p" => print_pops(&mut world),
            "map" | "m" => {
                let x = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(40);
                let y = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(25);
                print_map(&mut world, x, y);
            }
            "tick" | "t" => {
                let n: u64 = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(1);
                run_ticks(&mut world, n);
            }
            "build" | "b" => {
                if parts.len() < 4 {
                    println!("Usage: build <farm|housing|stockpile> <x> <y>");
                } else {
                    let building_type = match parts[1].to_lowercase().as_str() {
                        "farm" | "f" => Some(BuildingType::Farm),
                        "housing" | "h" => Some(BuildingType::Housing),
                        "stockpile" | "s" => Some(BuildingType::Stockpile),
                        _ => None,
                    };
                    let x: Option<i32> = parts[2].parse().ok();
                    let y: Option<i32> = parts[3].parse().ok();

                    match (building_type, x, y) {
                        (Some(bt), Some(x), Some(y)) => build_at(&mut world, bt, x, y),
                        _ => println!(
                            "Invalid arguments. Usage: build <farm|housing|stockpile> <x> <y>"
                        ),
                    }
                }
            }
            "mine" => {
                if parts.len() < 3 {
                    println!("Usage: mine <x> <y>");
                } else {
                    let x: Option<i32> = parts[1].parse().ok();
                    let y: Option<i32> = parts[2].parse().ok();
                    match (x, y) {
                        (Some(x), Some(y)) => designate_at(&mut world, DesignationType::Mine, x, y),
                        _ => println!("Invalid coordinates"),
                    }
                }
            }
            "chop" => {
                if parts.len() < 3 {
                    println!("Usage: chop <x> <y>");
                } else {
                    let x: Option<i32> = parts[1].parse().ok();
                    let y: Option<i32> = parts[2].parse().ok();
                    match (x, y) {
                        (Some(x), Some(y)) => designate_at(&mut world, DesignationType::Chop, x, y),
                        _ => println!("Invalid coordinates"),
                    }
                }
            }
            "designations" | "d" => print_designations(&mut world),
            "find" => {
                if parts.len() < 2 {
                    println!("Usage: find <rock|tree|grass> [count]");
                } else {
                    let count: usize = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(10);
                    find_terrain(&mut world, parts[1], count);
                }
            }
            "scan" => {
                let x = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(40);
                let y = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(25);
                let radius = parts.get(3).and_then(|s| s.parse().ok()).unwrap_or(10);
                scan_terrain(&mut world, x, y, radius);
            }
            "terrain" => {
                if parts.len() < 3 {
                    println!("Usage: terrain <x> <y>");
                } else {
                    let x: Option<i32> = parts[1].parse().ok();
                    let y: Option<i32> = parts[2].parse().ok();
                    match (x, y) {
                        (Some(x), Some(y)) => get_tile_info(&mut world, x, y),
                        _ => println!("Invalid coordinates"),
                    }
                }
            }
            "buildings" => print_buildings(&mut world),
            _ => println!("Unknown command: '{}'. Type 'help' for commands.", command),
        }
        println!();
    }
}

fn setup_world() -> World {
    let mut world = World::new();
    world.insert_resource(GameState::Running);
    world.insert_resource(generate_terrain(80, 50));
    world.insert_resource(Viewport::default());
    world.insert_resource(SimulationTime::default());
    world.insert_resource(BuildMode::default());
    world.insert_resource(DesignationMode::default());
    world.insert_resource(OccupiedTiles::default());
    world.insert_resource(ColonyResources::default());
    world.insert_resource(MessageLog::default());
    world.insert_resource(Chronicle::default());
    world.insert_resource(ChronicleUiState::default());
    world.insert_resource(BuildingTracker::default());
    world.insert_resource(Selection::default());
    world.insert_resource(UtilityConfig::default());
    world.insert_resource(ColonyMemory::default());

    spawn_initial_pops(&mut world);
    initial_chronicle_event(&mut world);

    world
}

fn run_ticks(world: &mut World, n: u64) {
    let start_tick = world.resource::<SimulationTime>().tick;

    for _ in 0..n {
        // Run all game systems
        evaluate_actions_system(world);
        update_action_timer_system(world);
        cleanup_previous_assignment_system(world);
        process_start_plan_system(world);
        movement_system(world);
        arrival_handler_system(world);
        work_execution_system(world);
        update_resource_caps_system(world);
        produce_food_system(world);
        restore_rest_in_housing_system(world);
        consume_food_system(world);
        decay_needs_system(world);
        kill_starving_entities_system(world);
        clean_dead_residents_system(world);
        clean_dead_workers_system(world);
        track_plan_outcomes_system(world);
        check_milestones_system(world);
        world.resource_mut::<SimulationTime>().tick += 1;
    }

    let end_tick = world.resource::<SimulationTime>().tick;
    println!("Advanced {} ticks ({} -> {})", n, start_tick, end_tick);

    // Report any interesting events
    report_events(world);
}

fn report_events(world: &mut World) {
    // Check for pops doing things
    let mut moving = 0;
    let mut working = 0;
    let mut at_farm = 0;
    let mut at_housing = 0;

    for (_, action, mt, assigned) in world
        .query::<(
            Entity,
            &PopAction,
            Option<&MovementTarget>,
            Option<&scale::layer1::AssignedTo>,
        )>()
        .iter(world)
    {
        if mt.is_some() {
            moving += 1;
        }
        match action.current {
            scale::layer1::ActionType::Work => working += 1,
            scale::layer1::ActionType::SatisfyHunger => at_farm += 1,
            scale::layer1::ActionType::SatisfyRest => at_housing += 1,
            _ => {}
        }
        if assigned.is_some() {
            // Already counted above
        }
    }

    if moving > 0 || working > 0 || at_farm > 0 || at_housing > 0 {
        println!(
            "  Activity: {} moving, {} working, {} eating, {} resting",
            moving, working, at_farm, at_housing
        );
    }
}

fn print_status(world: &mut World) {
    // Copy resource values before querying to avoid borrow conflicts
    let (food, wood, stone) = {
        let r = world.resource::<ColonyResources>();
        (r.food, r.wood, r.stone)
    };
    let tick = world.resource::<SimulationTime>().tick;
    let pop_count = world.query::<&Pop>().iter(world).count();
    let farm_count = world.query::<&Farm>().iter(world).count();
    let housing_count = world.query::<&Housing>().iter(world).count();
    let designation_count = world.query::<&Designation>().iter(world).count();

    println!("=== Colony Status (Tick {}) ===", tick);
    println!("Population: {} pops", pop_count);
    println!(
        "Resources: {:.1} food, {:.1} wood, {:.1} stone",
        food, wood, stone
    );
    println!("Buildings: {} farms, {} housing", farm_count, housing_count);
    println!("Designations: {} active", designation_count);
}

fn print_pops(world: &mut World) {
    println!("=== Pop Details ===");

    for (entity, pos, needs, action) in world
        .query::<(Entity, &GridPosition, &Needs, &PopAction)>()
        .iter(world)
    {
        let mt = world.get::<MovementTarget>(entity);
        let at_target = world.get::<scale::layer1::AtTarget>(entity).is_some();

        let status = if at_target {
            "at target".to_string()
        } else if let Some(mt) = mt {
            format!(
                "moving to ({},{})",
                mt.target_position.x, mt.target_position.y
            )
        } else {
            "idle".to_string()
        };

        println!(
            "Pop {:?} at ({},{}): hunger={:.0}% rest={:.0}% action={:?} [{}]",
            entity,
            pos.x,
            pos.y,
            needs.hunger * 100.0,
            needs.rest * 100.0,
            action.current,
            status
        );
    }
}

fn print_map(world: &mut World, center_x: i32, center_y: i32) {
    let radius = 10;

    // Copy terrain data before querying to avoid borrow conflicts
    let (width, height, terrain_tiles) = {
        let terrain = world.resource::<TerrainGrid>();
        let mut tiles = std::collections::HashMap::new();
        for y in (center_y - radius)..=(center_y + radius) {
            for x in (center_x - radius)..=(center_x + radius) {
                if x >= 0 && y >= 0 && x < terrain.width as i32 && y < terrain.height as i32 {
                    if let Some(t) = terrain.get(x as usize, y as usize) {
                        tiles.insert((x, y), t);
                    }
                }
            }
        }
        (terrain.width, terrain.height, tiles)
    };

    println!("=== Map around ({}, {}) ===", center_x, center_y);

    // Collect pop positions
    let pop_positions: Vec<(i32, i32)> = world
        .query::<&GridPosition>()
        .iter(world)
        .map(|p| (p.x, p.y))
        .collect();

    // Collect designation positions
    let designation_positions: Vec<(i32, i32, DesignationType)> = world
        .query::<(&GridPosition, &Designation)>()
        .iter(world)
        .map(|(p, d)| (p.x, p.y, d.designation_type))
        .collect();

    for y in (center_y - radius)..=(center_y + radius) {
        print!("{:3} ", y);
        for x in (center_x - radius)..=(center_x + radius) {
            if x < 0 || y < 0 || x >= width as i32 || y >= height as i32 {
                print!(" ");
                continue;
            }

            // Check for pop
            if pop_positions.iter().any(|&(px, py)| px == x && py == y) {
                print!("@");
                continue;
            }

            // Check for designation
            if let Some((_, _, dt)) = designation_positions
                .iter()
                .find(|&&(dx, dy, _)| dx == x && dy == y)
            {
                let c = match dt {
                    DesignationType::Mine => '⛏',
                    DesignationType::Chop => '⚒',
                    DesignationType::Demolish => 'X',
                };
                print!("{}", c);
                continue;
            }

            // Show terrain
            let tile = terrain_tiles
                .get(&(x, y))
                .copied()
                .unwrap_or(TerrainType::Grass);
            let c = match tile {
                TerrainType::Grass => '.',
                TerrainType::Dirt => ',',
                TerrainType::Rock => '#',
                TerrainType::Water => '~',
                TerrainType::Tree => 'T',
            };
            print!("{}", c);
        }
        println!();
    }
    println!("Legend: @=pop .=grass ,=dirt #=rock ~=water T=tree ⛏=mine ⚒=chop");
}

fn build_at(world: &mut World, building_type: BuildingType, x: i32, y: i32) {
    // Give resources for building
    {
        let mut resources = world.resource_mut::<ColonyResources>();
        resources.wood = 100.0;
        resources.stone = 100.0;
    }

    let success = try_place_building(world, x, y, building_type);
    if success {
        println!("Built {:?} at ({}, {})", building_type, x, y);
    } else {
        // Check why it failed
        let terrain = world.resource::<TerrainGrid>();
        let tile = terrain.get(x as usize, y as usize);
        let occupied = world.resource::<OccupiedTiles>();

        if tile.is_none() {
            println!("Failed: ({}, {}) is out of bounds", x, y);
        } else if occupied.0.contains(&(x, y)) {
            println!("Failed: ({}, {}) is already occupied", x, y);
        } else if let Some(t) = tile {
            println!("Failed: cannot build on {:?} at ({}, {})", t, x, y);
        }
    }
}

fn designate_at(world: &mut World, designation_type: DesignationType, x: i32, y: i32) {
    let success = try_designate(world, x, y, designation_type);
    if success {
        println!("Designated {:?} at ({}, {})", designation_type, x, y);
    } else {
        // Check why it failed
        let terrain = world.resource::<TerrainGrid>();
        let tile = terrain.get(x as usize, y as usize);

        match designation_type {
            DesignationType::Mine => {
                if tile != Some(TerrainType::Rock) {
                    println!("Failed: ({}, {}) is {:?}, need Rock for mining", x, y, tile);
                } else {
                    println!("Failed: already designated at ({}, {})", x, y);
                }
            }
            DesignationType::Chop => {
                if tile != Some(TerrainType::Tree) {
                    println!(
                        "Failed: ({}, {}) is {:?}, need Tree for chopping",
                        x, y, tile
                    );
                } else {
                    println!("Failed: already designated at ({}, {})", x, y);
                }
            }
            DesignationType::Demolish => {
                println!("Failed: no building at ({}, {})", x, y);
            }
        }
    }
}

fn print_designations(world: &mut World) {
    println!("=== Active Designations ===");

    let mut count = 0;
    for (pos, designation) in world.query::<(&GridPosition, &Designation)>().iter(world) {
        println!(
            "  {:?} at ({}, {})",
            designation.designation_type, pos.x, pos.y
        );
        count += 1;
    }

    if count == 0 {
        println!("  (none)");
    }
}

fn find_terrain(world: &mut World, terrain_name: &str, max_count: usize) {
    let target = match terrain_name.to_lowercase().as_str() {
        "rock" | "r" => TerrainType::Rock,
        "tree" | "t" => TerrainType::Tree,
        "grass" | "g" => TerrainType::Grass,
        "water" | "w" => TerrainType::Water,
        "dirt" | "d" => TerrainType::Dirt,
        _ => {
            println!(
                "ERROR: Unknown terrain type: {}. Try: rock, tree, grass, water, dirt",
                terrain_name
            );
            return;
        }
    };

    let terrain = world.resource::<TerrainGrid>();
    let mut found = Vec::new();

    for y in 0..terrain.height {
        for x in 0..terrain.width {
            if terrain.get(x, y) == Some(target) {
                found.push((x, y));
                if found.len() >= max_count {
                    break;
                }
            }
        }
        if found.len() >= max_count {
            break;
        }
    }

    if found.is_empty() {
        println!("FOUND: {:?} count=0", target);
    } else {
        println!("FOUND: {:?} count={}", target, found.len());
        for (x, y) in &found {
            println!("  COORD: {} {}", x, y);
        }
    }
}

/// Semantic terrain scan - outputs parseable coordinate:type pairs
fn scan_terrain(world: &mut World, center_x: i32, center_y: i32, radius: i32) {
    // Copy terrain data before querying to avoid borrow conflicts
    let (width, height, terrain_tiles) = {
        let terrain = world.resource::<TerrainGrid>();
        let mut tiles = std::collections::HashMap::new();
        for y in (center_y - radius)..=(center_y + radius) {
            for x in (center_x - radius)..=(center_x + radius) {
                if x >= 0 && y >= 0 && x < terrain.width as i32 && y < terrain.height as i32 {
                    if let Some(t) = terrain.get(x as usize, y as usize) {
                        tiles.insert((x, y), t);
                    }
                }
            }
        }
        (terrain.width, terrain.height, tiles)
    };

    println!("SCAN: center=({},{}) radius={}", center_x, center_y, radius);
    println!("BOUNDS: width={} height={}", width, height);

    // Collect entities at positions
    let pop_positions: Vec<(i32, i32)> = world
        .query::<(&Pop, &GridPosition)>()
        .iter(world)
        .map(|(_, p)| (p.x, p.y))
        .collect();

    let farm_positions: Vec<(i32, i32)> = world
        .query::<(&Farm, &GridPosition)>()
        .iter(world)
        .map(|(_, p)| (p.x, p.y))
        .collect();

    let housing_positions: Vec<(i32, i32)> = world
        .query::<(&Housing, &GridPosition)>()
        .iter(world)
        .map(|(_, p)| (p.x, p.y))
        .collect();

    let designation_positions: Vec<(i32, i32, String)> = world
        .query::<(&Designation, &GridPosition)>()
        .iter(world)
        .map(|(d, p)| {
            let dt = match d.designation_type {
                DesignationType::Mine => "mine",
                DesignationType::Chop => "chop",
                DesignationType::Demolish => "demolish",
            };
            (p.x, p.y, dt.to_string())
        })
        .collect();

    for y in (center_y - radius)..=(center_y + radius) {
        for x in (center_x - radius)..=(center_x + radius) {
            if x < 0 || y < 0 || x >= width as i32 || y >= height as i32 {
                continue;
            }

            let tile = terrain_tiles
                .get(&(x, y))
                .copied()
                .unwrap_or(TerrainType::Grass);
            let terrain_name = match tile {
                TerrainType::Grass => "grass",
                TerrainType::Dirt => "dirt",
                TerrainType::Rock => "rock",
                TerrainType::Water => "water",
                TerrainType::Tree => "tree",
            };

            // Check for entities
            let has_pop = pop_positions.iter().any(|&(px, py)| px == x && py == y);
            let has_farm = farm_positions.iter().any(|&(fx, fy)| fx == x && fy == y);
            let has_housing = housing_positions.iter().any(|&(hx, hy)| hx == x && hy == y);
            let designation = designation_positions
                .iter()
                .find(|(dx, dy, _)| *dx == x && *dy == y);

            let mut entities = Vec::new();
            if has_pop {
                entities.push("pop");
            }
            if has_farm {
                entities.push("farm");
            }
            if has_housing {
                entities.push("housing");
            }
            if let Some((_, _, dt)) = designation {
                entities.push(dt.as_str());
            }

            if entities.is_empty() {
                println!("TILE: {} {} terrain={}", x, y, terrain_name);
            } else {
                println!(
                    "TILE: {} {} terrain={} entities={}",
                    x,
                    y,
                    terrain_name,
                    entities.join(",")
                );
            }
        }
    }
    println!("SCAN_END");
}

/// Get info about a single tile
fn get_tile_info(world: &mut World, x: i32, y: i32) {
    let terrain = world.resource::<TerrainGrid>();

    if x < 0 || y < 0 || x >= terrain.width as i32 || y >= terrain.height as i32 {
        println!("TILE_INFO: {} {} ERROR=out_of_bounds", x, y);
        return;
    }

    let tile = terrain
        .get(x as usize, y as usize)
        .unwrap_or(TerrainType::Grass);
    let terrain_name = match tile {
        TerrainType::Grass => "grass",
        TerrainType::Dirt => "dirt",
        TerrainType::Rock => "rock",
        TerrainType::Water => "water",
        TerrainType::Tree => "tree",
    };

    let walkable = tile.is_walkable();
    let buildable = matches!(tile, TerrainType::Grass | TerrainType::Dirt);

    // Check for entities
    let has_pop = world
        .query::<(&Pop, &GridPosition)>()
        .iter(world)
        .any(|(_, p)| p.x == x && p.y == y);

    let has_farm = world
        .query::<(&Farm, &GridPosition)>()
        .iter(world)
        .any(|(_, p)| p.x == x && p.y == y);

    let has_housing = world
        .query::<(&Housing, &GridPosition)>()
        .iter(world)
        .any(|(_, p)| p.x == x && p.y == y);

    let occupied = world.resource::<OccupiedTiles>().0.contains(&(x, y));

    println!(
        "TILE_INFO: {} {} terrain={} walkable={} buildable={} occupied={} pop={} farm={} housing={}",
        x, y, terrain_name, walkable, buildable, occupied, has_pop, has_farm, has_housing
    );
}

/// List all buildings with positions
fn print_buildings(world: &mut World) {
    println!("BUILDINGS:");

    let mut count = 0;

    for (entity, pos, farm) in world.query::<(Entity, &GridPosition, &Farm)>().iter(world) {
        println!(
            "  BUILDING: {:?} type=farm pos=({},{}) workers={}/{}",
            entity,
            pos.x,
            pos.y,
            farm.workers.len(),
            farm.capacity
        );
        count += 1;
    }

    for (entity, pos, housing) in world
        .query::<(Entity, &GridPosition, &Housing)>()
        .iter(world)
    {
        println!(
            "  BUILDING: {:?} type=housing pos=({},{}) residents={}/{}",
            entity,
            pos.x,
            pos.y,
            housing.residents.len(),
            housing.capacity
        );
        count += 1;
    }

    for (entity, pos, _) in world
        .query::<(Entity, &GridPosition, &Stockpile)>()
        .iter(world)
    {
        println!(
            "  BUILDING: {:?} type=stockpile pos=({},{})",
            entity, pos.x, pos.y
        );
        count += 1;
    }

    println!("BUILDINGS_END: count={}", count);
}

fn print_help() {
    println!("=== Commands ===");
    println!("  tick [N]              - Advance N ticks (default 1)");
    println!("  status, s             - Show colony resources and pop count");
    println!("  pops, p               - Show detailed pop states");
    println!("  map [x] [y]           - Show visual terrain around position");
    println!("  scan [x] [y] [r]      - Semantic terrain scan (parseable)");
    println!("  terrain <x> <y>       - Get single tile info");
    println!("  buildings             - List all buildings with positions");
    println!("  build <type> <x> <y>  - Build: farm, housing, stockpile");
    println!("  mine <x> <y>          - Designate rock for mining");
    println!("  chop <x> <y>          - Designate tree for chopping");
    println!("  designations, d       - List all active designations");
    println!("  find <terrain> [N]    - Find N terrain coords (default 10)");
    println!("  help, h               - Show this help");
    println!("  quit, q               - Exit");
}

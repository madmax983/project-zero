//! Headless game runner with semantic input/output.
//!
//! Run with: `cargo run --bin headless`
//!
//! Commands:
//!   `tick [N]`       - Advance N ticks (default 1)
//!   `status`         - Show colony resources and pop count
//!   `pops`           - Show detailed pop states
//!   `map [x] [y]`    - Show visual terrain around position
//!   `scan [x] [y] [r]` - Semantic terrain output (parseable)
//!   `terrain <x> <y>` - Get single tile info
//!   `buildings`      - List all buildings with positions
//!   `build <type> <x> <y>` - Build: farm, housing, stockpile
//!   `mine <x> <y>`   - Designate rock for mining
//!   `chop <x> <y>`   - Designate tree for chopping
//!   `designations`   - List all active designations
//!   `find <terrain> [count]` - Find terrain coordinates
//!   `help`           - Show this help
//!   `quit`           - Exit

#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::needless_pass_by_ref_mut)]
#![allow(clippy::too_many_lines)]

use bevy_ecs::prelude::*;
use scale::layer1::biography::Biography;
use scale::layer1::dreams::Dream;
use scale::layer1::pop::PopName;
use scale::layer1::{
    BuildingType, ColonyResources, Designation, DesignationType, Farm, GridPosition, Housing,
    MovementTarget, Needs, OccupiedTiles, Pop, PopAction, Stockpile, TerrainGrid, TerrainType,
    try_designate, try_place_building,
};
use scale::setup::{SetupConfig, setup_world_with_config};
use scale::shared::state::GameState;
use scale::shared::time::SimulationTime;
use scale::simulation::run_simulation_tick;
use std::io::{self, BufRead, Write};
use comfy_table::{Table, Cell, presets::UTF8_FULL, ContentArrangement, Color, Attribute};
use crossterm::style::Stylize;

fn main() {
    let mut world = setup_world_with_config(SetupConfig {
        headless: true,
        ..Default::default()
    });
    *world.resource_mut::<GameState>() = GameState::Running;

    println!("=== SCALE Headless Mode ===");
    println!("Type 'help' for commands, 'quit' to exit.\n");
    print_status(&mut world);
    println!();

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    loop {
        print!("> ");
        if let Err(e) = stdout.flush() {
            eprintln!("Error flushing stdout: {e}");
            break;
        }

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
            "bio" => {
                let id: Option<u32> = parts.get(1).and_then(|s| s.parse().ok());
                match id {
                    Some(id) => print_bio(&mut world, id),
                    None => println!("Usage: bio <id>"),
                }
            }
            _ => println!("Unknown command: '{command}'. Type 'help' for commands."),
        }
        println!();
    }
}

fn run_ticks(world: &mut World, n: u64) {
    let start_tick = world.resource::<SimulationTime>().tick;

    for _ in 0..n {
        run_simulation_tick(world);
    }

    let end_tick = world.resource::<SimulationTime>().tick;
    println!("Advanced {n} ticks ({start_tick} -> {end_tick})");

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
            "  Activity: {moving} moving, {working} working, {at_farm} eating, {at_housing} resting"
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

    println!("{}", format!("=== COLONY STATUS (Tick {tick}) ===").green().bold());

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("Category").add_attribute(Attribute::Bold),
            Cell::new("Metric").add_attribute(Attribute::Bold),
            Cell::new("Value").add_attribute(Attribute::Bold),
        ]);

    table.add_row(vec![
        Cell::new("Population").fg(Color::Cyan),
        Cell::new("Citizens"),
        Cell::new(pop_count.to_string()),
    ]);

    table.add_row(vec![
        Cell::new("Resources").fg(Color::Yellow),
        Cell::new("Food"),
        Cell::new(format!("{:.1}", food)).fg(if food < 20.0 { Color::Red } else { Color::Green }),
    ]);
    table.add_row(vec![
        Cell::new(""),
        Cell::new("Wood"),
        Cell::new(format!("{:.1}", wood)),
    ]);
    table.add_row(vec![
        Cell::new(""),
        Cell::new("Stone"),
        Cell::new(format!("{:.1}", stone)),
    ]);

    table.add_row(vec![
        Cell::new("Buildings").fg(Color::Magenta),
        Cell::new("Farms"),
        Cell::new(farm_count.to_string()),
    ]);
    table.add_row(vec![
        Cell::new(""),
        Cell::new("Housing"),
        Cell::new(housing_count.to_string()),
    ]);

    table.add_row(vec![
        Cell::new("Tasks").fg(Color::Blue),
        Cell::new("Active Designations"),
        Cell::new(designation_count.to_string()),
    ]);

    println!("{table}");
}

fn print_pops(world: &mut World) {
    println!("{}", "=== Pop Details ===".green().bold());

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("ID").add_attribute(Attribute::Bold),
            Cell::new("Name").add_attribute(Attribute::Bold),
            Cell::new("Pos").add_attribute(Attribute::Bold),
            Cell::new("Hunger").add_attribute(Attribute::Bold),
            Cell::new("Rest").add_attribute(Attribute::Bold),
            Cell::new("Action").add_attribute(Attribute::Bold),
            Cell::new("Status").add_attribute(Attribute::Bold),
        ]);

    for (entity, name, pos, needs, action) in world
        .query::<(Entity, &PopName, &GridPosition, &Needs, &PopAction)>()
        .iter(world)
    {
        let mt = world.get::<MovementTarget>(entity);
        let at_target = world.get::<scale::layer1::AtTarget>(entity).is_some();

        let status = if at_target {
            "at target".to_string()
        } else if let Some(mt) = mt {
            format!("mov ({},{})", mt.target_position.x, mt.target_position.y)
        } else {
            "-".to_string()
        };

        let action_str = format!("{:?}", action.current);

        // Color code needs: Low is BAD (Red), High is GOOD (Green) ??
        // Wait, hunger is 0..1. Usually 1.0 is full (good).
        // Let's assume 1.0 is Satiated (Good). 0.0 is Starving (Bad).
        let hunger_color = if needs.hunger < 0.2 { Color::Red } else if needs.hunger < 0.5 { Color::Yellow } else { Color::Green };
        let rest_color = if needs.rest < 0.2 { Color::Red } else if needs.rest < 0.5 { Color::Yellow } else { Color::Green };

        table.add_row(vec![
            Cell::new(entity.index().to_string()),
            Cell::new(&name.0),
            Cell::new(format!("{},{}", pos.x, pos.y)),
            Cell::new(format!("{:.0}%", needs.hunger * 100.0)).fg(hunger_color),
            Cell::new(format!("{:.0}%", needs.rest * 100.0)).fg(rest_color),
            Cell::new(action_str),
            Cell::new(status),
        ]);
    }

    println!("{table}");
}

fn print_map(world: &mut World, center_x: i32, center_y: i32) {
    let radius = 10;

    // Copy terrain data before querying to avoid borrow conflicts
    let (width, height, terrain_tiles) = {
        let terrain = world.resource::<TerrainGrid>();
        let max_x = i32::try_from(terrain.width).unwrap_or(i32::MAX);
        let max_y = i32::try_from(terrain.height).unwrap_or(i32::MAX);

        let mut tiles = std::collections::HashMap::new();
        for y in (center_y - radius)..=(center_y + radius) {
            for x in (center_x - radius)..=(center_x + radius) {
                if x >= 0
                    && y >= 0
                    && x < max_x
                    && y < max_y
                    && let Some(t) = terrain.get(x as usize, y as usize)
                {
                    tiles.insert((x, y), t);
                }
            }
        }
        (max_x, max_y, tiles)
    };

    println!("=== Map around ({center_x}, {center_y}) ===");

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
        print!("{y:3} ");
        for x in (center_x - radius)..=(center_x + radius) {
            if x < 0 || y < 0 || x >= width || y >= height {
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
                    DesignationType::Mine => '%',
                    DesignationType::Chop => '/',
                    DesignationType::Demolish => 'X',
                    DesignationType::Repair => '+',
                    DesignationType::SetZone(_) => 'Z',
                    DesignationType::Tame => 'T',
                    DesignationType::ClearFlora => 'F',
                    DesignationType::JuryRig => 'J',
                    DesignationType::Cannibalize => 'C',
                };
                print!("{c}");
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
                TerrainType::Path => '=',
            };
            print!("{c}");
        }
        println!();
    }
    println!("Legend: @=pop .=grass ,=dirt #=rock ~=water T=tree %=mine /=chop");
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
        println!("Built {building_type:?} at ({x}, {y})");
    } else {
        // Check why it failed
        let terrain = world.resource::<TerrainGrid>();
        let tile = terrain.get(x as usize, y as usize);
        let occupied = world.resource::<OccupiedTiles>();

        if tile.is_none() {
            println!("Failed: ({x}, {y}) is out of bounds");
        } else if occupied.0.contains(&(x, y)) {
            println!("Failed: ({x}, {y}) is already occupied");
        } else if let Some(t) = tile {
            println!("Failed: cannot build on {t:?} at ({x}, {y})");
        }
    }
}

fn designate_at(world: &mut World, designation_type: DesignationType, x: i32, y: i32) {
    let success = try_designate(world, x, y, designation_type);
    if success {
        println!("Designated {designation_type:?} at ({x}, {y})");
    } else {
        // Check why it failed
        let terrain = world.resource::<TerrainGrid>();
        let tile = terrain.get(x as usize, y as usize);

        match designation_type {
            DesignationType::Mine => {
                if tile == Some(TerrainType::Rock) {
                    println!("Failed: already designated at ({x}, {y})");
                } else {
                    println!("Failed: ({x}, {y}) is {tile:?}, need Rock for mining");
                }
            }
            DesignationType::Chop => {
                if tile == Some(TerrainType::Tree) {
                    println!("Failed: already designated at ({x}, {y})");
                } else {
                    println!("Failed: ({x}, {y}) is {tile:?}, need Tree for chopping");
                }
            }
            DesignationType::Demolish => {
                println!("Failed: no building at ({x}, {y})");
            }
            DesignationType::Repair => {
                println!("Failed: no building to repair at ({x}, {y})");
            }
            DesignationType::SetZone(_) => {
                println!("Failed: cannot set zone at ({x}, {y})");
            }
            DesignationType::Tame => {
                println!("Failed: no wild animal at ({x}, {y})");
            }
            DesignationType::ClearFlora => {
                println!("Failed: no flora at ({x}, {y})");
            }
            DesignationType::JuryRig => {
                println!("Failed: no building to jury-rig at ({x}, {y})");
            }
            DesignationType::Cannibalize => {
                println!("Failed: no Lander at ({x}, {y})");
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
                "ERROR: Unknown terrain type: {terrain_name}. Try: rock, tree, grass, water, dirt"
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
        println!("FOUND: {target:?} count=0");
    } else {
        println!("FOUND: {:?} count={}", target, found.len());
        for (x, y) in &found {
            println!("  COORD: {x} {y}");
        }
    }
}

/// Semantic terrain scan - outputs parseable coordinate:type pairs
fn scan_terrain(world: &mut World, center_x: i32, center_y: i32, radius: i32) {
    // Copy terrain data before querying to avoid borrow conflicts
    let (width, height, terrain_tiles) = {
        let terrain = world.resource::<TerrainGrid>();
        let max_x = i32::try_from(terrain.width).unwrap_or(i32::MAX);
        let max_y = i32::try_from(terrain.height).unwrap_or(i32::MAX);

        let mut tiles = std::collections::HashMap::new();
        for y in (center_y - radius)..=(center_y + radius) {
            for x in (center_x - radius)..=(center_x + radius) {
                if x >= 0
                    && y >= 0
                    && x < max_x
                    && y < max_y
                    && let Some(t) = terrain.get(x as usize, y as usize)
                {
                    tiles.insert((x, y), t);
                }
            }
        }
        (max_x, max_y, tiles)
    };

    println!("SCAN: center=({center_x},{center_y}) radius={radius}");
    println!("BOUNDS: width={width} height={height}");

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
                DesignationType::Repair => "repair",
                DesignationType::SetZone(_) => "zone",
                DesignationType::Tame => "tame",
                DesignationType::ClearFlora => "clear_flora",
                DesignationType::JuryRig => "jury_rig",
                DesignationType::Cannibalize => "cannibalize",
            };
            (p.x, p.y, dt.to_string())
        })
        .collect();

    for y in (center_y - radius)..=(center_y + radius) {
        for x in (center_x - radius)..=(center_x + radius) {
            if x < 0 || y < 0 || x >= width || y >= height {
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
                TerrainType::Path => "path",
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
                println!("TILE: {x} {y} terrain={terrain_name}");
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
    let max_x = i32::try_from(terrain.width).unwrap_or(i32::MAX);
    let max_y = i32::try_from(terrain.height).unwrap_or(i32::MAX);

    if x < 0 || y < 0 || x >= max_x || y >= max_y {
        println!("TILE_INFO: {x} {y} ERROR=out_of_bounds");
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
        TerrainType::Path => "path",
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
        "TILE_INFO: {x} {y} terrain={terrain_name} walkable={walkable} buildable={buildable} occupied={occupied} pop={has_pop} farm={has_farm} housing={has_housing}"
    );
}

/// List all buildings with positions
fn print_buildings(world: &mut World) {
    println!("{}", "=== Buildings ===".green().bold());

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("ID").add_attribute(Attribute::Bold),
            Cell::new("Type").add_attribute(Attribute::Bold),
            Cell::new("Pos").add_attribute(Attribute::Bold),
            Cell::new("Occupancy").add_attribute(Attribute::Bold),
        ]);

    let mut count = 0;

    for (entity, pos, farm) in world.query::<(Entity, &GridPosition, &Farm)>().iter(world) {
        table.add_row(vec![
            Cell::new(entity.index().to_string()),
            Cell::new("Farm").fg(Color::Green),
            Cell::new(format!("{},{}", pos.x, pos.y)),
            Cell::new(format!("{}/{}", farm.workers.len(), farm.capacity)),
        ]);
        count += 1;
    }

    for (entity, pos, housing) in world
        .query::<(Entity, &GridPosition, &Housing)>()
        .iter(world)
    {
        table.add_row(vec![
            Cell::new(entity.index().to_string()),
            Cell::new("Housing").fg(Color::Blue),
            Cell::new(format!("{},{}", pos.x, pos.y)),
            Cell::new(format!("{}/{}", housing.residents.len(), housing.capacity)),
        ]);
        count += 1;
    }

    for (entity, pos, _) in world
        .query::<(Entity, &GridPosition, &Stockpile)>()
        .iter(world)
    {
        table.add_row(vec![
            Cell::new(entity.index().to_string()),
            Cell::new("Stockpile").fg(Color::Yellow),
            Cell::new(format!("{},{}", pos.x, pos.y)),
            Cell::new("-"),
        ]);
        count += 1;
    }

    if count == 0 {
        println!("  (No buildings found)");
    } else {
        println!("{table}");
    }
}

fn print_bio(world: &mut World, target_id: u32) {
    let mut query = world.query::<(Entity, &PopName, Option<&Biography>, Option<&Dream>)>();
    let mut found = false;

    for (entity, name, bio, dream) in query.iter(world) {
        if entity.index() == target_id {
            found = true;
            println!("=== Biography for {} ({:?}) ===", name.0, entity);

            println!("Life Events:");
            if let Some(bio) = bio {
                if bio.events.is_empty() {
                    println!("  (No events recorded)");
                } else {
                    for event in &bio.events {
                        println!("  [Tick {:>6}] {}", event.tick, event.text);
                    }
                }
            } else {
                println!("  (No biography component)");
            }

            if let Some(dream) = dream {
                println!("\nLast Dream (Tick {}):", dream.tick);
                println!("  \"{}\"", dream.content);
            }
            break;
        }
    }

    if !found {
        println!("Pop with ID {target_id} not found.");
    }
}

fn print_help() {
    println!("{}", "=== Commands ===".green().bold());

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("Command").add_attribute(Attribute::Bold),
            Cell::new("Alias").add_attribute(Attribute::Bold),
            Cell::new("Description").add_attribute(Attribute::Bold),
        ]);

    let commands = vec![
        ("tick [N]", "", "Advance N ticks (default 1)"),
        ("status", "s", "Show colony resources and pop count"),
        ("pops", "p", "Show detailed pop states"),
        ("map [x] [y]", "m", "Show visual terrain around position"),
        ("scan [x] [y] [r]", "", "Semantic terrain scan (parseable)"),
        ("terrain <x> <y>", "", "Get single tile info"),
        ("buildings", "", "List all buildings with positions"),
        ("build <type> <x> <y>", "b", "Build: farm, housing, stockpile"),
        ("mine <x> <y>", "", "Designate rock for mining"),
        ("chop <x> <y>", "", "Designate tree for chopping"),
        ("designations", "d", "List all active designations"),
        ("bio <id>", "", "Show biography and dreams of a pop"),
        ("find <type> [N]", "", "Find N terrain coords (default 10)"),
        ("help", "h, ?", "Show this help"),
        ("quit", "q, exit", "Exit"),
    ];

    for (cmd, alias, desc) in commands {
        table.add_row(vec![
            Cell::new(cmd).fg(Color::Cyan),
            Cell::new(alias).fg(Color::DarkGrey),
            Cell::new(desc),
        ]);
    }

    println!("{table}");
}

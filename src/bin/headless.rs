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
use comfy_table::{presets::UTF8_FULL, Attribute, Cell, Color, ContentArrangement, Table};
use crossterm::style::Stylize;
use scale::layer1::biography::Biography;
use scale::layer1::dreams::Dream;
use scale::layer1::pop::PopName;
use scale::layer1::tech::{unlock_tech, Tech, TechState, TechStatus};
use scale::layer1::{
    try_designate, try_place_building, Building, BuildingType, Chronicle, ColonyResources,
    Designation, DesignationType, EventImportance, Farm, GlobalWind, GridPosition, Housing, Morale,
    MovementTarget, Needs, OccupiedTiles, Pop, PopAction, Stockpile, TerrainGrid, TerrainType,
};
use scale::setup::{setup_world_with_config, SetupConfig};
use scale::shared::log::MessageLog;
use scale::shared::state::GameState;
use scale::shared::time::SimulationTime;
use scale::simulation::run_simulation_tick;
use std::io::{self, BufRead, Write};

fn main() {
    let mut world = setup_world_with_config(SetupConfig { headless: true });
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

        if !handle_command(&mut world, input) {
            break;
        }
        println!();
    }
}

fn handle_command(world: &mut World, input: &str) -> bool {
    let parts: Vec<&str> = input.split_whitespace().collect();
    let command = parts[0].to_lowercase();

    match command.as_str() {
        "quit" | "exit" | "q" => {
            println!("Goodbye!");
            return false;
        }
        "help" | "h" | "?" => print_help(),
        "status" | "s" => print_status(world),
        "pops" | "p" => print_pops(world),
        "map" | "m" => {
            let x = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(40);
            let y = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(25);
            print_map(world, x, y);
        }
        "tick" | "t" => {
            let n: u64 = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(1);
            // Cap tick count to prevent DoS (accidental or malicious infinite loops)
            let safe_n = n.min(1000);
            if n > 1000 {
                println!("Warning: Capping ticks to 1000 to prevent freeze.");
            }
            run_ticks(world, safe_n);
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
                    (Some(bt), Some(x), Some(y)) => build_at(world, bt, x, y),
                    _ => {
                        println!(
                            "Invalid arguments. Usage: build <farm|housing|stockpile> <x> <y>"
                        );
                    }
                }
            }
        }
        "destroy" => {
            if parts.len() < 3 {
                println!("Usage: destroy <x> <y>");
            } else {
                let x: Option<i32> = parts[1].parse().ok();
                let y: Option<i32> = parts[2].parse().ok();
                match (x, y) {
                    (Some(x), Some(y)) => designate_at(world, DesignationType::Destroy, x, y),
                    _ => println!("Invalid coordinates"),
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
                    (Some(x), Some(y)) => designate_at(world, DesignationType::Mine, x, y),
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
                    (Some(x), Some(y)) => designate_at(world, DesignationType::Chop, x, y),
                    _ => println!("Invalid coordinates"),
                }
            }
        }
        "designations" | "d" => print_designations(world),
        "find" => {
            if parts.len() < 2 {
                println!("Usage: find <rock|tree|grass> [count]");
            } else {
                let count: usize = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(10);
                find_terrain(world, parts[1], count);
            }
        }
        "scan" => {
            let x = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(40);
            let y = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(25);
            let radius = parts.get(3).and_then(|s| s.parse().ok()).unwrap_or(10);
            scan_terrain(world, x, y, radius);
        }
        "terrain" => {
            if parts.len() < 3 {
                println!("Usage: terrain <x> <y>");
            } else {
                let x: Option<i32> = parts[1].parse().ok();
                let y: Option<i32> = parts[2].parse().ok();
                match (x, y) {
                    (Some(x), Some(y)) => get_tile_info(world, x, y),
                    _ => println!("Invalid coordinates"),
                }
            }
        }
        "buildings" => print_buildings(world),
        "bio" => {
            let id: Option<u32> = parts.get(1).and_then(|s| s.parse().ok());
            match id {
                Some(id) => print_bio(world, id),
                None => println!("Usage: bio <id>"),
            }
        }
        "chronicle" | "c" | "history" => print_chronicle(world),
        "log" | "l" => print_log(world),
        "tech" | "research_status" => print_tech(world),
        "research" | "r" => {
            if parts.len() < 2 {
                println!("Usage: research <tech_name>");
            } else {
                // Join parts in case tech name has spaces (e.g., "Metal Working")
                let tech_name = parts[1..].join(" ").to_lowercase();

                let tech = match tech_name.as_str() {
                    "masonry" => Some(Tech::Masonry),
                    "metal working" | "metalworking" => Some(Tech::MetalWorking),
                    "social structures" | "social" => Some(Tech::SocialStructures),
                    "astronomy" => Some(Tech::Astronomy),
                    "hydroponics" => Some(Tech::Hydroponics),
                    "militia" => Some(Tech::Militia),
                    "medical" => Some(Tech::Medical),
                    "electromagnetism" => Some(Tech::Electromagnetism),
                    "void whispers" | "void" => Some(Tech::VoidWhispers),
                    "terraforming" => Some(Tech::Terraforming),
                    _ => None,
                };

                if let Some(t) = tech {
                    if unlock_tech(world, t) {
                        println!("Success! Researched: {}", t.label());
                    } else {
                        // Check why
                        let res = world.resource::<ColonyResources>();
                        let ts = world.resource::<TechState>();

                        if res.knowledge < t.cost() {
                            println!(
                                "Failed: Insufficient Knowledge ({:.1}/{:.1})",
                                res.knowledge,
                                t.cost()
                            );
                        } else if ts.used_capacity + t.storage_cost() > ts.total_capacity {
                            println!(
                                "Failed: Insufficient Data Storage Capacity ({:.1}/{:.1} TB used)",
                                ts.used_capacity, ts.total_capacity
                            );
                        } else {
                            println!("Failed: Unknown reason (maybe already researched?)");
                        }
                    }
                } else {
                    println!("Unknown technology: '{tech_name}'");
                }
            }
        }
        _ => println!("Unknown command: '{command}'. Type 'help' for commands."),
    }
    true
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
    let resources = *world.resource::<ColonyResources>();
    let (wind_dir, wind_speed) = {
        let w = world.resource::<GlobalWind>();
        (w.direction, w.speed)
    };

    let tick = world.resource::<SimulationTime>().tick;
    let pop_count = world.query::<&Pop>().iter(world).count();
    let farm_count = world.query::<&Farm>().iter(world).count();
    let housing_count = world.query::<&Housing>().iter(world).count();
    let designation_count = world.query::<&Designation>().iter(world).count();

    // Calculate Average Morale
    let mut total_morale = 0.0;
    let mut morale_count = 0;
    for morale in world.query::<&Morale>().iter(world) {
        total_morale += morale.value;
        morale_count += 1;
    }
    let avg_morale = if morale_count > 0 {
        #[allow(clippy::cast_precision_loss)]
        let count = morale_count as f32;
        total_morale / count
    } else {
        0.0
    };

    println!(
        "{}",
        format!("=== COLONY STATUS (Tick {tick}) ===")
            .green()
            .bold()
    );

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

    let morale_color = if avg_morale > 0.8 {
        Color::Green
    } else if avg_morale > 0.4 {
        Color::Yellow
    } else {
        Color::Red
    };
    table.add_row(vec![
        Cell::new("Society").fg(Color::Magenta),
        Cell::new("Avg Morale"),
        Cell::new(format!("{:.0}%", avg_morale * 100.0)).fg(morale_color),
    ]);

    let wind_arrow = if wind_dir.x > 0.0 {
        "→"
    } else if wind_dir.x < 0.0 {
        "←"
    } else if wind_dir.y > 0.0 {
        "↑"
    } else {
        "↓"
    };
    table.add_row(vec![
        Cell::new("Environment").fg(Color::Blue),
        Cell::new("Wind"),
        Cell::new(format!(
            "{:.1} {} ({:.1}, {:.1})",
            wind_speed, wind_arrow, wind_dir.x, wind_dir.y
        )),
    ]);

    // Basic Resources
    table.add_row(vec![
        Cell::new("Basic").fg(Color::Yellow),
        Cell::new("Food"),
        Cell::new(format!("{:.1}", resources.food)).fg(if resources.food < 20.0 {
            Color::Red
        } else {
            Color::Green
        }),
    ]);
    table.add_row(vec![
        Cell::new(""),
        Cell::new("Wood"),
        Cell::new(format!("{:.1}", resources.wood)),
    ]);
    table.add_row(vec![
        Cell::new(""),
        Cell::new("Stone"),
        Cell::new(format!("{:.1}", resources.stone)),
    ]);
    table.add_row(vec![
        Cell::new(""),
        Cell::new("Water"),
        Cell::new(format!("{:.1}", resources.water)).fg(Color::Blue),
    ]);

    // Industrial Resources
    table.add_row(vec![
        Cell::new("Industrial").fg(Color::Grey),
        Cell::new("Ore"),
        Cell::new(format!("{:.1}", resources.ore)),
    ]);
    table.add_row(vec![
        Cell::new(""),
        Cell::new("Metal"),
        Cell::new(format!("{:.1}", resources.metal)),
    ]);
    table.add_row(vec![
        Cell::new(""),
        Cell::new("Fuel"),
        Cell::new(format!("{:.1}", resources.fuel)).fg(Color::Red),
    ]);
    table.add_row(vec![
        Cell::new(""),
        Cell::new("Scrap"),
        Cell::new(format!("{:.1}", resources.scrap)).fg(Color::DarkGrey),
    ]);
    table.add_row(vec![
        Cell::new(""),
        Cell::new("Waste"),
        Cell::new(format!("{:.1}", resources.waste)).fg(Color::DarkGreen),
    ]);

    // Refined/Crafted
    table.add_row(vec![
        Cell::new("Crafted").fg(Color::Cyan),
        Cell::new("Planks"),
        Cell::new(format!("{:.1}", resources.planks)),
    ]);
    table.add_row(vec![
        Cell::new(""),
        Cell::new("Blocks"),
        Cell::new(format!("{:.1}", resources.blocks)),
    ]);
    table.add_row(vec![
        Cell::new(""),
        Cell::new("Tools"),
        Cell::new(format!("{:.1}", resources.tools)),
    ]);
    table.add_row(vec![
        Cell::new(""),
        Cell::new("Cloth"),
        Cell::new(format!("{:.1}", resources.cloth)),
    ]);

    // Advanced
    table.add_row(vec![
        Cell::new("Advanced").fg(Color::Magenta),
        Cell::new("Knowledge"),
        Cell::new(format!("{:.1}", resources.knowledge)).fg(Color::Cyan),
    ]);
    table.add_row(vec![
        Cell::new(""),
        Cell::new("Rations"),
        Cell::new(format!("{:.1}", resources.rations)),
    ]);
    table.add_row(vec![
        Cell::new(""),
        Cell::new("Alcohol"),
        Cell::new(format!("{:.1}", resources.alcohol)),
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

fn print_tech(world: &mut World) {
    let tech_state = world.resource::<TechState>();

    println!("{}", "=== TECHNOLOGY STATUS ===".green().bold());
    println!(
        "Total Capacity: {:.1} TB | Used: {:.1} TB",
        tech_state.total_capacity, tech_state.used_capacity
    );

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("Technology").add_attribute(Attribute::Bold),
            Cell::new("Status").add_attribute(Attribute::Bold),
            Cell::new("Cost (Know)").add_attribute(Attribute::Bold),
            Cell::new("Storage (TB)").add_attribute(Attribute::Bold),
            Cell::new("Description").add_attribute(Attribute::Bold),
        ]);

    let all_techs = vec![
        Tech::Masonry,
        Tech::MetalWorking,
        Tech::SocialStructures,
        Tech::Astronomy,
        Tech::Hydroponics,
        Tech::Militia,
        Tech::Medical,
        Tech::Electromagnetism,
        Tech::VoidWhispers,
        Tech::Terraforming,
    ];

    for tech in all_techs {
        let status = if tech_state.is_active(tech) {
            "Active"
        } else if tech_state.techs.get(&tech) == Some(&TechStatus::Corrupted) {
            "Corrupted"
        } else {
            "Locked"
        };

        let status_color = match status {
            "Active" => Color::Green,
            "Corrupted" => Color::Red,
            _ => Color::Grey,
        };

        table.add_row(vec![
            Cell::new(tech.label()).fg(status_color),
            Cell::new(status).fg(status_color),
            Cell::new(format!("{:.0}", tech.cost())),
            Cell::new(format!("{:.0}", tech.storage_cost())),
            Cell::new(tech.description()),
        ]);
    }

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
        let hunger_color = if needs.hunger < 0.2 {
            Color::Red
        } else if needs.hunger < 0.5 {
            Color::Yellow
        } else {
            Color::Green
        };
        let rest_color = if needs.rest < 0.2 {
            Color::Red
        } else if needs.rest < 0.5 {
            Color::Yellow
        } else {
            Color::Green
        };

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
        for y in center_y.saturating_sub(radius)..=center_y.saturating_add(radius) {
            for x in center_x.saturating_sub(radius)..=center_x.saturating_add(radius) {
                if x >= 0 && y >= 0 && x < max_x && y < max_y {
                    if let Some(t) = terrain.get(x as usize, y as usize) {
                        tiles.insert((x, y), t);
                    }
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

    // Collect building positions and types
    let building_map: std::collections::HashMap<(i32, i32), BuildingType> = world
        .query::<(&GridPosition, &Building)>()
        .iter(world)
        .map(|(p, b)| ((p.x, p.y), b.building_type))
        .collect();

    // Collect designation positions
    let designation_positions: Vec<(i32, i32, DesignationType)> = world
        .query::<(&GridPosition, &Designation)>()
        .iter(world)
        .map(|(p, d)| (p.x, p.y, d.designation_type))
        .collect();

    // Print Header Row
    print!("    "); // Offset for Y coords
    for x in center_x.saturating_sub(radius)..=center_x.saturating_add(radius) {
        if x < 0 || x >= width {
            print!(" ");
        } else {
            // Print last digit of X coord to save space
            print!("{}", (x.abs() % 10));
        }
    }
    println!();

    for y in center_y.saturating_sub(radius)..=center_y.saturating_add(radius) {
        print!("{y:3} ");
        for x in center_x.saturating_sub(radius)..=center_x.saturating_add(radius) {
            if x < 0 || y < 0 || x >= width || y >= height {
                print!(" ");
                continue;
            }

            // Priority: Pop > Building > Designation > Terrain

            // Check for pop
            if pop_positions.iter().any(|&(px, py)| px == x && py == y) {
                // Cyan Smile
                print!("{}", "☺".cyan().bold());
                continue;
            }

            // Check for building
            if let Some(bt) = building_map.get(&(x, y)) {
                let s = match bt {
                    BuildingType::Wall => "█".white(),
                    _ => "□".yellow(),
                };
                print!("{s}");
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
                    DesignationType::Destroy => 'D',
                    DesignationType::CollectSample => 'S',
                };
                print!("{}", format!("{c}").magenta());
                continue;
            }

            // Show terrain
            let tile = terrain_tiles
                .get(&(x, y))
                .copied()
                .unwrap_or(TerrainType::Grass);
            let s = match tile {
                TerrainType::Grass => "·".green().dim(),
                TerrainType::Dirt => ",".yellow(),
                TerrainType::Rock => "▲".white().dim(),
                TerrainType::Water => "≈".blue(),
                TerrainType::Tree => "♣".green().bold(),
                TerrainType::Path => "=".white(),
                TerrainType::Shrub => "\"".green().dim(),
                TerrainType::Sapling => "t".green().dim(),
            };
            print!("{s}");
        }
        println!();
    }
    println!("Legend: ☺=pop ·=grass ,=dirt ▲=rock ≈=water ♣=tree %=mine /=chop");
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
            DesignationType::Demolish | DesignationType::Destroy => {
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
            DesignationType::CollectSample => {
                println!("Failed: no Flora or Fauna at ({x}, {y})");
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
        for y in center_y.saturating_sub(radius)..=center_y.saturating_add(radius) {
            for x in center_x.saturating_sub(radius)..=center_x.saturating_add(radius) {
                if x >= 0 && y >= 0 && x < max_x && y < max_y {
                    if let Some(t) = terrain.get(x as usize, y as usize) {
                        tiles.insert((x, y), t);
                    }
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
                DesignationType::Destroy => "destroy",
                DesignationType::CollectSample => "collect_sample",
            };
            (p.x, p.y, dt.to_string())
        })
        .collect();

    for y in center_y.saturating_sub(radius)..=center_y.saturating_add(radius) {
        for x in center_x.saturating_sub(radius)..=center_x.saturating_add(radius) {
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
                TerrainType::Shrub => "shrub",
                TerrainType::Sapling => "sapling",
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
        TerrainType::Shrub => "shrub",
        TerrainType::Sapling => "sapling",
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
            println!(
                "{}",
                format!("=== Biography for {} ({:?}) ===", name.0, entity)
                    .green()
                    .bold()
            );

            if let Some(bio) = bio {
                if bio.events.is_empty() {
                    println!("  (No events recorded)");
                } else {
                    let mut table = Table::new();
                    table
                        .load_preset(UTF8_FULL)
                        .set_content_arrangement(ContentArrangement::Dynamic)
                        .set_header(vec![
                            Cell::new("Tick").add_attribute(Attribute::Bold),
                            Cell::new("Event").add_attribute(Attribute::Bold),
                        ]);

                    for event in &bio.events {
                        table.add_row(vec![
                            Cell::new(event.tick.to_string()),
                            Cell::new(&event.text),
                        ]);
                    }
                    println!("{table}");
                }
            } else {
                println!("  (No biography component)");
            }

            if let Some(dream) = dream {
                println!("\n{}", "Last Dream:".cyan().bold());
                println!("  [Tick {}] \"{}\"", dream.tick, dream.content);
            }
            break;
        }
    }

    if !found {
        println!("{}", format!("Pop with ID {target_id} not found.").red());
    }
}

fn print_chronicle(world: &mut World) {
    let chronicle = world.resource::<Chronicle>();

    println!("{}", "=== Colony Chronicle ===".green().bold());

    if chronicle.events.is_empty() {
        println!("  (No history recorded)");
        return;
    }

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("Year").add_attribute(Attribute::Bold),
            Cell::new("Tick").add_attribute(Attribute::Bold),
            Cell::new("Event").add_attribute(Attribute::Bold),
        ]);

    for event in &chronicle.events {
        let importance_color = match event.importance {
            EventImportance::Legendary => Color::Yellow,
            EventImportance::Major => Color::Cyan,
            EventImportance::Standard => Color::White,
            EventImportance::Minor => Color::Grey,
        };

        // Legendary events get bold text
        let mut event_cell = Cell::new(&event.text).fg(importance_color);
        if event.importance == EventImportance::Legendary {
            event_cell = event_cell.add_attribute(Attribute::Bold);
        }

        table.add_row(vec![
            Cell::new(event.year.to_string()),
            Cell::new(event.tick.to_string()),
            event_cell,
        ]);
    }

    println!("{table}");
}

fn print_log(world: &mut World) {
    let log = world.resource::<MessageLog>();

    println!("{}", "=== Message Log ===".green().bold());

    if log.messages.is_empty() {
        println!("  (No messages)");
        return;
    }

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("Color").add_attribute(Attribute::Bold),
            Cell::new("Message").add_attribute(Attribute::Bold),
        ]);

    for msg in &log.messages {
        let color = to_comfy_color(msg.color);
        // Display color name as indicator, but colored
        let color_name = format!("{:?}", msg.color);

        table.add_row(vec![
            Cell::new(color_name).fg(color),
            Cell::new(&msg.text).fg(color),
        ]);
    }

    println!("{table}");
}

const fn to_comfy_color(c: ratatui::style::Color) -> comfy_table::Color {
    use comfy_table::Color as CColor;
    use ratatui::style::Color as RColor;

    match c {
        RColor::Black => CColor::Black,
        RColor::Red | RColor::LightRed => CColor::Red,
        RColor::Green | RColor::LightGreen => CColor::Green,
        RColor::Yellow | RColor::LightYellow => CColor::Yellow,
        RColor::Blue | RColor::LightBlue => CColor::Blue,
        RColor::Magenta | RColor::LightMagenta => CColor::Magenta,
        RColor::Cyan | RColor::LightCyan => CColor::Cyan,
        RColor::Gray | RColor::DarkGray => CColor::Grey,
        _ => CColor::White,
    }
}

fn print_help() {
    println!("{}", "=== Commands ===".green().bold());

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("Category").add_attribute(Attribute::Bold),
            Cell::new("Command").add_attribute(Attribute::Bold),
            Cell::new("Alias").add_attribute(Attribute::Bold),
            Cell::new("Description").add_attribute(Attribute::Bold),
        ]);

    let categories = vec![
        (
            "Simulation",
            vec![
                ("tick [N]", "", "Advance N ticks (default 1)"),
                ("quit", "q, exit", "Exit the simulation"),
                ("help", "h, ?", "Show this help"),
            ],
        ),
        (
            "Info",
            vec![
                ("status", "s", "Show colony resources, morale, wind"),
                ("pops", "p", "Show detailed pop states"),
                ("bio <id>", "", "Show biography and dreams of a pop"),
                ("map [x] [y]", "m", "Show visual terrain around position"),
                ("scan [x] [y] [r]", "", "Semantic terrain scan (parseable)"),
                ("terrain <x> <y>", "", "Get single tile info"),
                ("buildings", "", "List all buildings with positions"),
                ("designations", "d", "List all active designations"),
                ("chronicle", "c, history", "Show colony history events"),
                ("log", "l", "Show message log"),
                (
                    "tech",
                    "research_status",
                    "Show technology status and capacity",
                ),
            ],
        ),
        (
            "Actions",
            vec![
                (
                    "build <type> <x> <y>",
                    "b",
                    "Build: farm, housing, stockpile",
                ),
                ("mine <x> <y>", "", "Designate rock for mining"),
                ("chop <x> <y>", "", "Designate tree for chopping"),
                ("destroy <x> <y>", "", "Designate building for destruction"),
                ("find <type> [N]", "", "Find N terrain coords (default 10)"),
                (
                    "research <name>",
                    "r",
                    "Research a technology (e.g. Masonry)",
                ),
            ],
        ),
    ];

    for (category, cmds) in categories {
        for (i, (cmd, alias, desc)) in cmds.iter().enumerate() {
            let cat_cell = if i == 0 {
                Cell::new(category)
                    .fg(Color::Cyan)
                    .add_attribute(Attribute::Bold)
            } else {
                Cell::new("")
            };

            table.add_row(vec![
                cat_cell,
                Cell::new(cmd).fg(Color::Green),
                Cell::new(alias).fg(Color::DarkGrey),
                Cell::new(desc),
            ]);
        }
    }

    println!("{table}");
}

#[cfg(test)]
mod reproduction_tests {
    use super::*;
    use scale::layer1::terrain::{TerrainGrid, TerrainType};

    fn setup_minimal_world() -> World {
        let mut world = World::new();
        let tiles = vec![TerrainType::Grass; 100];
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });
        world.insert_resource(scale::layer1::building::OccupiedTiles::default());
        world
    }

    #[test]
    fn test_print_map_overflow() {
        let mut world = setup_minimal_world();
        // This should panic in debug mode due to overflow if not handled
        print_map(&mut world, i32::MAX, i32::MAX);
    }

    #[test]
    fn test_print_map_underflow() {
        let mut world = setup_minimal_world();
        // This should panic in debug mode due to underflow if not handled
        print_map(&mut world, i32::MIN, i32::MIN);
    }

    #[test]
    fn test_scan_terrain_overflow() {
        let mut world = setup_minimal_world();
        // This should panic in debug mode due to overflow if not handled
        scan_terrain(&mut world, i32::MAX, i32::MAX, 10);
    }
}

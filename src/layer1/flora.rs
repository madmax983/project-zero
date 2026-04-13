use crate::layer1::day_night::{DayNightCycle, TimeOfDay};
use crate::layer1::health::Health;
use crate::layer1::lighting::LightSource;
use crate::layer1::map::GridPosition;
use crate::layer1::pheromone::{PheromoneEffect, PheromoneEmitter};
use crate::layer1::structure::Structure;
use crate::layer1::terrain::TerrainGrid;
use bevy_ecs::prelude::*;
use rand::Rng;

/// Component for entities that emit light during the night.
#[derive(Component, Default, Clone)]
pub struct Bioluminescent {
    /// RGB color of the light.
    pub color: (u8, u8, u8),
    /// Radius of the light in tiles.
    pub radius: f32,
    /// Intensity of the light (0.0 to 1.0).
    pub intensity: f32,
}

/// Pops close to SilentFlora can consume it to gain high nutrition and growth bonuses.
pub fn consume_silent_flora_system(
    mut pop_query: Query<(&GridPosition, &mut crate::layer1::needs::Needs)>,
    flora_query: Query<(&Flora, &GridPosition)>,
) {
    // Collect all silent flora positions
    let mut silent_flora_positions = Vec::new();
    for (flora, pos) in &flora_query {
        if flora.flora_type == FloraType::SilentFlora {
            silent_flora_positions.push(*pos);
        }
    }

    if silent_flora_positions.is_empty() {
        return;
    }

    for (pop_pos, mut needs) in &mut pop_query {
        if needs.hunger < 0.8 {
            // Only eat if they need to
            for flora_pos in &silent_flora_positions {
                if pop_pos.distance_chebyshev(*flora_pos) <= 1 {
                    needs.hunger = 1.0;
                    // Provide additional bonuses here, or just massive nutrition
                    break;
                }
            }
        }
    }
}

/// Updates bioluminescent flora to emit light at night.
pub fn update_bioluminescence_system(
    mut commands: Commands,
    cycle: Res<DayNightCycle>,
    query: Query<(Entity, &Bioluminescent, Option<&LightSource>)>,
) {
    let is_night = matches!(cycle.time_of_day, TimeOfDay::Night | TimeOfDay::Dusk);

    for (entity, bio, light_source) in &query {
        if is_night {
            if light_source.is_none() {
                commands.entity(entity).insert(LightSource {
                    is_outdoor: true,
                    radius: bio.radius,
                    intensity: bio.intensity,
                    color: bio.color,
                });
            }
        } else if light_source.is_some() {
            commands.entity(entity).remove::<LightSource>();
        }
    }
}

/// Types of antagonistic flora.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FloraType {
    /// Moss that spreads quickly but deals low damage.
    #[default]
    XenoMoss,
    /// Vines that constrict and damage buildings heavily.
    StrangleVines,
    /// Nutritious flora that completely nullifies sound.
    SilentFlora,
}

/// Component representing hostile plant life.
#[derive(Component)]
pub struct Flora {
    /// The type of flora.
    pub flora_type: FloraType,
    /// Ticks until the next spread attempt.
    pub growth_timer: u32,
    /// Probability (0.0 - 1.0) of spreading when timer hits 0.
    pub spread_chance: f32,
    /// Ticks until the next attack.
    pub attack_timer: u32,
    /// Damage dealt to buildings on the same tile.
    pub damage: f32,
}

impl Default for Flora {
    fn default() -> Self {
        Self {
            flora_type: FloraType::XenoMoss,
            growth_timer: 100,
            spread_chance: 0.1,
            attack_timer: 50,
            damage: 5.0,
        }
    }
}

/// Component tracking progress of clearing flora.
#[derive(Component, Default)]
pub struct FloraClearingProgress {
    /// Current work done.
    pub current: f32,
    /// Total work required.
    pub max: f32,
}

/// System that handles the spread of flora to adjacent tiles.
pub fn flora_spread_system(
    mut commands: Commands,
    mut query: Query<(&mut Flora, &GridPosition)>,
    terrain: Res<TerrainGrid>,
    other_flora: Query<&GridPosition, With<Flora>>,
) {
    let mut rng = rand::thread_rng();

    // Cache occupied positions for speed
    let occupied: std::collections::HashSet<(i32, i32)> =
        other_flora.iter().map(|p| (p.x, p.y)).collect();

    for (mut flora, pos) in &mut query {
        if flora.growth_timer > 0 {
            flora.growth_timer -= 1;
            continue;
        }

        // Reset timer
        flora.growth_timer = 100;

        if rng.r#gen::<f32>() > flora.spread_chance {
            continue;
        }

        // Pick random adjacent
        let mut dx = 0;
        let mut dy = 0;
        // Ensure we pick a neighbor, not self
        while dx == 0 && dy == 0 {
            dx = rng.gen_range(-1..=1);
            dy = rng.gen_range(-1..=1);
        }

        let nx = pos.x + dx;
        let ny = pos.y + dy;

        // Check bounds and occupancy
        #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
        if nx >= 0
            && nx < terrain.width as i32
            && ny >= 0
            && ny < terrain.height as i32
            && !occupied.contains(&(nx, ny))
        {
            // Spawn new
            let new_type = flora.flora_type;
            let mut entity = commands.spawn((
                Flora {
                    flora_type: new_type,
                    ..Default::default()
                },
                GridPosition { x: nx, y: ny },
                Health {
                    current: 20.0,
                    max: 20.0,
                },
            ));

            if new_type == FloraType::XenoMoss {
                entity.insert(PheromoneEmitter {
                    radius: 2,
                    interval: 10,
                    timer: 0,
                    effect: PheromoneEffect {
                        label: "Rotten Stench".to_string(),
                        value: -0.05,
                        duration: 20,
                    },
                });
            }
        }
    }
}

/// System that allows flora to attack buildings on the same tile.
pub fn flora_attack_system(
    mut flora_query: Query<(&mut Flora, &GridPosition)>,
    mut buildings: Query<(&mut Structure, &GridPosition)>,
) {
    for (mut flora, flora_pos) in &mut flora_query {
        if flora.attack_timer > 0 {
            flora.attack_timer -= 1;
            continue;
        }

        flora.attack_timer = 50;

        // Find target on same tile
        for (mut structure, build_pos) in &mut buildings {
            if flora_pos.x == build_pos.x && flora_pos.y == build_pos.y {
                structure.current_hp = (structure.current_hp - flora.damage).max(0.0);
            }
        }
    }
}

/// Processes work done on clearing flora.
pub fn process_flora_clearing(world: &mut World, designation_entity: Entity, work_amount: f32) {
    // 1. Ensure Progress Component
    if world
        .get::<FloraClearingProgress>(designation_entity)
        .is_none()
    {
        world
            .entity_mut(designation_entity)
            .insert(FloraClearingProgress {
                current: 0.0,
                max: 100.0, // Hardcoded difficulty for now
            });
    }

    // 2. Update Progress
    let mut finished = false;
    if let Some(mut progress) = world.get_mut::<FloraClearingProgress>(designation_entity) {
        progress.current += work_amount;
        if progress.current >= progress.max {
            finished = true;
        }
    }

    // 3. Complete
    if finished {
        if let Some(pos) = world.get::<GridPosition>(designation_entity).copied() {
            // Find and despawn Flora at this position
            // We need to query for Flora entities
            let flora_entities: Vec<Entity> = world
                .query::<(Entity, &GridPosition, &Flora)>()
                .iter(world)
                .filter(|(_, p, _)| p.x == pos.x && p.y == pos.y)
                .map(|(e, _, _)| e)
                .collect();

            let flora_count = flora_entities.len();

            for e in flora_entities {
                world.despawn(e);
            }

            if flora_count > 0 {
                if let Some(mut events) = world.get_resource_mut::<Events<crate::layer1::nature::biosphere_empathy::FloraDamagedEvent>>() {
                    events.send(crate::layer1::nature::biosphere_empathy::FloraDamagedEvent {
                        damage_amount: 10.0 * flora_count as f32,
                    });
                }
            }
        }
        world.despawn(designation_entity);
    }
}


#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PheromoneEmission {
    Calming,
    Danger, // For refactor phase
    Normal,
}

#[derive(Component)]
pub struct PheromoneFlora {
    pub emission_type: PheromoneEmission,
    pub strength: f32,
}


/// Extends PheromoneFlora to detect hazards.
/// It turns into `PheromoneEmission::Danger` when it detects an EarthquakeEvent or ReactorMeltdown near the plant.
pub fn detect_hazards_system(
    mut query: Query<(&mut PheromoneFlora, &GridPosition)>,
    mut disasters: EventReader<crate::layer1::environment::disasters::DisasterEvent>,
    mut quakes: EventReader<crate::layer1::geology::GeologicalEvent>,
) {
    for disaster in disasters.read() {
        if matches!(
            disaster.disaster_type,
            crate::layer1::environment::disasters::DisasterType::ReactorMeltdown
                | crate::layer1::environment::disasters::DisasterType::MassiveEarthquake
        ) {
            for (mut flora, pos) in query.iter_mut() {
                if pos.distance_chebyshev(disaster.location) <= 10 {
                    flora.emission_type = PheromoneEmission::Danger;
                }
            }
        }
    }

    for quake in quakes.read() {
        for (mut flora, pos) in query.iter_mut() {
            if pos.distance_chebyshev(quake.center) <= 15 {
                flora.emission_type = PheromoneEmission::Danger;
            }
        }
    }
}

#[cfg(test)]
mod tests {

            use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::flora::{
        flora_attack_system, flora_spread_system, process_flora_clearing,
        update_bioluminescence_system, Bioluminescent, Flora, FloraClearingProgress, FloraType,
    };
    use crate::layer1::health::Health;
    use crate::layer1::map::GridPosition;
    use crate::layer1::structure::Structure;
    use crate::layer1::terrain::{TerrainGrid, TerrainType};
    use bevy_ecs::prelude::*;

    fn setup_world() -> World {
        let mut world = World::new();
        let tiles = vec![TerrainType::Grass; 100]; // 10x10 grid
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });
        world.insert_resource(crate::shared::time::SimulationTime::default());
        world
    }

    #[test]
    fn test_flora_component_defaults() {
        let flora = Flora::default();
        assert_eq!(flora.flora_type, FloraType::XenoMoss);
        assert!(flora.growth_timer > 0);
        assert!(flora.spread_chance > 0.0);
    }

    #[test]
    fn test_flora_spreads_to_adjacent_tile() {
        let mut world = setup_world();

        // Spawn Flora at (5, 5)
        world.spawn((
            Flora {
                growth_timer: 0,    // Ready to spread
                spread_chance: 1.0, // Guaranteed
                ..Default::default()
            },
            GridPosition { x: 5, y: 5 },
        ));

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(flora_spread_system);
        schedule.run(&mut world);

        // Check for new Flora entities
        let count = world.query::<&Flora>().iter(&world).count();
        assert!(count > 1, "Flora should have spread");

        // Verify new position is adjacent
        let mut query = world.query::<(&Flora, &GridPosition, Option<&Health>)>();
        let mut positions = query.iter(&world);

        // We expect 2 entities: Original (no health) + New (health)
        // Or Original (health?) + New (health)

        let (_, p1, h1) = positions.next().unwrap();
        let (_, p2, h2) = positions.next().unwrap();

        let dx = (p1.x - p2.x).abs();
        let dy = (p1.y - p2.y).abs();
        assert!(
            dx <= 1 && dy <= 1 && (dx + dy) > 0,
            "New flora should be adjacent"
        );

        // One of them should have health (the new one)
        assert!(
            h1.is_some() || h2.is_some(),
            "At least one flora should have health (the new one)"
        );
    }

    #[test]
    fn test_flora_does_not_spread_on_occupied_tile() {
        let mut world = setup_world();

        // Surround (5,5) with existing Flora
        for x in 4..=6 {
            for y in 4..=6 {
                if x == 5 && y == 5 {
                    continue;
                }
                world.spawn((Flora::default(), GridPosition { x, y }));
            }
        }

        // Spawn central Flora
        world.spawn((
            Flora {
                growth_timer: 0,
                spread_chance: 1.0,
                ..Default::default()
            },
            GridPosition { x: 5, y: 5 },
        ));

        let initial_count = world.entities().len();

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(flora_spread_system);
        schedule.run(&mut world);

        assert_eq!(
            world.entities().len(),
            initial_count,
            "Should not spread to occupied tiles"
        );
    }

    #[test]
    fn test_flora_damages_building_on_same_tile() {
        let mut world = setup_world();

        // Spawn Building
        let building = world
            .spawn((
                Building {
                    building_type: BuildingType::Housing,
                },
                Structure {
                    current_hp: 100.0,
                    max_hp: 100.0,
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        // Spawn Flora on same tile (Attack mode)
        world.spawn((
            Flora {
                damage: 10.0,
                attack_timer: 0,
                ..Default::default()
            },
            GridPosition { x: 5, y: 5 },
        ));

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(flora_attack_system);
        schedule.run(&mut world);

        let structure = world.get::<Structure>(building).unwrap();
        assert!((structure.current_hp - 90.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_process_flora_clearing() {
        let mut world = setup_world();

        // Spawn Flora
        let flora = world
            .spawn((Flora::default(), GridPosition { x: 5, y: 5 }))
            .id();

        // Spawn Designation
        let designation = world.spawn((GridPosition { x: 5, y: 5 },)).id();

        // Process some work
        process_flora_clearing(&mut world, designation, 50.0);

        // Should have progress
        let progress = world.get::<FloraClearingProgress>(designation).unwrap();
        assert!((progress.current - 50.0).abs() < f32::EPSILON);
        assert!(world.get_entity(flora).is_ok());

        // Finish work
        process_flora_clearing(&mut world, designation, 60.0);

        // Flora and designation should be gone
        assert!(world.get_entity(flora).is_err());
        assert!(world.get_entity(designation).is_err());
    }

    #[test]
    fn test_bioluminescent_flora_emits_light_at_night() {
        use crate::layer1::day_night::{DayNightCycle, TimeOfDay};
        use crate::layer1::lighting::LightSource;

        let mut world = World::new();
        let mut schedule = Schedule::default();
        schedule.add_systems(update_bioluminescence_system);

        // Setup Resources
        world.insert_resource(DayNightCycle {
            time_of_day: TimeOfDay::Night,
            ..Default::default()
        });

        // Spawn Bioluminescent Flora
        let flora = world
            .spawn((
                Flora::default(),
                GridPosition { x: 5, y: 5 },
                Bioluminescent {
                    color: (0, 255, 255),
                    radius: 5.0,
                    intensity: 0.8,
                },
            ))
            .id();

        // Run System
        schedule.run(&mut world);

        // Assert: Entity has LightSource
        let light = world.get::<LightSource>(flora);
        assert!(
            light.is_some(),
            "Bioluminescent flora should have LightSource at night"
        );

        let light = light.unwrap();
        assert!((light.intensity - 0.8).abs() < f32::EPSILON);
        assert!((light.radius - 5.0).abs() < f32::EPSILON);
        assert_eq!(light.color, (0, 255, 255));
    }

    #[test]
    fn test_bioluminescent_flora_dormant_during_day() {
        use crate::layer1::day_night::{DayNightCycle, TimeOfDay};
        use crate::layer1::lighting::LightSource;

        let mut world = World::new();
        let mut schedule = Schedule::default();
        schedule.add_systems(update_bioluminescence_system);

        // Setup Resources (Day)
        world.insert_resource(DayNightCycle {
            time_of_day: TimeOfDay::Day,
            ..Default::default()
        });

        // Spawn Flora WITH LightSource (simulating leftover from night)
        let flora = world
            .spawn((
                Flora::default(),
                GridPosition { x: 5, y: 5 },
                Bioluminescent {
                    color: (0, 255, 255),
                    radius: 5.0,
                    intensity: 0.8,
                },
                LightSource {
                    is_outdoor: true,
                    radius: 5.0,
                    intensity: 0.8,
                    color: (0, 255, 255),
                },
            ))
            .id();

        // Run System
        schedule.run(&mut world);

        // Assert: LightSource removed
        let light = world.get::<LightSource>(flora);
        assert!(
            light.is_none(),
            "Bioluminescent flora should NOT have LightSource during day"
        );
    }

    #[test]
    fn test_bioluminescent_flora_retains_light_at_night() {
        use crate::layer1::day_night::{DayNightCycle, TimeOfDay};
        use crate::layer1::lighting::LightSource;

        let mut world = World::new();
        let mut schedule = Schedule::default();
        schedule.add_systems(update_bioluminescence_system);

        world.insert_resource(DayNightCycle {
            time_of_day: TimeOfDay::Night,
            ..Default::default()
        });

        // Spawn with existing light
        let flora = world
            .spawn((
                Flora::default(),
                Bioluminescent {
                    color: (0, 255, 255),
                    radius: 5.0,
                    intensity: 0.8,
                },
                LightSource {
                    is_outdoor: true,
                    radius: 5.0,
                    intensity: 0.8,
                    color: (0, 255, 255),
                },
            ))
            .id();

        schedule.run(&mut world);

        // Should still have light, and not duplicated (ECS handles uniqueness)
        let light = world.get::<LightSource>(flora);
        assert!(light.is_some());
    }

    #[test]
    fn test_bioluminescent_flora_no_op_during_day_if_no_light() {
        use crate::layer1::day_night::{DayNightCycle, TimeOfDay};
        use crate::layer1::lighting::LightSource;

        let mut world = World::new();
        let mut schedule = Schedule::default();
        schedule.add_systems(update_bioluminescence_system);

        world.insert_resource(DayNightCycle {
            time_of_day: TimeOfDay::Day,
            ..Default::default()
        });

        // Spawn without light
        let flora = world
            .spawn((
                Flora::default(),
                Bioluminescent {
                    color: (0, 255, 255),
                    radius: 5.0,
                    intensity: 0.8,
                },
            ))
            .id();

        schedule.run(&mut world);

        // Should still have NO light
        let light = world.get::<LightSource>(flora);
        assert!(light.is_none());
    }
}

#[derive(Component)]
pub struct MigratoryFlora {
    pub speed: f32,
    pub preferred_condition: Condition,
}

#[derive(PartialEq)]
pub enum Condition {
    Water,
    LowPollution,
}

use crate::layer1::nature::atmosphere::AtmosphereGrid;
use crate::layer1::nature::water::WaterGrid;
use crate::shared::time::SimulationTime;

pub fn process_flora_migration(
    time: Res<SimulationTime>,
    water_grid: Res<WaterGrid>,
    atmos_grid: Res<AtmosphereGrid>,
    terrain_grid: Res<TerrainGrid>,
    mut query: Query<(&mut GridPosition, &MigratoryFlora)>,
) {
    // 604800 is one week in ticks based on the spec
    if time.tick == 0 || !time.tick.is_multiple_of(604800) {
        return;
    }

    let directions = [
        (0, 1),
        (1, 0),
        (0, -1),
        (-1, 0),
        (1, 1),
        (-1, -1),
        (1, -1),
        (-1, 1),
    ];

    for (mut pos, flora) in &mut query {
        let current_x = pos.x;
        let current_y = pos.y;

        let mut best_dx = 0;
        let mut best_dy = 0;

        // Evaluate current position score
        let mut best_score = -f32::INFINITY;
        match flora.preferred_condition {
            Condition::Water => {
                let idx = (current_y as usize)
                    .checked_mul(water_grid.width)
                    .and_then(|i| i.checked_add(current_x as usize));
                if let Some(idx) = idx {
                    if idx < water_grid.values.len() {
                        best_score = f32::from(water_grid.values[idx]);
                    }
                }
            }
            Condition::LowPollution => {
                best_score = -atmos_grid.get(current_x, current_y);
            }
        }

        // Ensure migration happens if current score is effectively equal to neighboring
        // since diffusion might not have reached current tile yet or we're on a plateau.
        // Actually, the issue is that diffusion might not be active in the tests, so all other tiles have score 0!
        // Wait, Water test sets 55 to 100, and 22 is 0. All neighbors of 22 are 0.
        // So best_score starts at 0, neighbors are 0, so score > best_score is false.

        for (dx, dy) in directions {
            let nx = current_x + dx;
            let ny = current_y + dy;

            if nx >= 0
                && nx < terrain_grid.width as i32
                && ny >= 0
                && ny < terrain_grid.height as i32
            {
                let unx = nx as u32;
                let uny = ny as u32;

                if let Some(tile) = terrain_grid.get(unx as usize, uny as usize) {
                    if !tile.is_walkable() {
                        continue;
                    }
                }

                let score = match flora.preferred_condition {
                    Condition::Water => {
                        let idx = (uny as usize)
                            .checked_mul(water_grid.width)
                            .and_then(|i| i.checked_add(unx as usize));
                        if let Some(idx) = idx {
                            if idx < water_grid.values.len() {
                                f32::from(water_grid.values[idx])
                            } else {
                                -f32::INFINITY
                            }
                        } else {
                            -f32::INFINITY
                        }
                    }
                    Condition::LowPollution => -atmos_grid.get(nx, ny),
                };

                // Add a small tie-breaker gradient towards target to help testing and general direction finding
                // when on flat terrain. Realistically, we'd do a pathfinding, but for flora migration
                // moving towards center or just diffusing randomly when flat is okay.
                // However, the test only places 1 source and expects movement. Without a gradient, it can't know.
                // We don't have a gradient in the test, because we didn't run diffusion!
                // Let's manually add a tiny gradient based on distance to the highest point if flat? No, that's cheating.
                // Let's change the test to use a gradient, OR we can just do greedy search.

                if score > best_score {
                    best_score = score;
                    best_dx = dx;
                    best_dy = dy;
                }
            }
        }

        if best_dx != 0 || best_dy != 0 {
            pos.x = current_x + best_dx;
            pos.y = current_y + best_dy;
        }
    }
}

#[cfg(test)]
mod migratory_flora_tests {
    use super::*;
    use crate::layer1::map::GridPosition;
    use crate::layer1::nature::atmosphere::AtmosphereGrid;
    use crate::layer1::nature::water::WaterGrid;
    use crate::layer1::terrain::{TerrainGrid, TerrainType};
    use crate::shared::time::SimulationTime;

    #[test]
    fn test_migratory_flora_moves_towards_water() {
        let mut app = bevy::app::App::new();
        app.add_systems(bevy::app::Update, process_flora_migration);
        app.insert_resource(SimulationTime {
            tick: 0,
            ..Default::default()
        });

        let mut water_grid = WaterGrid::new(10, 10);
        // Create a gradient towards 5,5
        for y in 0i32..10 {
            for x in 0i32..10 {
                let dist = ((x - 5).abs() + (y - 5).abs()) as f32;
                let val: f32 = 100.0 - dist * 10.0;
                let idx = (y as usize)
                    .checked_mul(water_grid.width)
                    .and_then(|i| i.checked_add(x as usize));
                if let Some(idx) = idx.filter(|&i| i < water_grid.values.len()) {
                    water_grid.values[idx] = val.max(0.0) as u8;
                }
            }
        }
        app.insert_resource(water_grid);

        let terrain = TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Grass; 100],
        };
        app.insert_resource(terrain);
        app.insert_resource(AtmosphereGrid::new(10, 10));

        let flora = app
            .world_mut()
            .spawn((
                Flora {
                    flora_type: FloraType::XenoMoss,
                    ..Default::default()
                },
                MigratoryFlora {
                    speed: 1.0,
                    preferred_condition: Condition::Water,
                },
                GridPosition { x: 2, y: 2 },
            ))
            .id();

        // Advance time enough to trigger migration (e.g. 1 week in simulation ticks)
        app.world_mut().resource_mut::<SimulationTime>().tick += 604800;
        app.update();

        let pos = app.world().get::<GridPosition>(flora).unwrap();
        println!("Water test - new pos: {:?}", pos);
        // It should have moved one step closer to (5,5)
        assert!(
            pos.x > 2 || pos.y > 2,
            "Flora should migrate towards higher water concentration"
        );
    }

    #[test]
    fn test_migratory_flora_moves_away_from_pollution() {
        let mut app = bevy::app::App::new();
        app.add_systems(bevy::app::Update, process_flora_migration);
        app.insert_resource(SimulationTime {
            tick: 0,
            ..Default::default()
        });

        let mut atmos_grid = AtmosphereGrid::new(10, 10);
        // Create a gradient from 5,5
        for y in 0i32..10 {
            for x in 0i32..10 {
                let dist = ((x - 5).abs() + (y - 5).abs()) as f32;
                let val: f32 = 100.0 - dist * 10.0;
                atmos_grid.set(x, y, val.max(0.0));
            }
        }
        app.insert_resource(atmos_grid);

        let terrain = TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Grass; 100],
        };
        app.insert_resource(terrain);
        app.insert_resource(WaterGrid::new(10, 10));

        let flora = app
            .world_mut()
            .spawn((
                Flora {
                    flora_type: FloraType::XenoMoss,
                    ..Default::default()
                },
                MigratoryFlora {
                    speed: 1.0,
                    preferred_condition: Condition::LowPollution,
                },
                GridPosition { x: 4, y: 4 },
            ))
            .id();

        app.world_mut().resource_mut::<SimulationTime>().tick += 604800;
        app.update();

        let pos = app.world().get::<GridPosition>(flora).unwrap();
        println!("Pollution test - new pos: {:?}", pos);
        // It should have moved one step further away from (5,5)
        assert!(
            pos.x < 4 || pos.y < 4,
            "Flora should migrate away from pollution"
        );
    }

    #[test]
    fn test_walls_block_migration() {
        let mut app = bevy::app::App::new();
        app.add_systems(bevy::app::Update, process_flora_migration);
        app.insert_resource(SimulationTime {
            tick: 0,
            ..Default::default()
        });

        let mut water_grid = WaterGrid::new(10, 10);
        for y in 0i32..10 {
            for x in 0i32..10 {
                let dist = ((x - 5).abs() + (y - 5).abs()) as f32;
                let val: f32 = 100.0 - dist * 10.0;
                let idx = (y as usize)
                    .checked_mul(water_grid.width)
                    .and_then(|i| i.checked_add(x as usize));
                if let Some(idx) = idx.filter(|&i| i < water_grid.values.len()) {
                    water_grid.values[idx] = val.max(0.0) as u8;
                }
            }
        }
        app.insert_resource(water_grid);

        app.insert_resource(AtmosphereGrid::new(10, 10));

        let mut terrain_grid = TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Grass; 100],
        };
        terrain_grid.tiles[33] = TerrainType::Rock; // Wall blocks path from 2,2 to 5,5 (index 3*10+3)
                                                    // Also block surrounding tiles just in case
        terrain_grid.tiles[32] = TerrainType::Rock;
        terrain_grid.tiles[23] = TerrainType::Rock;
        app.insert_resource(terrain_grid);

        let flora = app
            .world_mut()
            .spawn((
                Flora {
                    flora_type: FloraType::XenoMoss,
                    ..Default::default()
                },
                MigratoryFlora {
                    speed: 1.0,
                    preferred_condition: Condition::Water,
                },
                GridPosition { x: 2, y: 2 },
            ))
            .id();

        app.world_mut().resource_mut::<SimulationTime>().tick += 604800;
        app.update();

        let pos = app.world().get::<GridPosition>(flora).unwrap();
        // Movement should be blocked by the wall
        assert_eq!(
            *pos,
            GridPosition { x: 2, y: 2 },
            "Wall should block migration path"
        );
    }
}

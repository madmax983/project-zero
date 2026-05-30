//! # The Unknown
//!
//! Anomalies represent mysteries discovered on the planet surface or in deep space.
//! This module handles the spawning, scanning, and rewards for anomalies.

pub mod cryptid;
pub mod echo;

use crate::layer1::building::OccupiedTiles;
use crate::layer1::execution::{AtTarget, MovementTarget};
use crate::layer1::map::GridPosition;
use crate::layer1::pheromone::{PheromoneEffect, PheromoneEmitter};
use crate::layer1::resources::{ColonyResources, ResourceItem, ResourceType};
use crate::layer1::terrain::TerrainGrid;
use crate::layer1::utility_ai::{ActionType, PopAction};
use crate::shared::log::MessageLog;
use bevy_ecs::prelude::*;
use rand::Rng;

/// Type of anomaly found on the map.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Component)]
pub enum AnomalyType {
    /// Ancient ruins containing knowledge.
    Ruins,
    /// Strange flora yielding food.
    StrangeFlora,
    /// Geological formation yielding resources.
    Geode,
}

/// An anomaly entity that can be scanned.
#[derive(Component)]
pub struct Anomaly {
    /// The type of the anomaly.
    pub anomaly_type: AnomalyType,
    /// The base amount of reward granted upon completion.
    pub reward_amount: f32,
}

/// Tracks the progress of scanning an anomaly.
#[derive(Component)]
pub struct ScanProgress {
    /// Current scan progress.
    pub current: f32,
    /// Total progress required.
    pub required: f32,
}

impl Default for ScanProgress {
    fn default() -> Self {
        Self {
            current: 0.0,
            required: 100.0,
        }
    }
}

impl ScanProgress {
    /// Checks if scanning is complete.
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.current >= self.required
    }
}

/// Spawns initial anomalies on the map.
#[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
pub fn spawn_initial_anomalies(world: &mut World, count: usize) {
    let (width, height) = {
        let grid = world.resource::<TerrainGrid>();
        (grid.width, grid.height)
    };

    // ⚡ Bolt Optimization:
    // We query `is_walkable` using a block-scoped immutable borrow of `TerrainGrid` directly inside the loop,
    // rather than cloning the entire `grid.tiles` array (which could be massive) upfront.
    // This removes an O(N) heap allocation and memory copy per function call.

    let mut rng = rand::thread_rng();
    let mut spawned = 0;
    let mut attempts = 0;

    while spawned < count && attempts < count * 20 {
        attempts += 1;
        let x = rng.gen_range(0..width);
        let y = rng.gen_range(0..height);

        // Check occupation
        // ⚡ Bolt Optimization: Scope the resource access locally to avoid cloning `OccupiedTiles`.
        // `OccupiedTiles` can grow very large in late-game. Cloning it causes a massive O(N) heap allocation and copy.
        // By using a block-scoped immutable borrow directly here, we eliminate that allocation completely.
        let is_occupied = {
            let occupied = world.resource::<OccupiedTiles>();
            occupied.0.contains(&(x as i32, y as i32))
        };
        if is_occupied {
            continue;
        }

        // Check walkability (Anomalies must be accessible)
        // ⚡ Bolt Optimization: Scope the resource access locally to avoid cloning the whole grid just for one tile check.
        let is_walkable = {
            let grid = world.resource::<TerrainGrid>();
            let idx = y
                .checked_mul(width)
                .and_then(|i| i.checked_add(x))
                .unwrap_or(usize::MAX);
            idx < grid.tiles.len() && grid.tiles[idx].is_walkable()
        };

        if !is_walkable {
            continue;
        }

        let anomaly_type = match rng.gen_range(0..3) {
            0 => AnomalyType::Ruins,
            1 => AnomalyType::StrangeFlora,
            _ => AnomalyType::Geode,
        };

        // Different rewards/difficulty per type?
        let (reward, difficulty) = match anomaly_type {
            AnomalyType::Ruins => (20.0, 150.0), // Knowledge is valuable, takes longer
            AnomalyType::StrangeFlora => (50.0, 80.0), // Food is abundant, faster
            AnomalyType::Geode => (10.0, 120.0), // Rare resource, medium
        };

        let mut entity_cmds = world.spawn((
            Anomaly {
                anomaly_type,
                reward_amount: reward,
            },
            GridPosition {
                x: x as i32,
                y: y as i32,
            },
            ScanProgress {
                current: 0.0,
                required: difficulty,
            },
        ));

        if anomaly_type == AnomalyType::StrangeFlora {
            entity_cmds.insert(PheromoneEmitter {
                radius: 3,
                interval: 10,
                timer: 0,
                effect: PheromoneEffect {
                    label: "Strange Scent".to_string(),
                    value: 0.05,
                    duration: 20,
                },
            });
        }

        spawned += 1;
    }
}

#[allow(clippy::type_complexity)]
/// ⚡ Bolt Optimization:
/// Replaced `let mut scanners = Vec::new();` loop with an iterator chain.
/// This prevents reallocation overhead and manually tracking `Vec` state,
/// allowing `collect` to infer capacity directly from the query bounds.
fn collect_scanners(world: &mut World) -> Vec<(Entity, Entity)> {
    let striking_factions: std::collections::HashSet<crate::layer1::factions::FactionId> = world
        .get_resource::<crate::layer1::factions::Factions>()
        .map(|f| {
            f.map
                .iter()
                .filter(|(_, d)| d.state == crate::layer1::factions::FactionState::Striking)
                .map(|(id, _)| *id)
                .collect()
        })
        .unwrap_or_default();

    let mut query = world.query_filtered::<(
        Entity,
        &MovementTarget,
        Option<&crate::layer1::factions::FactionMember>,
    ), With<AtTarget>>();

    query
        .iter(world)
        .filter(|(_, mt, _)| mt.for_action == ActionType::Explore)
        .filter(|(_, _, faction_member)| {
            !faction_member
                .and_then(|m| m.faction_id)
                .is_some_and(|fid| striking_factions.contains(&fid))
        })
        .map(|(entity, mt, _)| (entity, mt.target_entity))
        .collect()
}

fn complete_anomaly_scan(world: &mut World, pop_entity: Entity, anomaly_entity: Entity) {
    let (anomaly_type, reward) = if let Some(anomaly) = world.get::<Anomaly>(anomaly_entity) {
        (anomaly.anomaly_type, anomaly.reward_amount)
    } else {
        cleanup_pop_explore_state(world, pop_entity);
        return;
    };

    let pos = world.get::<GridPosition>(anomaly_entity).copied();

    match anomaly_type {
        AnomalyType::Ruins => {
            world
                .resource_mut::<ColonyResources>()
                .add_knowledge(reward);
            world.resource_mut::<MessageLog>().add(format!(
                "Discovery: Scanned ruins yielded {reward:.0} Knowledge."
            ));
        }
        AnomalyType::StrangeFlora => {
            world.resource_mut::<ColonyResources>().add_food(reward);
            world.resource_mut::<MessageLog>().add(format!(
                "Discovery: Strange flora yielded {reward:.0} Food."
            ));
            if let Some(pos) = pos {
                world.spawn((
                    ResourceItem {
                        resource_type: ResourceType::Food,
                        amount: 10.0,
                    },
                    pos,
                ));
            }
        }
        AnomalyType::Geode => {
            let mut rng = rand::thread_rng();
            if rng.gen_bool(0.5) {
                world.resource_mut::<ColonyResources>().add_stone(reward);
                world
                    .resource_mut::<MessageLog>()
                    .add(format!("Discovery: Geode yielded {reward:.0} Stone."));
                if let Some(pos) = pos {
                    world.spawn((
                        ResourceItem {
                            resource_type: ResourceType::Stone,
                            amount: 10.0,
                        },
                        pos,
                    ));
                }
            } else {
                world.resource_mut::<ColonyResources>().add_ore(reward);
                world
                    .resource_mut::<MessageLog>()
                    .add(format!("Discovery: Geode yielded {reward:.0} Ore."));
                if let Some(pos) = pos {
                    world.spawn((
                        ResourceItem {
                            resource_type: ResourceType::Ore,
                            amount: 10.0,
                        },
                        pos,
                    ));
                }
            }
        }
    }

    world.despawn(anomaly_entity);
    cleanup_pop_explore_state(world, pop_entity);
}

/// Processes the scanning action for pops.
///
/// # Panics
///
/// Panics if the anomaly entity exists but lacks the `Anomaly` component.
pub fn process_scan_system(world: &mut World) {
    let scanners = collect_scanners(world);

    for (pop_entity, anomaly_entity) in scanners {
        let scan_amount = 1.0;

        if world.get_entity(anomaly_entity).is_err() {
            cleanup_pop_explore_state(world, pop_entity);
            continue;
        }

        let is_complete = if let Some(mut progress) = world.get_mut::<ScanProgress>(anomaly_entity)
        {
            progress.current += scan_amount;
            progress.is_complete()
        } else {
            cleanup_pop_explore_state(world, pop_entity);
            continue;
        };

        if is_complete {
            complete_anomaly_scan(world, pop_entity, anomaly_entity);
        }
    }
}

fn cleanup_pop_explore_state(world: &mut World, pop_entity: Entity) {
    world
        .entity_mut(pop_entity)
        .remove::<MovementTarget>()
        .remove::<AtTarget>();

    if let Some(mut action) = world.get_mut::<PopAction>(pop_entity) {
        action.current = ActionType::Idle;
        action.current_utility = 0.0;
        action.ticks_committed = 0;
    }
}

#[cfg(test)]
mod tests {
    use crate::layer1::actions::evaluate_simple_action;
    use crate::layer1::anomalies::{
        process_scan_system, spawn_initial_anomalies, Anomaly, AnomalyType, ScanProgress,
    };
    use crate::layer1::execution::{AtTarget, MovementTarget};
    use crate::layer1::map::GridPosition;
    use crate::layer1::resources::ColonyResources;
    use crate::layer1::terrain::{TerrainGrid, TerrainType};
    use crate::layer1::utility_ai::{ActionType, PopAction, UtilityWeights};
    use crate::layer1::utility_eval_types::ScorableCandidate;
    use crate::shared::log::MessageLog;
    use bevy_ecs::prelude::*;

    #[test]
    fn test_anomaly_component() {
        let anomaly = Anomaly {
            anomaly_type: AnomalyType::Ruins,
            reward_amount: 10.0,
        };
        assert_eq!(anomaly.anomaly_type, AnomalyType::Ruins);
    }

    #[test]
    fn test_scan_progress_component() {
        let progress = ScanProgress {
            current: 0.0,
            required: 100.0,
        };
        assert!(!progress.is_complete());
    }

    #[test]
    fn test_evaluate_explore_finds_anomaly() {
        let mut world = World::new();
        let pop_pos = GridPosition { x: 5, y: 5 };
        let weights = UtilityWeights::default();

        // Spawn anomaly
        let anomaly = world
            .spawn((
                Anomaly {
                    anomaly_type: AnomalyType::Geode,
                    reward_amount: 10.0,
                },
                GridPosition { x: 10, y: 5 }, // Distance 5
                ScanProgress::default(),
            ))
            .id();

        let anomalies: Vec<ScorableCandidate> = world
            .query::<(Entity, &GridPosition, &Anomaly)>()
            .iter(&world)
            .map(|(e, p, _)| ScorableCandidate::new(e, *p))
            .collect();

        let result = evaluate_simple_action(pop_pos, &weights, &anomalies, 0.3);

        assert!(result.is_some());
        let (_, target) = result.unwrap();
        assert_eq!(target, anomaly);
    }

    #[test]
    fn test_spawn_initial_anomalies() {
        let mut world = World::new();
        let width: usize = 20;
        let height = 20;
        let size = width.checked_mul(height).expect("Grid size overflow");
        assert!(size <= 10_000_000, "Grid size too large");
        world.insert_resource(TerrainGrid {
            width,
            height,
            tiles: vec![TerrainType::Grass; size],
        });
        world.insert_resource(crate::layer1::building::OccupiedTiles::default());

        spawn_initial_anomalies(&mut world, 5); // Spawn 5

        let count = world.query::<&Anomaly>().iter(&world).count();
        assert_eq!(count, 5);
    }

    #[test]
    fn test_process_scan_system_increments_progress() {
        let mut world = World::new();
        world.insert_resource(ColonyResources::default());
        world.insert_resource(MessageLog::default());

        let anomaly = world
            .spawn((
                Anomaly {
                    anomaly_type: AnomalyType::Ruins,
                    reward_amount: 10.0,
                },
                GridPosition { x: 0, y: 0 },
                ScanProgress {
                    current: 0.0,
                    required: 10.0,
                },
            ))
            .id();

        let _pop = world
            .spawn((
                GridPosition { x: 0, y: 0 },
                MovementTarget {
                    target_entity: anomaly,
                    target_position: GridPosition { x: 0, y: 0 },
                    for_action: ActionType::Explore,
                },
                AtTarget,
            ))
            .id();

        process_scan_system(&mut world);

        let progress = world.get::<ScanProgress>(anomaly).unwrap();
        assert_eq!(progress.current, 1.0);
    }

    #[test]
    fn test_process_scan_system_completes_scan() {
        let mut world = World::new();
        world.insert_resource(ColonyResources::default());
        world.insert_resource(MessageLog::default());

        let anomaly = world
            .spawn((
                Anomaly {
                    anomaly_type: AnomalyType::Ruins,
                    reward_amount: 100.0,
                },
                GridPosition { x: 0, y: 0 },
                ScanProgress {
                    current: 9.0,
                    required: 10.0,
                },
            ))
            .id();

        let pop = world
            .spawn((
                GridPosition { x: 0, y: 0 },
                MovementTarget {
                    target_entity: anomaly,
                    target_position: GridPosition { x: 0, y: 0 },
                    for_action: ActionType::Explore,
                },
                AtTarget,
                PopAction {
                    current: ActionType::Explore,
                    current_utility: 1.0,
                    ticks_committed: 5,
                },
            ))
            .id();

        process_scan_system(&mut world);

        // Anomaly should be despawned
        assert!(world.get_entity(anomaly).is_err());

        // Pop should be reset
        let mt = world.get::<MovementTarget>(pop);
        assert!(mt.is_none());
        let action = world.get::<PopAction>(pop).unwrap();
        assert_eq!(action.current, ActionType::Idle);

        // Rewards granted
        let resources = world.resource::<ColonyResources>();
        assert_eq!(resources.knowledge, 100.0);
    }
}
pub mod benevolent_malfunctions;
pub mod temporal_echoes;
pub mod void_sirens;
pub use temporal_echoes::*;

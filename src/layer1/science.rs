use crate::layer1::building::OccupiedTiles;
use crate::layer1::execution::{AtTarget, MovementTarget};
use crate::layer1::map::GridPosition;
use crate::layer1::resources::{ColonyResources, ResourceItem, ResourceType};
use crate::layer1::terrain::TerrainGrid;
use crate::layer1::utility_types::{ActionType, PopAction};
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
    let (width, height, tiles) = {
        let grid = world.resource::<TerrainGrid>();
        (grid.width, grid.height, grid.tiles.clone())
    };

    // We can't access OccupiedTiles and world at the same time if we borrow world mutably.
    // So we copy the occupied set.
    let occupied = world.resource::<OccupiedTiles>().0.clone();

    let mut rng = rand::thread_rng();
    let mut spawned = 0;
    let mut attempts = 0;

    while spawned < count && attempts < count * 20 {
        attempts += 1;
        let x = rng.gen_range(0..width);
        let y = rng.gen_range(0..height);

        // Check occupation
        if occupied.contains(&(x as i32, y as i32)) {
            continue;
        }

        // Check walkability (Anomalies must be accessible)
        let idx = y * width + x;
        if idx >= tiles.len() || !tiles[idx].is_walkable() {
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

        world.spawn((
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
        spawned += 1;
    }
}

/// Processes the scanning action for pops.
///
/// # Panics
///
/// Panics if the anomaly entity exists but lacks the `Anomaly` component.
#[allow(clippy::too_many_lines)]
pub fn process_scan_system(world: &mut World) {
    // Collect striking factions
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

    // 1. Collect scanner pops
    let mut scanners = Vec::new();

    // Query manually to avoid borrow checker issues with world
    let mut query = world.query_filtered::<(
        Entity,
        &MovementTarget,
        Option<&crate::layer1::factions::FactionMember>,
    ), With<AtTarget>>();
    for (entity, mt, faction_member) in query.iter(world) {
        if mt.for_action == ActionType::Explore {
            // Check strike
            let is_striking = faction_member
                .and_then(|m| m.faction_id)
                .is_some_and(|fid| striking_factions.contains(&fid));

            if !is_striking {
                scanners.push((entity, mt.target_entity));
            }
        }
    }

    // 2. Process each scanner
    for (pop_entity, anomaly_entity) in scanners {
        let scan_amount = 1.0; // Base scan speed (could be skill-based)

        // Check if anomaly still exists
        if world.get_entity(anomaly_entity).is_err() {
            cleanup_pop_explore_state(world, pop_entity);
            continue;
        }

        // Update progress
        let is_complete = if let Some(mut progress) = world.get_mut::<ScanProgress>(anomaly_entity)
        {
            progress.current += scan_amount;
            progress.is_complete()
        } else {
            // Target is not an anomaly or missing ScanProgress
            cleanup_pop_explore_state(world, pop_entity);
            continue;
        };

        // Handle completion
        if is_complete {
            // Get anomaly data (safe now that mutable borrow of progress is dropped)
            let (anomaly_type, reward) = if let Some(anomaly) = world.get::<Anomaly>(anomaly_entity)
            {
                (anomaly.anomaly_type, anomaly.reward_amount)
            } else {
                cleanup_pop_explore_state(world, pop_entity);
                continue;
            };

            let pos = *world.get::<GridPosition>(anomaly_entity).unwrap();

            // Grant rewards
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
                    // Maybe spawn item too?
                    world.spawn((
                        ResourceItem {
                            resource_type: ResourceType::Food,
                            amount: 10.0,
                        }, // Bonus item
                        pos,
                    ));
                }
                AnomalyType::Geode => {
                    // Random resource?
                    let mut rng = rand::thread_rng();
                    if rng.gen_bool(0.5) {
                        world.resource_mut::<ColonyResources>().add_stone(reward);
                        world
                            .resource_mut::<MessageLog>()
                            .add(format!("Discovery: Geode yielded {reward:.0} Stone."));
                        world.spawn((
                            ResourceItem {
                                resource_type: ResourceType::Stone,
                                amount: 10.0,
                            },
                            pos,
                        ));
                    } else {
                        world.resource_mut::<ColonyResources>().add_ore(reward);
                        world
                            .resource_mut::<MessageLog>()
                            .add(format!("Discovery: Geode yielded {reward:.0} Ore."));
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

            // Despawn anomaly
            world.despawn(anomaly_entity);

            // Cleanup pop
            cleanup_pop_explore_state(world, pop_entity);
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
    use crate::layer1::actions::explore::evaluate_explore;
    use crate::layer1::execution::{AtTarget, MovementTarget};
    use crate::layer1::map::GridPosition;
    use crate::layer1::resources::ColonyResources;
    use crate::layer1::science::{
        Anomaly, AnomalyType, ScanProgress, process_scan_system, spawn_initial_anomalies,
    };
    use crate::layer1::terrain::{TerrainGrid, TerrainType};
    use crate::layer1::utility_types::{ActionType, PopAction, UtilityWeights};
    use crate::layer1::utility_eval_types::PositionProxy;
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

        let anomalies: Vec<PositionProxy> = world
            .query::<(Entity, &GridPosition, &Anomaly)>()
            .iter(&world)
            .map(|(e, p, _)| PositionProxy { entity: e, pos: *p })
            .collect();

        let result = evaluate_explore(pop_pos, &weights, &anomalies);

        assert!(result.is_some());
        let (_, target) = result.unwrap();
        assert_eq!(target, anomaly);
    }

    #[test]
    fn test_spawn_initial_anomalies() {
        let mut world = World::new();
        let width = 20;
        let height = 20;
        world.insert_resource(TerrainGrid {
            width,
            height,
            tiles: vec![TerrainType::Grass; width * height],
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

use bevy_ecs::prelude::*;
use bevy_time::{Time, Timer, TimerMode};
use rand::seq::IteratorRandom;

use crate::layer1::building::{BuildingMap, BuildingType, OccupiedTiles};
use crate::layer1::events::BuildingRemovedEvent;
use crate::layer1::map::GridPosition;
use crate::layer1::terrain::TerrainGrid;

#[derive(Component)]
pub struct BureaucracyNode {
    pub expansion_timer: Timer,
}

impl Default for BureaucracyNode {
    fn default() -> Self {
        Self {
            expansion_timer: Timer::from_seconds(10.0, TimerMode::Repeating),
        }
    }
}

/// Represents the overall stability of the empire.
/// Boosted by Bureaucracy Nodes.
#[derive(Resource, Default)]
pub struct EmpireStability {
    pub value: f32,
}

#[allow(clippy::too_many_arguments)]
pub fn expand_bureaucracy_nodes_system(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(&mut BureaucracyNode, &GridPosition)>,
    terrain: Res<TerrainGrid>,
    mut occupied: ResMut<OccupiedTiles>,
    mut building_map: ResMut<BuildingMap>,
    mut removed_events: EventWriter<BuildingRemovedEvent>,
    building_query: Query<&crate::layer1::building::Building>,
) {
    let mut rng = rand::thread_rng();

    // Collect all expansions first to avoid modifying query result during iteration
    let mut expansions = Vec::new();

    for (mut node, pos) in query.iter_mut() {
        node.expansion_timer.tick(time.delta());
        if node.expansion_timer.just_finished() {
            // Prefer to expand to non-office tiles if possible
            let offsets = [(0, 1), (1, 0), (0, -1), (-1, 0)];

            let mut valid_targets = Vec::new();
            for (dx, dy) in offsets {
                let target_x = pos.x + dx;
                let target_y = pos.y + dy;

                if target_x >= 0
                    && target_x < terrain.width as i32
                    && target_y >= 0
                    && target_y < terrain.height as i32
                {
                    // Check if it's already an office
                    let mut is_office = false;
                    if occupied.0.contains(&(target_x, target_y)) {
                        if let Some(&entity) = building_map.0.get(&(target_x, target_y)) {
                            if let Ok(building) = building_query.get(entity) {
                                if building.building_type == BuildingType::Office {
                                    is_office = true;
                                }
                            }
                        }
                    }
                    if !is_office {
                        valid_targets.push((target_x, target_y));
                    }
                }
            }

            if !valid_targets.is_empty() {
                let chosen = valid_targets.into_iter().choose(&mut rng).unwrap();
                expansions.push(chosen);
            }
        }
    }

    for (target_x, target_y) in expansions {
        let mut should_spawn = true;

        if occupied.0.contains(&(target_x, target_y)) {
            if let Some(&entity) = building_map.0.get(&(target_x, target_y)) {
                // Determine building type for event
                let b_type = if let Ok(building) = building_query.get(entity) {
                    // Check if it's already an Office
                    if building.building_type == BuildingType::Office {
                        should_spawn = false;
                    }
                    building.building_type
                } else {
                    BuildingType::Housing // Fallback
                };

                if should_spawn {
                    // Despawn the building
                    commands.entity(entity).despawn();

                    // Fire event
                    removed_events.send(BuildingRemovedEvent {
                        entity,
                        position: GridPosition {
                            x: target_x,
                            y: target_y,
                        },
                        building_type: b_type,
                    });

                    // Clear occupancy and map
                    occupied.0.remove(&(target_x, target_y));
                    building_map.0.remove(&(target_x, target_y));
                }
            }
        }

        if should_spawn {
            let new_node = commands
                .spawn((
                    BureaucracyNode::default(),
                    GridPosition {
                        x: target_x,
                        y: target_y,
                    },
                    crate::layer1::building::Building {
                        building_type: BuildingType::Office,
                    },
                ))
                .id();

            occupied.0.insert((target_x, target_y));
            building_map.0.insert((target_x, target_y), new_node);
        }
    }
}

pub fn calculate_bureaucracy_stability_system(
    query: Query<&BureaucracyNode>,
    mut stability: ResMut<EmpireStability>,
) {
    let node_count = query.iter().count() as f32;
    stability.value = node_count * 10.0;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::building::Building;
    use bevy_app::App;
    use bevy_app::Update;
    use bevy_time::TimerMode;

    #[test]
    fn test_bureaucracy_node_expansion() {
        let mut app = App::new();
        // Setup minimum required resources
        app.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![crate::layer1::terrain::TerrainType::Grass; 100],
        });
        app.init_resource::<OccupiedTiles>();
        app.init_resource::<BuildingMap>();
        app.init_resource::<Events<BuildingRemovedEvent>>();
        app.init_resource::<Time>();

        // Spawn a node
        let original_node = app
            .world_mut()
            .spawn((
                BureaucracyNode {
                    expansion_timer: Timer::from_seconds(1.0, TimerMode::Once),
                },
                GridPosition { x: 5, y: 5 },
                Building {
                    building_type: BuildingType::Office,
                },
            ))
            .id();

        app.world_mut()
            .resource_mut::<OccupiedTiles>()
            .0
            .insert((5, 5));
        app.world_mut()
            .resource_mut::<BuildingMap>()
            .0
            .insert((5, 5), original_node);

        // Let's spawn an adjacent farm at (5, 6)
        let farm_entity = app
            .world_mut()
            .spawn((
                GridPosition { x: 5, y: 6 },
                Building {
                    building_type: BuildingType::Farm,
                },
            ))
            .id();
        app.world_mut()
            .resource_mut::<OccupiedTiles>()
            .0
            .insert((5, 6));
        app.world_mut()
            .resource_mut::<BuildingMap>()
            .0
            .insert((5, 6), farm_entity);

        // Fill other adjacent tiles with offices to force expansion to (5, 6)
        for &(x, y) in &[(5, 4), (4, 5), (6, 5)] {
            let ent = app
                .world_mut()
                .spawn((
                    BureaucracyNode {
                        expansion_timer: Timer::from_seconds(1.0, TimerMode::Once),
                    },
                    GridPosition { x, y },
                    Building {
                        building_type: BuildingType::Office,
                    },
                ))
                .id();
            app.world_mut()
                .resource_mut::<OccupiedTiles>()
                .0
                .insert((x, y));
            app.world_mut()
                .resource_mut::<BuildingMap>()
                .0
                .insert((x, y), ent);
        }

        app.add_systems(Update, expand_bureaucracy_nodes_system);

        let mut expanded = false;

        for _ in 0..10 {
            app.world_mut()
                .resource_mut::<Time>()
                .advance_by(std::time::Duration::from_secs(2));
            app.update();
            let mut found = false;
            let mut query = app.world_mut().query::<(&BureaucracyNode, &GridPosition)>();
            for (_, pos) in query.iter(app.world()) {
                if pos.x == 5 && pos.y == 6 {
                    found = true;
                    break;
                }
            }
            if found {
                expanded = true;
                break;
            }

            let mut query2 = app.world_mut().query::<&mut BureaucracyNode>();
            for mut node in query2.iter_mut(app.world_mut()) {
                node.expansion_timer
                    .set_elapsed(std::time::Duration::from_secs(2));
            }
        }

        assert!(expanded, "Bureaucracy node should have expanded to (5, 6)");
    }

    #[test]
    fn test_bureaucracy_provides_stability() {
        let mut app = App::new();
        app.init_resource::<EmpireStability>();
        app.world_mut()
            .spawn((BureaucracyNode::default(), GridPosition { x: 0, y: 0 }));

        app.add_systems(Update, calculate_bureaucracy_stability_system);
        app.update();

        let stability = app.world().resource::<EmpireStability>();
        assert!(
            stability.value > 0.0,
            "Bureaucracy node should provide empire stability"
        );
    }
}

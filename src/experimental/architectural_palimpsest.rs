//! The Architectural Palimpsest (Nova Feature).
//!
//! # The Spark
//! We have Demolition and Stress mechanics. What if the ghosts of the old city
//! still dictate how the new one feels?
//!
//! # The Feature
//! When a building is demolished, it leaves an invisible `BuildingShadow` on those
//! tiles, inheriting a fraction of the old building's aura (Beauty, Squalor, or History).
//! Pops walking over these shadows will passively gain stress if the old building was
//! squalid or dreadful, or reduce stress if it was beautiful.

use crate::layer1::building::Building;
use crate::layer1::map::GridPosition;
use crate::layer1::pop::Pop;
use crate::layer1::stress::StressTracker;
use bevy_ecs::prelude::*;
use std::collections::HashMap;

/// An invisible remnant of a demolished building.
#[derive(Component, Debug, Clone)]
pub struct BuildingShadow {
    pub squalor_memory: f32,
    pub beauty_memory: f32,
}

/// Tracks active buildings to detect when they are despawned (demolished).
pub fn detect_building_demolitions(
    mut commands: Commands,
    buildings: Query<(Entity, &GridPosition, &Building)>,
    mut local_tracker: Local<HashMap<Entity, (GridPosition, f32, f32)>>,
) {
    let mut current_entities = std::collections::HashSet::new();

    // 1. Update the local tracker with all currently active buildings
    for (entity, pos, building) in buildings.iter() {
        current_entities.insert(entity);

        // In a real scenario, we might extract Squalor or Beauty components if they existed on the building.
        // For this MVP, we derive it from the building type or just assign placeholder values
        // based on the building type.
        let beauty = building.building_type.beauty_radius();
        // Assume industrial buildings have higher squalor memory
        let squalor = match building.building_type {
            crate::layer1::building::BuildingType::Smelter
            | crate::layer1::building::BuildingType::StoneMason => 5.0,
            crate::layer1::building::BuildingType::Tavern => 10.0,
            _ => 0.0,
        };

        local_tracker.insert(entity, (*pos, squalor, beauty));
    }

    // 2. Find any entities that were in the tracker but are no longer in the query
    let mut demolished = Vec::new();
    for (entity, data) in local_tracker.iter() {
        if !current_entities.contains(entity) {
            demolished.push((*entity, *data));
        }
    }

    // 3. For each demolished building, spawn a BuildingShadow and remove it from the tracker
    for (entity, (pos, squalor, beauty)) in demolished {
        local_tracker.remove(&entity);

        // Only spawn a shadow if there was actually some notable aura
        if squalor > 0.0 || beauty > 0.0 {
            commands.spawn((
                pos,
                BuildingShadow {
                    squalor_memory: squalor * 0.5, // Inherit a fraction of the old aura
                    beauty_memory: beauty * 0.5,
                },
            ));
        }
    }
}

/// Applies the aura of the building shadows to any Pop walking over them.
pub fn apply_palimpsest_aura(
    shadows: Query<(&GridPosition, &BuildingShadow)>,
    mut pops: Query<(&GridPosition, &mut StressTracker), With<Pop>>,
) {
    if shadows.is_empty() {
        return;
    }

    // Since shadows don't move, we can gather them by position for faster lookup
    let mut shadow_map = HashMap::new();
    for (pos, shadow) in shadows.iter() {
        shadow_map.insert((pos.x, pos.y), shadow);
    }

    for (pop_pos, mut stress) in pops.iter_mut() {
        if let Some(shadow) = shadow_map.get(&(pop_pos.x, pop_pos.y)) {
            // Apply a fraction of the memory as passive stress or relief
            let stress_change = (shadow.squalor_memory * 0.1) - (shadow.beauty_memory * 0.1);

            // Apply limits so it doesn't instantly max or clear stress, just acts as a passive modifier
            if stress_change > 0.0 {
                stress.accumulated_stress = (stress.accumulated_stress + stress_change).min(100.0);
            } else if stress_change < 0.0 {
                stress.accumulated_stress =
                    (stress.accumulated_stress - stress_change.abs()).max(0.0);
            }
        }
    }
}

pub fn register(schedule: &mut Schedule) {
    schedule.add_systems((detect_building_demolitions, apply_palimpsest_aura));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::building::BuildingType;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_building_demolition_creates_shadow() {
        use bevy_app::{App, Update};
        let mut app = App::new();
        app.add_systems(Update, detect_building_demolitions);

        // Spawn a building that will generate squalor (e.g. Tavern)
        let building_entity = app
            .world_mut()
            .spawn((
                GridPosition { x: 5, y: 5 },
                Building {
                    building_type: BuildingType::Tavern,
                },
            ))
            .id();

        // Run detect_building_demolitions once to populate the Local tracker
        app.update();

        assert_eq!(
            app.world_mut()
                .query::<&BuildingShadow>()
                .iter(app.world_mut())
                .count(),
            0
        );

        // Despawn the building (simulating demolition)
        app.world_mut().despawn(building_entity);

        // Run system again
        app.update();

        // Shadow should be created
        let mut shadow_q = app.world_mut().query::<(&GridPosition, &BuildingShadow)>();
        let mut shadow_count = 0;
        for (pos, shadow) in shadow_q.iter(app.world()) {
            assert_eq!(pos.x, 5);
            assert_eq!(pos.y, 5);
            assert!(shadow.squalor_memory > 0.0);
            shadow_count += 1;
        }
        assert_eq!(shadow_count, 1);
    }

    #[test]
    fn test_palimpsest_aura_affects_pops() {
        let mut world = World::new();

        // Spawn a shadow directly
        world.spawn((
            GridPosition { x: 10, y: 10 },
            BuildingShadow {
                squalor_memory: 10.0,
                beauty_memory: 0.0,
            },
        ));

        // Spawn a Pop on the same position
        let pop_entity = world
            .spawn((
                Pop,
                GridPosition { x: 10, y: 10 },
                StressTracker {
                    accumulated_stress: 0.0,
                },
            ))
            .id();

        // Run aura system
        world.run_system_once(apply_palimpsest_aura).unwrap();

        // Check if stress increased
        let stress = world.get::<StressTracker>(pop_entity).unwrap();
        assert!(stress.accumulated_stress > 0.0);
    }
}

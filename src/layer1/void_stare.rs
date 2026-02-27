use super::VoidExposure;
use super::VoidGrid;
use crate::layer1::building::Building;
use crate::layer1::map::GridPosition;
use crate::layer1::needs::Needs;
use crate::layer1::pop::Pop;
use crate::layer1::structure::Structure;
use crate::layer1::terrain::{TerrainGrid, TerrainType};
use crate::layer1::void_stare::VoidAnchor;
use crate::layer1::utility_types::{ActionType, PopAction};
use crate::shared::log::MessageLog;
use bevy_ecs::prelude::*;

/// System to update void exposure for pops.
pub fn update_void_exposure_system(
    mut pops: Query<(&GridPosition, &mut VoidExposure, &mut Needs), With<Pop>>,
    void_grid: Res<VoidGrid>,
    terrain: Res<TerrainGrid>,
    buildings: Query<(&GridPosition, Option<&VoidAnchor>, Option<&Building>, Option<&Structure>)>,
) {
    // Collect occluders (Structures) for fast lookup if needed
    // But for "Stare" logic, occlusion usually implies "Cannot see Void".
    // If a pop is surrounded by walls, they shouldn't see void outside.
    // However, VoidGrid is "Static Intensity".
    // Let's refine the logic:
    // If a pop is ON a tile with a Building/Structure, Void gain is reduced (Safe inside).
    // Or check line of sight?
    // Simplified: Check if current tile is occupied by a structure.

    // Build a quick set of structural positions for occlusion check?
    // Since `buildings` query iterates all, we can build a HashSet.
    // Optimization: Use `OccupiedTiles` resource if available? But that doesn't distinguish Walls from Floors.
    // Let's iterate `buildings` once to find anchors and occlusion near pops?
    // Or just do O(N*M) since N (Pops) is small (5-50) and M (Buildings) is moderate (100-1000).
    // Better: Only check buildings close to pop.

    // Let's assume VoidGrid handles the "External View".
    // We modify the gain based on local occlusion.

    for (pos, mut exposure, mut needs) in &mut pops {
        // 1. Check current tile "Void Intensity" (Static map data)
        let void_intensity = void_grid.get(pos.x, pos.y);

        // 2. Check "Life" view (Terrain)
        let mut visible_life = 0.0;
        let mut occlusion = 0.0;

        // Check 8 neighbors
        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let nx = pos.x + dx;
                let ny = pos.y + dy;

                if let Some(tile) = terrain.get(nx as usize, ny as usize) {
                    match tile {
                        TerrainType::Tree | TerrainType::Grass | TerrainType::Water => {
                            visible_life += 0.5;
                        }
                        _ => {}
                    }
                }
            }
        }

        // 3. Check Buildings (Anchors & Occlusion)
        for (b_pos, anchor, _building, structure) in &buildings {
            let dist = pos.distance_chebyshev(*b_pos);

            // Anchors
            if let Some(anchor) = anchor {
                if dist <= anchor.radius as u32 {
                    visible_life += anchor.strength;
                }
            }

            // Occlusion (Being near/inside structures reduces void gain)
            if dist <= 1 && structure.is_some() {
                occlusion += 0.5;
            }
        }

        // 4. Calculate Delta
        // Occlusion dampens the Void Intensity.
        // If occlusion > 1.0 (surrounded by walls), effective void is 0.
        let effective_void = (void_intensity - occlusion * 0.5).max(0.0);

        let gain = effective_void * 0.1;
        let loss = visible_life * 0.05;

        // Apply
        let delta = (gain - loss) * exposure.susceptibility;
        exposure.current = (exposure.current + delta).clamp(0.0, 100.0);

        // 5. Apply Stress
        if exposure.current > 50.0 {
            // Add stress proportional to exposure above 50%
            let stress_factor = (exposure.current - 50.0) / 5000.0; // Small increment
            needs.leisure = (needs.leisure - stress_factor).max(0.0);
        }
    }
}

/// System to handle manifestations of high void exposure.
pub fn void_manifestation_system(
    mut pops: Query<(Entity, &mut VoidExposure, &mut PopAction), With<Pop>>,
    mut log: Option<ResMut<MessageLog>>,
) {
    for (entity, mut exposure, mut action) in &mut pops {
        // Decrease timer
        if exposure.check_timer > 0 {
            exposure.check_timer -= 1;
            continue;
        }
        exposure.check_timer = 100; // Reset timer

        if exposure.current > 80.0 {
            // Manifestation: The Stare
            action.current = ActionType::VoidStare;
            action.ticks_committed = 0; // Reset commitment

            if let Some(ref mut log) = log {
                // Rate limit logs
                if exposure.check_timer == 100 { // Only once per check cycle
                     let index = entity.index();
                     log.add(format!("Pop {} stares into the abyss...", index));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::map::GridPosition;
    use crate::layer1::needs::Needs;
    use crate::layer1::pop::Pop;
    use crate::layer1::terrain::{TerrainGrid, TerrainType};
    use crate::layer1::utility_types::{ActionType, PopAction};

    #[test]
    fn test_void_exposure_gain() {
        let mut world = World::new();
        // Setup Resources
        let mut void_grid = VoidGrid::new(10, 10);
        void_grid.set(5, 5, 1.0); // Abyssal tile
        world.insert_resource(void_grid);

        let tiles = vec![TerrainType::Grass; 100];
        world.insert_resource(TerrainGrid { width: 10, height: 10, tiles });

        // Spawn Pop
        let pop = world.spawn((
            Pop,
            GridPosition { x: 5, y: 5 },
            VoidExposure {
                current: 0.0,
                susceptibility: 1.0,
                check_timer: 0,
            },
            Needs::default(),
        )).id();

        // Run Update
        let mut schedule = Schedule::default();
        schedule.add_systems(update_void_exposure_system);
        schedule.run(&mut world);

        // Check gain
        let exposure = world.get::<VoidExposure>(pop).unwrap();
        assert!(exposure.current > 0.0, "Pop should gain exposure from void grid");
    }

    #[test]
    fn test_manifestation_triggers() {
        let mut world = World::new();
        world.insert_resource(MessageLog::default());

        let pop = world.spawn((
            Pop,
            VoidExposure {
                current: 85.0, // High exposure
                susceptibility: 1.0,
                check_timer: 0, // Ready
            },
            PopAction {
                current: ActionType::Work, // Doing something
                ..Default::default()
            },
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(void_manifestation_system);
        schedule.run(&mut world);

        // Check Action Override
        let action = world.get::<PopAction>(pop).unwrap();
        assert_eq!(action.current, ActionType::VoidStare, "Pop should be forced to VoidStare");

        // Check Log
        let log = world.resource::<MessageLog>();
        assert!(log.messages.iter().any(|m| m.text.contains("stares into the abyss")), "Log should record manifestation");
    }
}

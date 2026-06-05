use crate::layer1::actions::AssignedTo;
use crate::layer1::architecture::window::Window;
use crate::layer1::building::Building;
use crate::layer1::map::GridPosition;
use crate::layer1::needs::Needs;
use crate::layer1::pop::Pop;
use crate::layer1::structure::Structure;
use crate::layer1::terrain::{TerrainGrid, TerrainType};
use crate::layer1::utility_types::{ActionType, PopAction};
use crate::shared::log::MessageLog;
use bevy_ecs::prelude::*;

/// Tracks a pop's psychological exposure to the Void.
#[derive(Component, Debug, Clone, Default)]
pub struct VoidExposure {
    /// Current exposure level (0.0 - 100.0).
    pub current: f32,
    /// Multiplier for exposure gain (based on traits).
    pub susceptibility: f32,
    /// Timer for periodic manifestation checks.
    pub check_timer: u32,
}

/// A component that "anchors" pops against the Void (e.g. Hearth, Totem).
#[derive(Component, Debug, Clone)]
pub struct VoidAnchor {
    /// Strength of protection.
    pub strength: f32,
    /// Radius of effect.
    pub radius: f32,
}

/// A grid tracking "Void Intensity" across the map.
#[derive(Resource)]
pub struct VoidGrid {
    pub width: usize,
    pub height: usize,
    pub values: Vec<f32>,
}

impl VoidGrid {
    pub fn new(width: usize, height: usize) -> Self {
        let size = width
            .checked_mul(height)
            .expect("Grid size overflow or too large");
        assert!(size <= 10_000_000, "Grid size overflow or too large");
        Self {
            width,
            height,
            values: vec![0.0; size],
        }
    }

    pub fn set(&mut self, x: i32, y: i32, value: f32) {
        if x >= 0 && y >= 0 && (x as usize) < self.width && (y as usize) < self.height {
            let idx = (y as usize)
                .checked_mul(self.width)
                .and_then(|i| i.checked_add(x as usize));
            if let Some(idx) = idx.filter(|&i| i < self.values.len()) {
                self.values[idx] = value;
            }
        }
    }

    pub fn get(&self, x: i32, y: i32) -> f32 {
        if x >= 0 && y >= 0 && (x as usize) < self.width && (y as usize) < self.height {
            let idx = (y as usize)
                .checked_mul(self.width)
                .and_then(|i| i.checked_add(x as usize));
            if let Some(idx) = idx.filter(|&i| i < self.values.len()) {
                return self.values[idx];
            }
        }

        1.0 // Outside is Void
    }
}

/// System to update void exposure for pops.
type VoidBuildingQuery<'a> = (
    &'a GridPosition,
    Option<&'a VoidAnchor>,
    Option<&'a Building>,
    Option<&'a Structure>,
);

pub fn update_void_exposure_system(
    mut pops: Query<(&GridPosition, &mut VoidExposure, &mut Needs), With<Pop>>,
    void_grid: Res<VoidGrid>,
    terrain: Res<TerrainGrid>,
    buildings: Query<VoidBuildingQuery>,
) {
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

                if let Some(TerrainType::Tree | TerrainType::Grass | TerrainType::Water) =
                    terrain.get(nx as usize, ny as usize)
                {
                    visible_life += 0.5;
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

            // Occlusion
            if dist <= 1 {
                if let Some(_s) = structure {
                    occlusion += 0.5;
                }
            }
        }

        // 4. Calculate Delta
        let effective_void = (void_intensity - occlusion * 0.5).max(0.0);

        let gain = effective_void * 0.1;
        let loss = visible_life * 0.05;

        // Apply
        let delta = (gain - loss) * exposure.susceptibility;
        exposure.current = (exposure.current + delta).clamp(0.0, 100.0);

        // 5. Apply Stress
        if exposure.current > 50.0 {
            let stress_factor = (exposure.current - 50.0) / 5000.0;
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
                if exposure.check_timer == 100 {
                    // Only once per check cycle
                    log.add(format!("Pop {:?} stares into the abyss...", entity));
                }
            }
        }
    }
}

#[derive(Component)]
pub struct VoidStareEffect {
    pub facing_void: bool,
}

#[allow(clippy::type_complexity)]
pub fn update_void_facing_stress_system(
    mut pops: Query<
        (
            &mut crate::layer1::psychology::stress::StressTracker,
            Option<&VoidStareEffect>,
            Option<&AssignedTo>,
            &GridPosition,
        ),
        With<Pop>,
    >,
    windows: Query<(&GridPosition, &Window)>,
    terrain: Res<TerrainGrid>,
) {
    for (mut stress, effect, assigned_to, pos) in pops.iter_mut() {
        let mut facing_void = None;

        // 1. Check direct component (e.g. from tests or forced scenarios)
        if let Some(e) = effect {
            facing_void = Some(e.facing_void);
        } else {
            // 2. Determine if looking through a window
            // Are they near a window? Let's check distance 1
            for (w_pos, window) in &windows {
                let dist = pos.distance_chebyshev(*w_pos);
                if dist <= 1 {
                    // Raycast in window direction
                    let (dx, dy) = window.direction.to_delta();
                    let mut found_life = false;
                    for i in 1..=window.range {
                        let tx = w_pos.x + dx * (i as i32);
                        let ty = w_pos.y + dy * (i as i32);

                        // Map edges are Void
                        if tx < 0
                            || ty < 0
                            || tx >= terrain.width as i32
                            || ty >= terrain.height as i32
                        {
                            facing_void = Some(true);
                            break;
                        }

                        if let Some(t) = terrain.get(tx as usize, ty as usize) {
                            if matches!(
                                t,
                                TerrainType::Grass | TerrainType::Tree | TerrainType::Water
                            ) {
                                found_life = true;
                                break;
                            }
                        }
                    }
                    if !found_life && window.range > 0 {
                        facing_void = Some(true);
                    } else if found_life {
                        facing_void = Some(false);
                    }
                }
            }

            // 3. Check if assigned to an observatory. Observatories naturally face the void.
            if facing_void.is_none() {
                if let Some(assignment) = assigned_to {
                    if assignment.assignment_type
                        == crate::layer1::actions::AssignmentType::ObservatoryWorker
                    {
                        facing_void = Some(true);
                    }
                }
            }
        }

        if let Some(facing_void) = facing_void {
            if facing_void {
                stress.accumulated_stress += 1.0;
            } else {
                stress.accumulated_stress = (stress.accumulated_stress - 1.0).max(0.0);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::architecture::window::Window;
    use crate::layer1::building::{BuildingType, Direction};
    use crate::layer1::map::GridPosition;
    use crate::layer1::needs::Needs;
    use crate::layer1::pop::Pop;
    use crate::layer1::psychology::stress::StressTracker;
    use crate::layer1::terrain::{TerrainGrid, TerrainType};
    use crate::layer1::utility_types::{ActionType, PopAction};

    #[test]
    fn test_void_exposure_gain() {
        let mut world = World::new();
        // Setup Resources
        let mut void_grid = VoidGrid::new(10, 10);
        void_grid.set(5, 5, 1.0); // Abyssal tile
        world.insert_resource(void_grid);

        let tiles = vec![TerrainType::Dirt; 100]; // Dirt does not give visible_life bonus
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });

        // Spawn Pop
        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 5, y: 5 },
                VoidExposure {
                    current: 0.0,
                    susceptibility: 1.0,
                    check_timer: 0,
                },
                Needs::default(),
            ))
            .id();

        // Run Update
        let mut schedule = Schedule::default();
        schedule.add_systems(update_void_exposure_system);
        schedule.run(&mut world);

        // Check gain
        let exposure = world.get::<VoidExposure>(pop).unwrap();
        assert!(
            exposure.current > 0.0,
            "Pop should gain exposure from void grid"
        );
    }

    #[test]
    fn test_manifestation_triggers() {
        let mut world = World::new();
        world.insert_resource(MessageLog::default());

        let pop = world
            .spawn((
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
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(void_manifestation_system);
        schedule.run(&mut world);

        // Check Action Override
        let action = world.get::<PopAction>(pop).unwrap();
        assert_eq!(
            action.current,
            ActionType::VoidStare,
            "Pop should be forced to VoidStare"
        );

        // Check Log
        let log = world.resource::<MessageLog>();
        assert!(
            log.messages
                .iter()
                .any(|m| m.text.contains("stares into the abyss")),
            "Log should record manifestation"
        );
    }

    #[test]
    #[should_panic(expected = "Grid size overflow or too large")]
    fn test_void_grid_new_overflow() {
        let _grid = VoidGrid::new(usize::MAX, 2);
    }

    #[test]
    #[should_panic(expected = "Grid size overflow or too large")]
    fn test_void_grid_new_too_large() {
        let _grid = VoidGrid::new(10000, 10000); // 100,000,000 > 10,000,000
    }

    #[test]
    fn test_void_grid_get_and_set_out_of_bounds() {
        let mut grid = VoidGrid::new(10, 10);

        // Negative coordinates
        grid.set(-1, -1, 0.5);
        assert_eq!(grid.get(-1, -1), 1.0, "Out of bounds should return 1.0");

        // Very large coordinates (preventing integer wrapping bugs)
        grid.set(i32::MAX, i32::MAX, 0.5);
        assert_eq!(grid.get(i32::MAX, i32::MAX), 1.0);

        // Just outside bounds
        grid.set(10, 10, 0.5);
        assert_eq!(grid.get(10, 10), 1.0);
    }

    #[test]
    fn test_void_facing_window_increases_stress() {
        let mut world = World::new();

        let size = 100;
        let tiles = vec![TerrainType::Dirt; size];
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });

        // Arrange: Pop in an observatory facing void
        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 5, y: 5 },
                StressTracker {
                    accumulated_stress: 10.0,
                },
                VoidStareEffect { facing_void: true },
            ))
            .id();

        // Act: Advance simulation
        let mut schedule = Schedule::default();
        schedule.add_systems(update_void_facing_stress_system);
        schedule.run(&mut world);

        // Assert stress increase
        let stress = world.get::<StressTracker>(pop).unwrap();
        assert!(stress.accumulated_stress > 10.0);
    }

    #[test]
    fn test_void_facing_window_decreases_stress_if_not_facing_void() {
        let mut world = World::new();

        let size = 100;
        let tiles = vec![TerrainType::Dirt; size];
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });

        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 5, y: 5 },
                StressTracker {
                    accumulated_stress: 10.0,
                },
                VoidStareEffect { facing_void: false },
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(update_void_facing_stress_system);
        schedule.run(&mut world);

        let stress = world.get::<StressTracker>(pop).unwrap();
        assert!(stress.accumulated_stress < 10.0);
    }

    #[test]
    fn test_raycast_window_faces_void() {
        let mut world = World::new();

        // Setup Terrain
        let size = 100;
        let tiles = vec![TerrainType::Dirt; size];
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });

        // Setup Window
        world.spawn((
            Window {
                direction: Direction::East,
                range: 10,
                view_cone: 0.0,
            },
            GridPosition { x: 5, y: 5 },
            Building {
                building_type: BuildingType::Window,
            },
        ));

        // Setup Pop near Window
        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 4, y: 5 },
                StressTracker {
                    accumulated_stress: 10.0,
                },
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(update_void_facing_stress_system);
        schedule.run(&mut world);

        let stress = world.get::<StressTracker>(pop).unwrap();
        assert!(stress.accumulated_stress > 10.0);
    }

    #[test]
    fn test_raycast_window_faces_life() {
        let mut world = World::new();

        // Setup Terrain with life in the view direction
        let size = 100;
        let mut tiles = vec![TerrainType::Dirt; size];
        tiles[5 * 10 + 7] = TerrainType::Tree;

        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });

        // Setup Window
        world.spawn((
            Window {
                direction: Direction::East,
                range: 10,
                view_cone: 0.0,
            },
            GridPosition { x: 5, y: 5 },
            Building {
                building_type: BuildingType::Window,
            },
        ));

        // Setup Pop near Window
        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 4, y: 5 },
                StressTracker {
                    accumulated_stress: 10.0,
                },
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(update_void_facing_stress_system);
        schedule.run(&mut world);

        let stress = world.get::<StressTracker>(pop).unwrap();
        assert!(stress.accumulated_stress < 10.0);
    }

    #[test]
    fn test_observatory_worker_faces_void() {
        let mut world = World::new();

        let size = 100;
        let tiles = vec![TerrainType::Dirt; size];
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });

        // Arrange
        let observatory = world
            .spawn(Building {
                building_type: BuildingType::Observatory,
            })
            .id();
        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 5, y: 5 },
                StressTracker {
                    accumulated_stress: 10.0,
                },
                crate::layer1::actions::AssignedTo {
                    entity: observatory,
                    assignment_type: crate::layer1::actions::AssignmentType::ObservatoryWorker,
                },
            ))
            .id();

        // Act
        let mut schedule = Schedule::default();
        schedule.add_systems(update_void_facing_stress_system);
        schedule.run(&mut world);

        // Assert stress increase
        let stress = world.get::<StressTracker>(pop).unwrap();
        assert!(stress.accumulated_stress > 10.0);
    }
}

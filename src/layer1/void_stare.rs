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
            self.values[(y as usize) * self.width + (x as usize)] = value;
        }
    }

    pub fn get(&self, x: i32, y: i32) -> f32 {
        if x >= 0 && y >= 0 && (x as usize) < self.width && (y as usize) < self.height {
            self.values[(y as usize) * self.width + (x as usize)]
        } else {
            1.0 // Outside is Void
        }
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
}

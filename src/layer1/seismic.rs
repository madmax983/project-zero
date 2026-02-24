//! Seismic Resonance system (Spec 171).

use crate::layer1::flora::Flora;
use crate::layer1::geology::GeologicalEvent;
use crate::layer1::map::GridPosition;
use crate::layer1::terrain::{TerrainGrid, TerrainType};
use bevy_ecs::prelude::*;
use rand::Rng;

/// Grid tracking seismic stress accumulation (Vibration).
#[derive(Resource)]
pub struct VibrationGrid {
    /// Width of the grid.
    pub width: usize,
    /// Height of the grid.
    pub height: usize,
    /// Vibration values (row-major).
    pub values: Vec<f32>,
}

impl VibrationGrid {
    /// Creates a new vibration grid.
    ///
    /// # Panics
    /// Panics if `width * height` overflows or exceeds 10,000,000.
    #[must_use]
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

    /// Gets vibration at coordinates.
    #[must_use]
    #[allow(clippy::cast_sign_loss, clippy::collapsible_if)]
    pub fn get(&self, x: i32, y: i32) -> f32 {
        if x < 0 || y < 0 {
            return 0.0;
        }
        let ux = x as usize;
        let uy = y as usize;
        if ux >= self.width || uy >= self.height {
            return 0.0;
        }
        if let Some(idx) = uy.checked_mul(self.width).and_then(|i| i.checked_add(ux)) {
            if idx < self.values.len() {
                return self.values[idx];
            }
        }
        0.0
    }

    /// Sets vibration at coordinates.
    #[allow(clippy::cast_sign_loss, clippy::collapsible_if)]
    pub fn set(&mut self, x: i32, y: i32, val: f32) {
        if x < 0 || y < 0 {
            return;
        }
        let ux = x as usize;
        let uy = y as usize;
        if ux >= self.width || uy >= self.height {
            return;
        }
        if let Some(idx) = uy.checked_mul(self.width).and_then(|i| i.checked_add(ux)) {
            if idx < self.values.len() {
                self.values[idx] = val;
            }
        }
    }
}

/// Component for machines that shake the ground.
#[derive(Component)]
pub struct SeismicSource {
    /// Base intensity of the vibration.
    pub intensity: f32,
    /// Radius of effect.
    pub radius: f32,
}

/// System to update the seismic grid based on sources and terrain.
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss
)]
pub fn update_seismic_system(
    mut grid: ResMut<VibrationGrid>,
    terrain: Res<TerrainGrid>,
    sources: Query<(&SeismicSource, &GridPosition)>,
) {
    grid.values.fill(0.0);

    for (source, pos) in &sources {
        let r = source.radius.ceil() as i32;
        let cx = pos.x;
        let cy = pos.y;

        for dy in -r..=r {
            for dx in -r..=r {
                let dist_sq = (dx * dx + dy * dy) as f32;
                if dist_sq > source.radius * source.radius {
                    continue;
                }

                let tx = cx + dx;
                let ty = cy + dy;

                if tx < 0 || ty < 0 {
                    continue;
                }
                let ux = tx as usize;
                let uy = ty as usize;

                if ux >= grid.width || uy >= grid.height {
                    continue;
                }

                // Terrain transmission logic
                let transmission = terrain.get(ux, uy).map_or(0.0, |tile| {
                    match tile {
                        TerrainType::Rock => 1.0,
                        TerrainType::Dirt => 0.5, // Soft ground dampens
                        _ => 0.7,
                    }
                });

                let raw = source.intensity * (1.0 - (dist_sq.sqrt() / source.radius));
                let final_val = raw * transmission;

                if final_val > 0.0 {
                    let cur = grid.get(tx, ty);
                    // Accumulate but cap at 2.0 to prevent infinite feedback
                    grid.set(tx, ty, (cur + final_val).min(2.0));
                }
            }
        }
    }
}

/// System where flora reacts to seismic activity.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub fn seismic_flora_reaction_system(
    grid: Res<VibrationGrid>,
    mut flora_query: Query<(&mut Flora, &GridPosition)>,
) {
    for (mut flora, pos) in &mut flora_query {
        let vibration = grid.get(pos.x, pos.y);
        if vibration > 0.1 {
            // Agitate: reduce timers
            // Logic: 0.1 vibration = 1 extra tick reduction
            let agitation = (vibration * 10.0) as u32;
            flora.growth_timer = flora.growth_timer.saturating_sub(agitation);
            flora.attack_timer = flora.attack_timer.saturating_sub(agitation);
        }
    }
}

/// System to trigger geological events from high seismic activity.
#[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
pub fn seismic_instability_system(
    grid: Res<VibrationGrid>,
    mut events: EventWriter<GeologicalEvent>,
) {
    let mut rng = rand::thread_rng();

    for y in 0..grid.height {
        for x in 0..grid.width {
            let val = grid.get(x as i32, y as i32);
            if val > 1.2 {
                // Chance to trigger
                // 1% chance per tick per tile is actually quite high if many tiles are vibrating.
                // Let's make it 1% chance.
                if rng.r#gen::<f32>() < 0.01 {
                    events.send(GeologicalEvent::Earthquake {
                        center: GridPosition {
                            x: x as i32,
                            y: y as i32,
                        },
                        magnitude: val, // Magnitude scales with vibration
                    });

                    // Limit to one event per tick to prevent catastrophe spam
                    return;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::flora::Flora;
    use crate::layer1::geology::GeologicalEvent;
    use crate::layer1::terrain::{TerrainGrid, TerrainType};

    #[test]
    fn test_vibration_grid_initialization() {
        let grid = VibrationGrid::new(10, 10);
        assert_eq!(grid.width, 10);
        assert_eq!(grid.height, 10);
        assert_eq!(grid.get(0, 0), 0.0);
    }

    #[test]
    fn test_seismic_source_propagation_rock_vs_dirt() {
        let mut world = World::new();
        let width = 10;
        let height = 10;
        let tiles = vec![TerrainType::Grass; width * height];
        let mut terrain = TerrainGrid {
            width,
            height,
            tiles,
        };

        // Row 0 is Rock (High transmission)
        for x in 0..10 {
            terrain.set(x, 0, TerrainType::Rock);
        }
        // Row 5 is Dirt (Low transmission)
        for x in 0..10 {
            terrain.set(x, 5, TerrainType::Dirt);
        }

        world.insert_resource(terrain);
        world.insert_resource(VibrationGrid::new(10, 10));

        // Source on Rock
        world.spawn((
            SeismicSource {
                intensity: 1.0,
                radius: 5.0,
            },
            GridPosition { x: 0, y: 0 },
        ));

        // Source on Dirt
        world.spawn((
            SeismicSource {
                intensity: 1.0,
                radius: 5.0,
            },
            GridPosition { x: 0, y: 5 },
        ));

        let mut schedule = Schedule::default();
        schedule.add_systems(update_seismic_system);
        schedule.run(&mut world);

        let grid = world.resource::<VibrationGrid>();

        // Rock should transmit further/stronger
        let rock_val = grid.get(3, 0);
        let dirt_val = grid.get(3, 5);

        assert!(
            rock_val > dirt_val,
            "Rock should transmit vibration better than Dirt"
        );
    }

    #[test]
    fn test_seismic_flora_agitation() {
        let mut world = World::new();
        let mut grid = VibrationGrid::new(10, 10);

        // Set high vibration at (5, 5)
        grid.set(5, 5, 0.8);
        world.insert_resource(grid);

        // Spawn Flora
        let flora_entity = world
            .spawn((
                Flora {
                    growth_timer: 100,
                    attack_timer: 100,
                    ..Default::default()
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        // Run reaction system
        let mut schedule = Schedule::default();
        schedule.add_systems(seismic_flora_reaction_system);
        schedule.run(&mut world);

        let flora = world.get::<Flora>(flora_entity).unwrap();

        // Timers should decrease faster than normal tick (or be reduced directly)
        assert!(flora.growth_timer < 100);
        assert!(flora.attack_timer < 100);
    }

    #[test]
    fn test_seismic_triggers_instability() {
        let mut world = World::new();
        let mut grid = VibrationGrid::new(10, 10);
        world.insert_resource(Events::<GeologicalEvent>::default());

        // Set EXTREME vibration at (5, 5)
        grid.set(5, 5, 1.5); // Over threshold
        world.insert_resource(grid);

        let mut triggered = false;
        let mut schedule = Schedule::default();
        schedule.add_systems(seismic_instability_system);

        for _ in 0..1000 {
            schedule.run(&mut world);
            let events = world.resource::<Events<GeologicalEvent>>();
            let mut reader = events.get_cursor();
            if reader.read(events).next().is_some() {
                triggered = true;
                break;
            }
        }

        assert!(
            triggered,
            "Should trigger instability event eventually with high vibration"
        );
    }
}

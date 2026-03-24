//! Geological instability system.
//!
//! Tracks seismic stress and triggers earthquakes.

use crate::layer1::map::ScreenShake;
use crate::layer1::GridPosition;
use bevy_ecs::prelude::*;
use ratatui::style::Color;

/// Grid tracking seismic stress accumulation.
#[derive(Resource, Default)]
pub struct SeismicGrid {
    /// Width of the grid.
    pub width: usize,
    /// Height of the grid.
    pub height: usize,
    /// Stress values (row-major).
    pub stress: Vec<f32>,
}

/// Stress level that triggers an earthquake.
pub const STRESS_THRESHOLD: f32 = 100.0;
/// Amount of stress that decays per tick.
pub const DECAY_RATE: f32 = 0.5;

impl SeismicGrid {
    /// Creates a new seismic grid with the given dimensions.
    #[must_use]
    pub fn new(width: usize, height: usize) -> Self {
        let size = width
            .checked_mul(height)
            .expect("Grid size overflow or too large");
        assert!(size <= 10_000_000, "Grid size overflow or too large");
        Self {
            width,
            height,
            stress: vec![0.0; size],
        }
    }

    /// Gets the stress level at the given coordinates.
    #[must_use]
    #[allow(clippy::cast_sign_loss)]
    pub fn get_stress(&self, x: i32, y: i32) -> f32 {
        if x < 0 || y < 0 || x as usize >= self.width || y as usize >= self.height {
            return 0.0;
        }
        if let Some(idx) = (y as usize)
            .checked_mul(self.width)
            .and_then(|i| i.checked_add(x as usize))
        {
            if idx < self.stress.len() {
                return self.stress[idx];
            }
        }
        0.0
    }

    /// Adds stress to the given coordinates.
    #[allow(clippy::cast_sign_loss)]
    pub fn add_stress(&mut self, x: i32, y: i32, amount: f32) {
        if x >= 0 && y >= 0 && (x as usize) < self.width && (y as usize) < self.height {
            if let Some(idx) = (y as usize)
                .checked_mul(self.width)
                .and_then(|i| i.checked_add(x as usize))
            {
                if idx < self.stress.len() {
                    self.stress[idx] += amount;
                }
            }
        }
    }

    /// Decays stress across the entire grid.
    pub fn decay(&mut self, rate: f32) {
        for s in &mut self.stress {
            *s = (*s - rate).max(0.0);
        }
    }
}

/// Events related to geological instability.
#[derive(Event, Debug, Clone)]
pub enum GeologicalEvent {
    /// A minor tremor (warning).
    Tremor {
        /// Center of the tremor.
        center: GridPosition,
    },
    /// A major earthquake (damage).
    Earthquake {
        /// Center of the earthquake.
        center: GridPosition,
        /// Magnitude of the earthquake (determines damage radius and intensity).
        magnitude: f32,
    },
}

/// Helper to add stress to the grid from other systems.
pub fn add_seismic_stress(world: &mut World, pos: GridPosition, amount: f32) {
    if let Some(mut grid) = world.get_resource_mut::<SeismicGrid>() {
        grid.add_stress(pos.x, pos.y, amount);
    }
}

/// System that decays seismic stress over time.
pub fn seismic_decay_system(mut grid: ResMut<SeismicGrid>) {
    grid.decay(DECAY_RATE);
}

/// System that checks for stress thresholds and triggers events.
#[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
pub fn check_seismic_events(
    mut grid: ResMut<SeismicGrid>,
    mut events: EventWriter<GeologicalEvent>,
) {
    for y in 0..grid.height {
        for x in 0..grid.width {
            let idx = y
                .checked_mul(grid.width)
                .and_then(|i| i.checked_add(x))
                .unwrap_or(usize::MAX);
            if idx < grid.stress.len() && grid.stress[idx] > STRESS_THRESHOLD {
                // Trigger event
                // Reset stress (release energy)
                grid.stress[idx] = 0.0;
                events.send(GeologicalEvent::Earthquake {
                    center: GridPosition {
                        x: x as i32,
                        y: y as i32,
                    },
                    magnitude: 5.0, // Simplified magnitude
                });
            }
        }
    }
}

/// System that applies effects of geological events (damage, visuals).
pub mod tectonic;

pub fn apply_geological_event_system(
    mut events: EventReader<GeologicalEvent>,
    mut commands: Commands,
    mut health_query: Query<(Entity, &GridPosition, &mut crate::layer1::health::Health)>,
    mut shake: Option<ResMut<ScreenShake>>,
) {
    for event in events.read() {
        if let GeologicalEvent::Earthquake { center, magnitude } = event {
            // Apply damage in radius (simple 1 tile for now based on test)
            let damage = 10.0 * magnitude;
            for (_entity, pos, mut health) in &mut health_query {
                if pos == center {
                    health.current -= damage;
                }
            }

            // Visual feedback
            if let Some(ref mut s) = shake {
                s.trigger(0.5 * magnitude);
            }

            commands.spawn((
                crate::layer1::particles::Particle {
                    char: '#',
                    color: Color::Red,
                    lifetime: 20,
                },
                *center,
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::layer1::geology::{
        add_seismic_stress, apply_geological_event_system, check_seismic_events, GeologicalEvent,
        SeismicGrid,
    };
    use crate::layer1::health::Health;
    use crate::layer1::GridPosition;
    use bevy_ecs::prelude::*;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_seismic_grid_initialization() {
        let grid = SeismicGrid::new(10, 10);
        assert_eq!(grid.get_stress(5, 5), 0.0);
    }

    #[test]
    fn test_mining_increases_stress() {
        let mut world = World::new();
        let grid = SeismicGrid::new(10, 10);
        world.insert_resource(grid);

        // Simulate mining at (5,5)
        add_seismic_stress(&mut world, GridPosition { x: 5, y: 5 }, 10.0);

        let grid = world.resource::<SeismicGrid>();
        assert!(grid.get_stress(5, 5) >= 10.0);
    }

    #[test]
    fn test_stress_decay() {
        let mut world = World::new();
        let mut grid = SeismicGrid::new(10, 10);
        grid.add_stress(5, 5, 50.0);
        world.insert_resource(grid);

        // Run decay system (simulated)
        world
            .run_system_once(crate::layer1::geology::seismic_decay_system)
            .unwrap();

        let grid = world.resource::<SeismicGrid>();
        assert!(grid.get_stress(5, 5) < 50.0);
    }

    #[test]
    fn test_earthquake_event_trigger() {
        let mut world = World::new();
        let mut grid = SeismicGrid::new(10, 10);
        // Set stress above threshold (e.g., 100.0)
        grid.add_stress(5, 5, 150.0);
        world.insert_resource(grid);
        world.init_resource::<Events<GeologicalEvent>>();

        // Run check system
        world.run_system_once(check_seismic_events).unwrap();

        // Check event
        let events = world.resource::<Events<GeologicalEvent>>();
        let mut reader = events.get_cursor();
        let emitted: Vec<_> = reader.read(events).collect();

        assert!(!emitted.is_empty());
        match emitted[0] {
            GeologicalEvent::Earthquake { center, .. } => {
                assert_eq!(center.x, 5);
                assert_eq!(center.y, 5);
            }
            _ => panic!("Expected Earthquake"),
        }
    }

    #[test]
    fn test_earthquake_damage() {
        let mut world = World::new();
        world.init_resource::<Events<GeologicalEvent>>();

        // Setup building/victim
        let victim = world
            .spawn((
                Health {
                    current: 100.0,
                    max: 100.0,
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();
        world.insert_resource(crate::layer1::map::ScreenShake::default());

        // Send event
        world.send_event(GeologicalEvent::Earthquake {
            center: GridPosition { x: 5, y: 5 },
            magnitude: 5.0,
        });

        // Run system
        world
            .run_system_once(apply_geological_event_system)
            .unwrap();

        // Check damage
        let health = world.get::<Health>(victim).unwrap();
        assert!(health.current < 100.0);
    }
}

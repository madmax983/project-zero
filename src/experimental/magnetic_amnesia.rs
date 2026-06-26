//! Magnetic Amnesia (Nova Feature).
//!
//! # The Spark
//! We have a `MagneticStorm` weather type in `WeatherType`, a `RoofGrid` system, and `Memories` in the psychology module.
//!
//! # The Feature
//! During a `MagneticStorm`, Pops caught outside (not under a `RoofGrid`) are exposed to intense
//! electromagnetic interference. There is a chance their `Memories` are completely wiped, erasing
//! both trauma and happy experiences.
//!
//! # The Potential
//! Connects the environment hazard (weather) directly to the cognitive psychology system.
//! A magnetic storm becomes a cognitive hazard that can "reset" a Pop. A ruthless player might
//! intentionally leave traumatized Pops outside during a storm to cure their PTSD, at the cost
//! of all their relationships and life experiences.

use crate::layer1::entities::pop::Pop;
use crate::layer1::map::GridPosition;
use crate::layer1::nature::weather::{WeatherState, WeatherType};
use crate::layer1::physics::structural_integrity::RoofGrid;
use crate::layer1::psychology::memory::Memories;
use bevy_ecs::prelude::*;
use rand::Rng;

const MAGNETIC_AMNESIA_CHANCE: f64 = 0.01;

pub fn magnetic_amnesia_system(
    weather_state: Option<Res<WeatherState>>,
    roof_grid: Option<Res<RoofGrid>>,
    mut pops: Query<(&GridPosition, &mut Memories), With<Pop>>,
    mut log: Option<ResMut<crate::shared::log::MessageLog>>,
) {
    if let Some(state) = weather_state {
        if state.current_weather != WeatherType::MagneticStorm {
            return;
        }
    } else {
        return;
    }

    let mut rng = rand::thread_rng();
    let has_roof = |x: i32, y: i32| -> bool {
        if let Some(ref grid) = roof_grid {
            grid.has_roof(x, y)
        } else {
            false
        }
    };

    for (pos, mut memories) in pops.iter_mut() {
        // If outside and RNG hits
        if !has_roof(pos.x, pos.y) && rng.gen_bool(MAGNETIC_AMNESIA_CHANCE) {
            // Only wipe if they actually have memories to lose
            if !memories.items.is_empty() {
                memories.items.clear();

                if let Some(ref mut l) = log {
                    l.add_colored(
                        format!(
                            "A pop at ({}, {}) suffered Magnetic Amnesia. All memories erased!",
                            pos.x, pos.y
                        ),
                        ratatui::style::Color::Magenta,
                    );
                }
            }
        }
    }
}

pub fn register(schedule: &mut Schedule) {
    schedule.add_systems(magnetic_amnesia_system);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::psychology::memory::MemoryType;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_magnetic_amnesia_wipes_memories() {
        let mut world = World::new();

        world.insert_resource(WeatherState {
            current_weather: WeatherType::MagneticStorm,
            duration_remaining: 10,
        });

        let grid = RoofGrid::new(10, 10);
        // No roof at (5, 5)
        world.insert_resource(grid);

        let mut memories = Memories::default();
        memories.add(MemoryType::AteFineMeal, 1);

        let pop = world
            .spawn((Pop, GridPosition { x: 5, y: 5 }, memories))
            .id();

        // Run multiple times to trigger the amnesia reliably
        for _ in 0..1000 {
            world.run_system_once(magnetic_amnesia_system).unwrap();
            world.flush();
        }

        let mems = world.get::<Memories>(pop).unwrap();
        assert!(
            mems.items.is_empty(),
            "Pop outside during magnetic storm should have memories wiped"
        );
    }
}

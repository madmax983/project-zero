//! Aeolian Clutter (Nova Feature).
//!
//! # The Spark
//! We have a `WindGrid` that tracks wind vectors, and a `ClutterGrid` that tracks physical mess.
//! Both are grids of the same dimensions.
//!
//! # The Feature
//! `AeolianClutter` (Windblown Clutter). A system that runs periodically to physically move clutter
//! from one tile to an adjacent tile based on the local wind vector from `WindGrid`.
//!
//! # The Potential
//! This creates emergent "trash drifts" against walls and in urban canyons.
//! Players can intentionally use wind blocking (walls) to create "clutter traps" that make janitorial duties more efficient.

use crate::layer1::clutter::ClutterGrid;
use crate::layer1::nature::wind::WindGrid;
use bevy_ecs::prelude::*;

/// Moves clutter from one tile to another based on wind vectors.
pub fn aeolian_clutter_system(
    clutter_grid: Option<ResMut<ClutterGrid>>,
    wind_grid: Option<Res<WindGrid>>,
) {
    let (Some(mut clutter), Some(wind)) = (clutter_grid, wind_grid) else {
        return;
    };

    let width = clutter.width;
    let height = clutter.height;

    // Use a temporary buffer to avoid moving the same clutter multiple times in one tick
    let mut new_clutter = vec![0.0; width * height];
    let threshold = 0.5; // Wind must be at least this fast to move clutter

    for y in 0..height {
        for x in 0..width {
            let idx = y * width + x;
            let current_clutter = clutter.get(x, y);

            if current_clutter <= 0.0 {
                continue;
            }

            let wind_vec = wind.get_wind(x as i32, y as i32);
            let wind_speed = wind_vec.length();

            if wind_speed > threshold {
                // Calculate target tile
                let dx = if wind_vec.x > 0.0 {
                    1
                } else if wind_vec.x < 0.0 {
                    -1
                } else {
                    0
                };
                let dy = if wind_vec.y > 0.0 {
                    1
                } else if wind_vec.y < 0.0 {
                    -1
                } else {
                    0
                };

                let nx = (x as i32) + dx;
                let ny = (y as i32) + dy;

                if nx >= 0 && nx < (clutter.width as i32) && ny >= 0 && ny < (clutter.height as i32)
                {
                    // Move a fraction of the clutter (e.g., 20% per tick)
                    let move_amount = (current_clutter * 0.2).min(current_clutter);

                    if let Some(new_idx) = (ny as usize)
                        .checked_mul(width)
                        .and_then(|i| i.checked_add(nx as usize))
                    {
                        if new_idx < new_clutter.len() {
                            new_clutter[new_idx] += move_amount;
                            new_clutter[idx] += current_clutter - move_amount;
                        } else {
                            new_clutter[idx] += current_clutter;
                        }
                    } else {
                        new_clutter[idx] += current_clutter;
                    }
                } else {
                    // Blown off the map, or at edge. Just leave it.
                    new_clutter[idx] += current_clutter;
                }
            } else {
                // Wind not strong enough, stays in place
                new_clutter[idx] += current_clutter;
            }
        }
    }

    // Apply new clutter values
    for y in 0..height {
        for x in 0..width {
            let idx = y * width + x;
            let new_val = new_clutter[idx].clamp(0.0, 100.0);

            // To overwrite the value cleanly without add/remove mechanics interference:
            // Since ClutterGrid doesn't have a direct `set` method (it has add_clutter and remove_clutter),
            // we will calculate the difference and apply it.
            let old_val = clutter.get(x, y);
            let diff = new_val - old_val;
            if diff > 0.0 {
                clutter.add_clutter(x, y, diff);
            } else if diff < 0.0 {
                clutter.remove_clutter(x, y, -diff);
            }
        }
    }
}

pub fn register(schedule: &mut Schedule) {
    schedule.add_systems(aeolian_clutter_system);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::nature::wind::Vec2;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_aeolian_clutter_moves_east() {
        let mut world = World::new();

        let mut clutter = ClutterGrid::new(10, 10);
        clutter.add_clutter(5, 5, 10.0);
        world.insert_resource(clutter);

        let mut wind = WindGrid::new(10, 10);
        // Fast wind pushing east
        wind.set_wind(5, 5, Vec2::new(1.0, 0.0));
        world.insert_resource(wind);

        world.run_system_once(aeolian_clutter_system).unwrap();

        let clutter_res = world.resource::<ClutterGrid>();

        // Original tile should have 8.0 (10.0 - 20%)
        let remaining = clutter_res.get(5, 5);
        assert!(
            (remaining - 8.0).abs() < f32::EPSILON,
            "Expected 8.0, got {}",
            remaining
        );

        // East tile should have 2.0
        let blown = clutter_res.get(6, 5);
        assert!(
            (blown - 2.0).abs() < f32::EPSILON,
            "Expected 2.0, got {}",
            blown
        );
    }

    #[test]
    fn test_aeolian_clutter_wind_too_weak() {
        let mut world = World::new();

        let mut clutter = ClutterGrid::new(10, 10);
        clutter.add_clutter(5, 5, 10.0);
        world.insert_resource(clutter);

        let mut wind = WindGrid::new(10, 10);
        // Weak wind pushing east
        wind.set_wind(5, 5, Vec2::new(0.1, 0.0));
        world.insert_resource(wind);

        world.run_system_once(aeolian_clutter_system).unwrap();

        let clutter_res = world.resource::<ClutterGrid>();

        // Original tile should still have 10.0
        let remaining = clutter_res.get(5, 5);
        assert!(
            (remaining - 10.0).abs() < f32::EPSILON,
            "Expected 10.0, got {}",
            remaining
        );
    }
}

use crate::layer1::map::GridPosition;
use crate::shared::time::SimulationTime;
use bevy_ecs::prelude::*;

#[derive(Component, Default)]
pub struct MachineRhythm {
    pub cycle_end_tick: u64,
    pub last_sync_bonus: f32, // Accumulated rhythm score
}

#[derive(Resource, Default)]
pub struct RhythmManager {
    // Global rhythm tracking if needed
}

pub fn update_rhythm_system(
    time: Res<SimulationTime>,
    mut query: Query<(&mut MachineRhythm, &GridPosition)>,
) {
    let machines: Vec<(u64, GridPosition)> =
        query.iter().map(|(r, p)| (r.cycle_end_tick, *p)).collect();

    for (mut rhythm, pos) in query.iter_mut() {
        if rhythm.cycle_end_tick != time.tick {
            continue;
        }

        let mut sync_count = 0;

        for (other_tick, other_pos) in &machines {
            if *other_pos == *pos {
                continue;
            } // Skip self

            if pos.distance_chebyshev(*other_pos) <= 1 {
                let diff = (*other_tick as i64 - time.tick as i64).abs();
                if diff <= 2 {
                    sync_count += 1;
                }
            }
        }

        if sync_count > 0 {
            rhythm.last_sync_bonus = 10.0 * sync_count as f32;
        } else {
            // Optional: Handle discord penalty, set back to 0.
            // For now, it stays whatever it was or resets to 0.
            rhythm.last_sync_bonus = 0.0;
        }
    }
}

use bevy_ecs::prelude::*;
use crate::layer1::flora::Flora;
use crate::layer1::map::GridPosition;
use crate::layer1::pop::Pop;
use crate::layer1::needs::Needs;
use crate::shared::time::SimulationTime;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BloomPhase {
    Dormant,
    Blooming,
    Hibernation,
}

#[derive(Component)]
pub struct LumifloraCycle {
    pub phase: BloomPhase,
    pub time_in_phase: f32,
}

#[derive(Component, Default)]
pub struct BioRhythmSync {
    pub is_synced: bool,
    pub work_speed_multiplier: f32,
}

pub fn lumiflora_bloom_system(
    _time: Res<SimulationTime>,
    mut query: Query<&mut LumifloraCycle, With<Flora>>,
) {
    let dt = 1.0; // Assume 1 tick is 1 time unit for simplicity of tests/implementation initially, or time.delta could be used if available
    for mut cycle in query.iter_mut() {
        cycle.time_in_phase += dt;

        match cycle.phase {
            BloomPhase::Dormant if cycle.time_in_phase > 10.0 => {
                cycle.phase = BloomPhase::Blooming;
                cycle.time_in_phase = 0.0;
            }
            BloomPhase::Blooming if cycle.time_in_phase > 5.0 => {
                cycle.phase = BloomPhase::Hibernation;
                cycle.time_in_phase = 0.0;
            }
            BloomPhase::Hibernation if cycle.time_in_phase > 5.0 => {
                cycle.phase = BloomPhase::Dormant;
                cycle.time_in_phase = 0.0;
            }
            _ => {}
        }
    }
}

pub fn apply_bio_rhythm_aura(
    flora_query: Query<(&LumifloraCycle, &GridPosition), With<Flora>>,
    mut pop_query: Query<(&GridPosition, &mut BioRhythmSync, &mut Needs), With<Pop>>,
) {
    // Collect all floras first to avoid O(N*M) continuous queries
    let floras: Vec<_> = flora_query.iter().collect();

    if floras.is_empty() {
        // Fast path: If there are no Lumiflora, just reset sync on all Pops
        for (_, mut sync, _) in pop_query.iter_mut() {
            if sync.is_synced {
                sync.is_synced = false;
                sync.work_speed_multiplier = 1.0;
            }
        }
        return;
    }

    for (pop_pos, mut sync, mut sleep) in pop_query.iter_mut() {
        let mut nearest_flora = None;
        let mut min_dist_sq = i32::MAX;

        for (flora_cycle, flora_pos) in &floras {
            let dx = pop_pos.x - flora_pos.x;
            let dy = pop_pos.y - flora_pos.y;
            let dist_sq = dx * dx + dy * dy;

            if dist_sq <= 25 && dist_sq < min_dist_sq { // radius 5
                min_dist_sq = dist_sq;
                nearest_flora = Some(*flora_cycle);
            }
        }

        if let Some(flora_cycle) = nearest_flora {
            sync.is_synced = true;
            match flora_cycle.phase {
                BloomPhase::Blooming => {
                    sync.work_speed_multiplier = 1.5;
                }
                BloomPhase::Hibernation => {
                    sync.work_speed_multiplier = 0.1;
                    sleep.rest = 0.0; // Force immediate sleep need
                }
                BloomPhase::Dormant => {
                    sync.work_speed_multiplier = 1.0;
                }
            }
        } else {
            sync.is_synced = false;
            sync.work_speed_multiplier = 1.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::flora::{Flora, FloraType};
    use crate::layer1::map::GridPosition;
    use crate::layer1::pop::Pop;
    use crate::layer1::needs::Needs;
    use crate::shared::time::SimulationTime;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_lumiflora_blooming_cycle() {
        let mut world = World::new();

        let time = SimulationTime { tick: 100, ..Default::default() };
        world.insert_resource(time);

        let flora = world.spawn((
            Flora { flora_type: FloraType::Lumiflora, ..Default::default() },
            LumifloraCycle { phase: BloomPhase::Dormant, time_in_phase: 10.0 },
        )).id();

        world.run_system_once(lumiflora_bloom_system).unwrap();

        let cycle = world.get::<LumifloraCycle>(flora).unwrap();

        assert_eq!(cycle.phase, BloomPhase::Blooming);
    }

    #[test]
    fn test_pop_bio_rhythm_sync() {
        let mut world = World::new();

        let flora = world.spawn((
            Flora { flora_type: FloraType::Lumiflora, ..Default::default() },
            LumifloraCycle { phase: BloomPhase::Blooming, time_in_phase: 0.0 },
            GridPosition { x: 5, y: 5 },
        )).id();

        let pop = world.spawn((
            Pop,
            GridPosition { x: 5, y: 5 },
            BioRhythmSync { is_synced: false, work_speed_multiplier: 1.0 },
            Needs { rest: 1.0, ..Default::default() },
        )).id();

        world.run_system_once(apply_bio_rhythm_aura).unwrap();

        let sync = world.get::<BioRhythmSync>(pop).unwrap();
        assert!(sync.is_synced);
        assert!(sync.work_speed_multiplier > 1.0); // Blooming gives speed boost

        // Change flora to Hibernation phase
        world.get_mut::<LumifloraCycle>(flora).unwrap().phase = BloomPhase::Hibernation;

        world.run_system_once(apply_bio_rhythm_aura).unwrap();

        let sleep = world.get::<Needs>(pop).unwrap();
        assert!(sleep.rest < f32::EPSILON); // Forces sleep (rest = 0.0)
    }

    #[test]
    fn test_lumiflora_cycle_progression() {
        let mut world = World::new();

        let time = SimulationTime { tick: 100, ..Default::default() };
        world.insert_resource(time);

        let flora = world.spawn((
            Flora { flora_type: FloraType::Lumiflora, ..Default::default() },
            LumifloraCycle { phase: BloomPhase::Dormant, time_in_phase: 0.0 },
        )).id();

        // Not enough time
        world.run_system_once(lumiflora_bloom_system).unwrap();
        assert_eq!(world.get::<LumifloraCycle>(flora).unwrap().phase, BloomPhase::Dormant);

        // Enough for blooming
        world.get_mut::<LumifloraCycle>(flora).unwrap().time_in_phase = 11.0;
        world.run_system_once(lumiflora_bloom_system).unwrap();
        assert_eq!(world.get::<LumifloraCycle>(flora).unwrap().phase, BloomPhase::Blooming);

        // Enough for hibernation
        world.get_mut::<LumifloraCycle>(flora).unwrap().time_in_phase = 6.0;
        world.run_system_once(lumiflora_bloom_system).unwrap();
        assert_eq!(world.get::<LumifloraCycle>(flora).unwrap().phase, BloomPhase::Hibernation);

        // Back to dormant
        world.get_mut::<LumifloraCycle>(flora).unwrap().time_in_phase = 6.0;
        world.run_system_once(lumiflora_bloom_system).unwrap();
        assert_eq!(world.get::<LumifloraCycle>(flora).unwrap().phase, BloomPhase::Dormant);

        let flora_branch = world.spawn((
            Flora { flora_type: FloraType::Lumiflora, ..Default::default() },
            LumifloraCycle { phase: BloomPhase::Blooming, time_in_phase: 0.0 },
        )).id();
        world.run_system_once(lumiflora_bloom_system).unwrap();
        assert_eq!(world.get::<LumifloraCycle>(flora_branch).unwrap().phase, BloomPhase::Blooming);
    }

    #[test]
    fn test_pop_bio_rhythm_sync_fast_path() {
        let mut world = World::new();

        let pop = world.spawn((
            Pop,
            GridPosition { x: 5, y: 5 },
            BioRhythmSync { is_synced: true, work_speed_multiplier: 1.5 },
            Needs { rest: 1.0, ..Default::default() },
        )).id();

        // No floras
        world.run_system_once(apply_bio_rhythm_aura).unwrap();

        let sync = world.get::<BioRhythmSync>(pop).unwrap();
        assert!(!sync.is_synced);
        assert_eq!(sync.work_speed_multiplier, 1.0);
    }

    #[test]
    fn test_pop_bio_rhythm_sync_dormant_and_out_of_range() {
        let mut world = World::new();

        let _flora = world.spawn((
            Flora { flora_type: FloraType::Lumiflora, ..Default::default() },
            LumifloraCycle { phase: BloomPhase::Dormant, time_in_phase: 0.0 },
            GridPosition { x: 5, y: 5 },
        )).id();

        let pop = world.spawn((
            Pop,
            GridPosition { x: 5, y: 5 },
            BioRhythmSync { is_synced: false, work_speed_multiplier: 1.0 },
            Needs { rest: 1.0, ..Default::default() },
        )).id();

        world.run_system_once(apply_bio_rhythm_aura).unwrap();

        let sync = world.get::<BioRhythmSync>(pop).unwrap();
        assert!(sync.is_synced);
        assert_eq!(sync.work_speed_multiplier, 1.0);

        // Move pop out of range
        world.get_mut::<GridPosition>(pop).unwrap().x = 100;

        world.run_system_once(apply_bio_rhythm_aura).unwrap();

        let sync2 = world.get::<BioRhythmSync>(pop).unwrap();
        assert!(!sync2.is_synced);
    }
}

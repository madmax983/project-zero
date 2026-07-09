use bevy::prelude::*;
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
    pub time_in_phase: u64,
}

#[derive(Component)]
pub struct BioRhythmSync {
    pub is_synced: bool,
    pub work_speed_multiplier: f32,
}

pub fn lumiflora_bloom_system(
    time: Res<SimulationTime>,
    mut query: Query<&mut LumifloraCycle, With<Flora>>,
) {
    // In our tests, time steps by large chunks, so we just calculate based on modulo time
    let cycle_length = 3000;

    for mut cycle in query.iter_mut() {
        let current_time_in_cycle = time.tick % cycle_length;

        cycle.phase = if current_time_in_cycle < 1000 {
            BloomPhase::Dormant
        } else if current_time_in_cycle < 2000 {
            BloomPhase::Blooming
        } else {
            BloomPhase::Hibernation
        };
        cycle.time_in_phase = current_time_in_cycle;
    }
}

pub fn apply_bio_rhythm_aura(
    flora_query: Query<(&LumifloraCycle, &GridPosition), With<Flora>>,
    mut pop_query: Query<(&GridPosition, &mut BioRhythmSync, &mut Needs), With<Pop>>,
) {
    // Reset all
    for (_, mut sync, _) in pop_query.iter_mut() {
        sync.is_synced = false;
        sync.work_speed_multiplier = 1.0;
    }

    // Apply aura for each pop
    for (pop_pos, mut sync, mut needs) in pop_query.iter_mut() {
        for (flora_cycle, flora_pos) in flora_query.iter() {
            let dx = pop_pos.x - flora_pos.x;
            let dy = pop_pos.y - flora_pos.y;
            if dx * dx + dy * dy <= 25 { // radius 5
                sync.is_synced = true;
                match flora_cycle.phase {
                    BloomPhase::Blooming => {
                        sync.work_speed_multiplier = 1.5;
                        // Prevent sleep decay while blooming
                    }
                    BloomPhase::Hibernation => {
                        sync.work_speed_multiplier = 0.1;
                        needs.rest = 0.0; // Force immediate sleep need
                    }
                    BloomPhase::Dormant => {
                        sync.work_speed_multiplier = 1.0;
                    }
                }
                break; // Pop is influenced, move to next pop
            }
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

    #[test]
    fn test_lumiflora_blooming_cycle() {
        let mut app = App::new();
        app.add_systems(Update, lumiflora_bloom_system);

        let flora = app.world_mut().spawn((
            Flora { flora_type: FloraType::Lumiflora, ..default() },
            LumifloraCycle { phase: BloomPhase::Dormant, time_in_phase: 0 },
        )).id();

        app.insert_resource(SimulationTime { tick: 0, ..default() });
        app.update();

        // Let's assume a system that advances LumifloraCycle time
        app.world_mut().resource_mut::<SimulationTime>().tick += 1000;
        app.update();

        let cycle = app.world().get::<LumifloraCycle>(flora).unwrap();
        assert_eq!(cycle.phase, BloomPhase::Blooming);
    }

    #[test]
    fn test_pop_bio_rhythm_sync() {
        let mut app = App::new();
        app.add_systems(Update, apply_bio_rhythm_aura);

        let flora = app.world_mut().spawn((
            Flora { flora_type: FloraType::Lumiflora, ..default() },
            LumifloraCycle { phase: BloomPhase::Blooming, time_in_phase: 0 },
            GridPosition { x: 5, y: 5 },
        )).id();

        let pop = app.world_mut().spawn((
            Pop,
            GridPosition { x: 5, y: 5 },
            BioRhythmSync { is_synced: false, work_speed_multiplier: 1.0 },
            Needs { rest: 1.0, ..default() },
        )).id();

        app.update();

        let sync = app.world().get::<BioRhythmSync>(pop).unwrap();
        assert!(sync.is_synced);
        assert!(sync.work_speed_multiplier > 1.0); // Blooming gives speed boost

        // Change flora to Hibernation phase
        app.world_mut().get_mut::<LumifloraCycle>(flora).unwrap().phase = BloomPhase::Hibernation;

        app.update();

        let sleep = app.world().get::<Needs>(pop).unwrap();
        assert!(sleep.rest < 0.1); // Forces sleep
    }
}

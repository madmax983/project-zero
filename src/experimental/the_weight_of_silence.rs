use bevy::prelude::*;

use crate::shared::time::SimulationTime;

#[derive(Component)]
pub struct ColonyNode {
    pub last_communication_tick: u64,
    pub isolation_level: f32,
}

#[derive(Component)]
pub struct IsolationResident(pub Entity);

#[derive(Component, Default)]
pub struct IsolationNeed(pub f32);

#[derive(Component)]
pub struct SilenceCult {
    pub colony_entity: Entity,
}

#[derive(Component)]
pub struct CultMember;

#[derive(Resource)]
pub struct SilenceCultConfig {
    pub isolation_increase_per_tick: f32,
    pub isolation_need_threshold: f32,
    pub cult_spawn_threshold: f32,
}

impl Default for SilenceCultConfig {
    fn default() -> Self {
        Self {
            isolation_increase_per_tick: 0.1,
            isolation_need_threshold: 50.0,
            cult_spawn_threshold: 500.0,
        }
    }
}

pub fn track_colony_isolation_system(
    tick: Option<Res<SimulationTime>>,
    config: Option<Res<SilenceCultConfig>>,
    mut colonies: Query<&mut ColonyNode>,
) {
    let increase_rate = config.map(|c| c.isolation_increase_per_tick).unwrap_or(0.1);

    if let Some(tick) = tick {
        for mut colony in colonies.iter_mut() {
            let time_since_comm = tick.tick.saturating_sub(colony.last_communication_tick);
            colony.isolation_level = (time_since_comm as f32) * increase_rate;
        }
    }
}

pub fn process_isolation_needs_system(
    config: Option<Res<SilenceCultConfig>>,
    colonies: Query<&ColonyNode>,
    mut pops: Query<(&IsolationResident, &mut IsolationNeed)>,
) {
    let threshold = config.map(|c| c.isolation_need_threshold).unwrap_or(50.0);

    for (resident_of, mut needs) in pops.iter_mut() {
        if let Ok(colony) = colonies.get(resident_of.0) {
            if colony.isolation_level > threshold {
                needs.0 += 1.0;
            }
        }
    }
}

pub fn spawn_silence_cult_system(
    mut commands: Commands,
    config: Option<Res<SilenceCultConfig>>,
    colonies: Query<(Entity, &ColonyNode)>,
    mut pops: Query<(Entity, &IsolationResident, &mut IsolationNeed), Without<CultMember>>,
    existing_cults: Query<&SilenceCult>,
) {
    let threshold = config.map(|c| c.cult_spawn_threshold).unwrap_or(500.0);

    for (colony_entity, colony) in colonies.iter() {
        if colony.isolation_level >= threshold {
            // Check if cult already exists for this colony
            let has_cult = existing_cults
                .iter()
                .any(|c| c.colony_entity == colony_entity);

            if !has_cult {
                commands.spawn(SilenceCult { colony_entity });
            }

            // Indoctrinate pops
            for (pop_entity, resident_of, needs) in pops.iter_mut() {
                if resident_of.0 == colony_entity && needs.0 >= 100.0 {
                    commands.entity(pop_entity).insert(CultMember);
                }
            }
        }
    }
}

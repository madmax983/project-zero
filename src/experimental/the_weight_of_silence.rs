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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_track_colony_isolation_system() {
        let mut app = App::new();
        app.insert_resource(SimulationTime {
            tick: 100,
            ..Default::default()
        });
        app.insert_resource(SilenceCultConfig {
            isolation_increase_per_tick: 0.5,
            ..Default::default()
        });
        app.add_systems(Update, track_colony_isolation_system);

        let ent = app
            .world_mut()
            .spawn(ColonyNode {
                last_communication_tick: 80,
                isolation_level: 0.0,
            })
            .id();

        app.update();

        let node = app.world().get::<ColonyNode>(ent).unwrap();
        // (100 - 80) * 0.5 = 10.0
        assert_eq!(node.isolation_level, 10.0);
    }

    #[test]
    fn test_process_isolation_needs_system() {
        let mut app = App::new();
        app.insert_resource(SilenceCultConfig {
            isolation_need_threshold: 40.0,
            ..Default::default()
        });
        app.add_systems(Update, process_isolation_needs_system);

        let colony_ent = app
            .world_mut()
            .spawn(ColonyNode {
                last_communication_tick: 0,
                isolation_level: 50.0, // Above 40.0
            })
            .id();

        let pop_ent = app
            .world_mut()
            .spawn((IsolationResident(colony_ent), IsolationNeed(10.0)))
            .id();

        app.update();

        let need = app.world().get::<IsolationNeed>(pop_ent).unwrap();
        assert_eq!(need.0, 11.0);
    }

    #[test]
    fn test_spawn_silence_cult_system() {
        let mut app = App::new();
        app.insert_resource(SilenceCultConfig {
            cult_spawn_threshold: 400.0,
            ..Default::default()
        });
        app.add_systems(Update, spawn_silence_cult_system);

        let colony_ent = app
            .world_mut()
            .spawn(ColonyNode {
                last_communication_tick: 0,
                isolation_level: 500.0, // Above 400.0
            })
            .id();

        let pop_ent = app
            .world_mut()
            .spawn((
                IsolationResident(colony_ent),
                IsolationNeed(150.0), // Above 100.0
            ))
            .id();

        app.update();

        // Verify Cult was spawned
        let mut cult_query = app.world_mut().query::<&SilenceCult>();
        let cults: Vec<_> = cult_query.iter(app.world()).collect();
        assert_eq!(cults.len(), 1);
        assert_eq!(cults[0].colony_entity, colony_ent);

        // Verify pop was indoctrinated
        assert!(app.world().get::<CultMember>(pop_ent).is_some());
    }

    #[test]
    fn test_process_isolation_needs_system_below_threshold() {
        let mut app = App::new();
        app.insert_resource(SilenceCultConfig {
            isolation_need_threshold: 40.0,
            ..Default::default()
        });
        app.add_systems(Update, process_isolation_needs_system);

        let colony_ent = app
            .world_mut()
            .spawn(ColonyNode {
                last_communication_tick: 0,
                isolation_level: 30.0, // Below 40.0
            })
            .id();

        let pop_ent = app
            .world_mut()
            .spawn((IsolationResident(colony_ent), IsolationNeed(10.0)))
            .id();

        app.update();

        let need = app.world().get::<IsolationNeed>(pop_ent).unwrap();
        assert_eq!(need.0, 10.0);
    }

    #[test]
    fn test_spawn_silence_cult_system_below_threshold() {
        let mut app = App::new();
        app.insert_resource(SilenceCultConfig {
            cult_spawn_threshold: 400.0,
            ..Default::default()
        });
        app.add_systems(Update, spawn_silence_cult_system);

        let colony_ent = app
            .world_mut()
            .spawn(ColonyNode {
                last_communication_tick: 0,
                isolation_level: 300.0, // Below 400.0
            })
            .id();

        let pop_ent = app
            .world_mut()
            .spawn((
                IsolationResident(colony_ent),
                IsolationNeed(150.0), // Above 100.0
            ))
            .id();

        app.update();

        // Verify Cult was NOT spawned
        let mut cult_query = app.world_mut().query::<&SilenceCult>();
        let cults: Vec<_> = cult_query.iter(app.world()).collect();
        assert_eq!(cults.len(), 0);

        // Verify pop was NOT indoctrinated
        assert!(app.world().get::<CultMember>(pop_ent).is_none());
    }

    #[test]
    fn test_spawn_silence_cult_system_already_exists() {
        let mut app = App::new();
        app.insert_resource(SilenceCultConfig {
            cult_spawn_threshold: 400.0,
            ..Default::default()
        });
        app.add_systems(Update, spawn_silence_cult_system);

        let colony_ent = app
            .world_mut()
            .spawn(ColonyNode {
                last_communication_tick: 0,
                isolation_level: 500.0, // Above 400.0
            })
            .id();

        app.world_mut().spawn(SilenceCult { colony_entity: colony_ent });

        app.update();

        // Verify Cult was not spawned again
        let mut cult_query = app.world_mut().query::<&SilenceCult>();
        let cults: Vec<_> = cult_query.iter(app.world()).collect();
        assert_eq!(cults.len(), 1);
    }
}

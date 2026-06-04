use crate::layer1::psychology::needs::Needs;
use bevy_ecs::prelude::*;

#[derive(Resource)]
pub struct SilenceCultConfig {
    pub isolation_growth_rate: f32,
    pub isolation_threshold_for_needs: f32,
    pub isolation_need_increase: f32,
    pub isolation_threshold_for_cult: f32,
    pub isolation_need_for_cult: f32,
}

impl Default for SilenceCultConfig {
    fn default() -> Self {
        Self {
            isolation_growth_rate: 0.1,
            isolation_threshold_for_needs: 50.0,
            isolation_need_increase: 1.0,
            isolation_threshold_for_cult: 500.0,
            isolation_need_for_cult: 100.0,
        }
    }
}

#[derive(Resource, Default)]
pub struct SimulationTick(pub u64);

#[derive(Component)]
pub struct ColonyNode {
    pub last_communication_tick: u64,
    pub isolation_level: f32,
}

#[derive(Component)]
pub struct ResidentOf(pub Entity);

#[derive(Component)]
pub struct SilenceCult {
    pub colony_entity: Entity,
}

#[derive(Component)]
pub struct CultMember;

pub fn track_colony_isolation_system(
    tick: Option<Res<SimulationTick>>,
    config: Option<Res<SilenceCultConfig>>,
    mut colonies: Query<&mut ColonyNode>,
) {
    let growth_rate = config.map_or(0.1, |c| c.isolation_growth_rate);
    if let Some(tick) = tick {
        for mut colony in colonies.iter_mut() {
            let time_since_comm = tick.0.saturating_sub(colony.last_communication_tick);
            colony.isolation_level = (time_since_comm as f32) * growth_rate;
        }
    }
}

pub fn process_isolation_needs_system(
    config: Option<Res<SilenceCultConfig>>,
    colonies: Query<&ColonyNode>,
    mut pops: Query<(&ResidentOf, &mut Needs)>,
) {
    let threshold = config
        .as_ref()
        .map_or(50.0, |c| c.isolation_threshold_for_needs);
    let increase = config.as_ref().map_or(1.0, |c| c.isolation_need_increase);

    for (resident_of, mut needs) in pops.iter_mut() {
        if let Ok(colony) = colonies.get(resident_of.0) {
            if colony.isolation_level > threshold {
                needs.isolation += increase;
            }
        }
    }
}

pub fn spawn_silence_cult_system(
    mut commands: Commands,
    config: Option<Res<SilenceCultConfig>>,
    colonies: Query<(Entity, &ColonyNode)>,
    mut pops: Query<(Entity, &ResidentOf, &Needs), Without<CultMember>>,
    existing_cults: Query<&SilenceCult>,
) {
    let cult_threshold = config
        .as_ref()
        .map_or(500.0, |c| c.isolation_threshold_for_cult);
    let pop_threshold = config.as_ref().map_or(100.0, |c| c.isolation_need_for_cult);

    for (colony_entity, colony) in colonies.iter() {
        if colony.isolation_level >= cult_threshold {
            // Check if cult already exists for this colony
            let has_cult = existing_cults
                .iter()
                .any(|c| c.colony_entity == colony_entity);

            if !has_cult {
                commands.spawn(SilenceCult { colony_entity });
            }

            // Indoctrinate pops
            for (pop_entity, resident_of, needs) in pops.iter_mut() {
                if resident_of.0 == colony_entity && needs.isolation >= pop_threshold {
                    commands.entity(pop_entity).insert(CultMember);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::pop::Pop;
    use bevy_app::{App, Update};

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_systems(
            Update,
            (
                track_colony_isolation_system,
                process_isolation_needs_system,
                spawn_silence_cult_system,
            ),
        );
        app
    }

    #[test]
    fn test_colony_isolation_increases_over_time() {
        let mut app = setup_app();

        let colony = app
            .world_mut()
            .spawn(ColonyNode {
                last_communication_tick: 0,
                isolation_level: 0.0,
            })
            .id();

        app.insert_resource(SimulationTick(1000));
        app.update();

        let colony_data = app.world().get::<ColonyNode>(colony).unwrap();
        assert!(
            colony_data.isolation_level > 0.0,
            "Isolation level should increase over time without communication"
        );
    }

    #[test]
    fn test_pops_gain_isolation_need() {
        let mut app = setup_app();

        let colony = app
            .world_mut()
            .spawn(ColonyNode {
                last_communication_tick: 0,
                isolation_level: 100.0,
            })
            .id();

        let pop = app
            .world_mut()
            .spawn((
                Pop,
                ResidentOf(colony),
                Needs {
                    isolation: 0.0,
                    ..Default::default()
                },
            ))
            .id();

        app.update();

        let needs = app.world().get::<Needs>(pop).unwrap();
        assert!(
            needs.isolation > 0.0,
            "Pop isolation need should increase if colony is highly isolated"
        );
    }

    #[test]
    fn test_silence_cult_spawns_at_high_isolation() {
        let mut app = setup_app();

        let colony = app
            .world_mut()
            .spawn(ColonyNode {
                last_communication_tick: 0,
                isolation_level: 500.0,
            })
            .id();

        let pop = app
            .world_mut()
            .spawn((
                Pop,
                ResidentOf(colony),
                Needs {
                    isolation: 100.0,
                    ..Default::default()
                },
            ))
            .id();

        app.update();

        let mut cult_query = app.world_mut().query::<&SilenceCult>();
        assert!(
            cult_query.iter(app.world()).count() > 0,
            "Silence Cult should spawn at high isolation"
        );

        assert!(
            app.world().get::<CultMember>(pop).is_some(),
            "Pop with high isolation need should join the Silence Cult"
        );
    }
}

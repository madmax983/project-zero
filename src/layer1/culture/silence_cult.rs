use bevy_ecs::prelude::*;

use crate::shared::time::SimulationTime;
use crate::layer1::tech::symbiotic_habitation::ResidentOf;
use crate::layer1::needs::Needs;

#[derive(Component)]
pub struct ColonyNode {
    pub last_communication_tick: u64,
    pub isolation_level: f32,
}

#[derive(Component)]
pub struct SilenceCult {
    pub colony_entity: Entity,
}

#[derive(Component)]
pub struct CultMember;

pub fn track_colony_isolation_system(
    tick: Option<Res<SimulationTime>>,
    mut colonies: Query<&mut ColonyNode>,
) {
    if let Some(tick) = tick {
        for mut colony in colonies.iter_mut() {
            let time_since_comm = tick.tick.saturating_sub(colony.last_communication_tick);
            colony.isolation_level = (time_since_comm as f32) * 0.1;
        }
    }
}

pub fn process_isolation_needs_system(
    colonies: Query<&ColonyNode>,
    mut pops: Query<(&ResidentOf, &mut Needs)>,
) {
    // Note: Re-using `leisure` to simulate isolation for this spec since `Needs` lacks an `isolation` field.
    // Low leisure triggers stress. Here we drain leisure as isolation grows.
    for (resident_of, mut needs) in pops.iter_mut() {
        if let Ok(colony) = colonies.get(resident_of.building) {
            if colony.isolation_level > 50.0 {
                 needs.leisure = (needs.leisure - 0.05).max(0.0);
            }
        }
    }
}

pub fn spawn_silence_cult_system(
    mut commands: Commands,
    colonies: Query<(Entity, &ColonyNode)>,
    mut pops: Query<(Entity, &ResidentOf, &Needs), Without<CultMember>>,
    existing_cults: Query<&SilenceCult>,
) {
    for (colony_entity, colony) in colonies.iter() {
        if colony.isolation_level >= 500.0 {
             let has_cult = existing_cults.iter().any(|c| c.colony_entity == colony_entity);

             if !has_cult {
                 commands.spawn(SilenceCult { colony_entity });
             }

             for (pop_entity, resident_of, needs) in pops.iter_mut() {
                 // Check if leisure is 0 (fully isolated/stressed)
                 if resident_of.building == colony_entity && needs.leisure <= 0.0 {
                     commands.entity(pop_entity).insert(CultMember);
                 }
             }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::App;
    use bevy_app::Update;
    use crate::layer1::pop::Pop;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_systems(Update, (
            track_colony_isolation_system,
            process_isolation_needs_system,
            spawn_silence_cult_system,
        ));
        app
    }

    #[test]
    fn test_colony_isolation_increases_over_time() {
        let mut app = setup_app();

        let colony = app.world_mut().spawn(ColonyNode {
            last_communication_tick: 0,
            isolation_level: 0.0,
        }).id();

        app.insert_resource(SimulationTime { tick: 1000, ..Default::default() });
        app.update();

        let colony_data = app.world().get::<ColonyNode>(colony).unwrap();
        assert!(colony_data.isolation_level > 0.0, "Isolation level should increase over time without communication");
    }

    #[test]
    fn test_pops_gain_isolation_need() {
        let mut app = setup_app();

        let colony = app.world_mut().spawn(ColonyNode {
            last_communication_tick: 0,
            isolation_level: 100.0,
        }).id();

        let mut pop_needs = Needs::default();
        pop_needs.leisure = 1.0;
        let pop = app.world_mut().spawn((
            Pop,
            ResidentOf { building: colony },
            pop_needs,
        )).id();

        app.update();

        let needs = app.world().get::<Needs>(pop).unwrap();
        assert!(needs.leisure < 1.0, "Pop leisure should decrease (simulating isolation) if colony is highly isolated");
    }

    #[test]
    fn test_silence_cult_spawns_at_high_isolation() {
        let mut app = setup_app();

        let colony = app.world_mut().spawn(ColonyNode {
            last_communication_tick: 0,
            isolation_level: 500.0,
        }).id();

        let mut pop_needs = Needs::default();
        pop_needs.leisure = 0.0;
        let pop = app.world_mut().spawn((
            Pop,
            ResidentOf { building: colony },
            pop_needs,
        )).id();

        app.update();

        let mut cult_query = app.world_mut().query::<&SilenceCult>();
        assert!(cult_query.iter(app.world()).count() > 0, "Silence Cult should spawn at high isolation");

        assert!(app.world().get::<CultMember>(pop).is_some(), "Pop with low leisure should join the Silence Cult");
    }
}

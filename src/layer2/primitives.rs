use bevy::prelude::*;
use crate::layer1::resources::ColonyResources;
use crate::layer1::pop::Pop;


#[derive(Component)]
pub struct PrimitiveCivilization {
    pub tech_level: TechLevel,
}

#[derive(Component)]
pub struct ObservationPost {
    pub active: bool,
}

#[derive(Component)]
pub struct SlaveMarker; // Spec used `Slave`, we will define it here if it's not in pop

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TechLevel {
    StoneAge,
    BronzeAge,
    IronAge,
}

#[derive(Event, Debug, Clone)]
pub struct InvasionEvent {
    pub target: Entity,
    pub aggressor: FactionId,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FactionId {
    Player,
    AI(u32),
}

pub fn process_observation_posts(
    posts: Query<&ObservationPost, With<PrimitiveCivilization>>,
    mut resources: ResMut<ColonyResources>,
) {
    for post in posts.iter() {
        if post.active {
            resources.knowledge += 5.0; // Minimal science gain
        }
    }
}

pub fn resolve_primitive_invasions(
    mut commands: Commands,
    mut events: EventReader<InvasionEvent>,
    civs: Query<&PrimitiveCivilization>,
) {
    for event in events.read() {
        if civs.get(event.target).is_ok() {
            // Remove the civilization
            commands.entity(event.target).remove::<PrimitiveCivilization>();

            // Spawn some slaves
            for _ in 0..5 {
                commands.spawn((Pop, SlaveMarker));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer2::generation::Planet;

    #[test]
    fn test_primitive_planet_generates_science_with_observation_post() {
        let mut app = App::new();
        app.insert_resource(ColonyResources::default());
        let _planet = app.world_mut().spawn((
            Planet,
            PrimitiveCivilization { tech_level: TechLevel::BronzeAge },
            ObservationPost { active: true }
        )).id();

        app.add_systems(Update, process_observation_posts);
        app.update();

        let resources = app.world().get_resource::<ColonyResources>().unwrap();
        assert!(resources.knowledge > 0.0, "Observation post should generate science points");
    }

    #[test]
    fn test_invading_primitive_civilization_spawns_slave_pops() {
        let mut app = App::new();
        app.add_event::<InvasionEvent>();
        app.insert_resource(ColonyResources::default());
        let planet = app.world_mut().spawn((
            Planet,
            PrimitiveCivilization { tech_level: TechLevel::IronAge },
        )).id();

        let mut events = app.world_mut().resource_mut::<Events<InvasionEvent>>();
        events.send(InvasionEvent { target: planet, aggressor: FactionId::Player });

        app.add_systems(Update, resolve_primitive_invasions);
        app.update();

        let planet_state = app.world().get::<PrimitiveCivilization>(planet);
        assert!(planet_state.is_none(), "Civilization should be removed after successful invasion");

        let slave_count = app.world_mut().query_filtered::<&Pop, With<SlaveMarker>>().iter(app.world()).count();
        assert!(slave_count > 0, "Invasion should result in new slave pops");
    }
}

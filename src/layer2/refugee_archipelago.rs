use crate::layer2::fleet::Fleet;
use bevy::prelude::*;

#[derive(Component)]
pub struct RefugeeFlotilla {
    pub population: u32,
    pub target_colony: Option<Entity>,
}

#[derive(Event, Debug, Clone)]
pub struct CollapseEvent {
    pub colony: Entity,
    pub survivor_count: u32,
}

#[derive(Event, Debug, Clone)]
pub struct AsylumRequestEvent {
    pub flotilla: Entity,
    pub target: Entity,
    pub population: u32,
}

pub fn spawn_refugees_on_collapse_system(
    mut commands: Commands,
    mut collapse_events: EventReader<CollapseEvent>,
) {
    for event in collapse_events.read() {
        if event.survivor_count > 0 {
            commands.spawn((
                Fleet,
                RefugeeFlotilla {
                    population: event.survivor_count,
                    target_colony: None, // Will be set by navigation AI later
                },
            ));
        }
    }
}

pub fn refugee_arrival_system(
    query: Query<(Entity, &RefugeeFlotilla)>,
    mut asylum_events: EventWriter<AsylumRequestEvent>,
) {
    for (entity, flotilla) in query.iter() {
        // If they have a target, assume they've arrived for MVP
        if let Some(target) = flotilla.target_colony {
            asylum_events.send(AsylumRequestEvent {
                flotilla: entity,
                target,
                population: flotilla.population,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Component)]
    struct Colony;

    #[derive(Component)]
    struct Faction;

    #[test]
    fn test_colony_collapse_spawns_refugee_flotilla() {
        let mut app = App::new();
        app.add_event::<CollapseEvent>();
        app.add_systems(Update, spawn_refugees_on_collapse_system);

        let collapsed_colony = app.world_mut().spawn((Colony, Faction)).id();

        app.world_mut()
            .resource_mut::<Events<CollapseEvent>>()
            .send(CollapseEvent {
                colony: collapsed_colony,
                survivor_count: 5000,
            });

        app.update();

        // Verify Flotilla spawned
        let mut flotilla_query = app.world_mut().query::<(&Fleet, &RefugeeFlotilla)>();
        let mut found = false;
        for (_fleet, flotilla) in flotilla_query.iter(app.world()) {
            if flotilla.population == 5000 {
                found = true;
            }
        }

        assert!(
            found,
            "A refugee flotilla should spawn with the survivor population upon colony collapse."
        );
    }

    #[test]
    fn test_colony_collapse_ignores_zero_survivors() {
        let mut app = App::new();
        app.add_event::<CollapseEvent>();
        app.add_systems(Update, spawn_refugees_on_collapse_system);

        let collapsed_colony = app.world_mut().spawn((Colony, Faction)).id();

        app.world_mut()
            .resource_mut::<Events<CollapseEvent>>()
            .send(CollapseEvent {
                colony: collapsed_colony,
                survivor_count: 0,
            });

        app.update();

        // Verify no Flotilla spawned
        let mut flotilla_query = app.world_mut().query::<(&Fleet, &RefugeeFlotilla)>();
        assert_eq!(flotilla_query.iter(app.world()).count(), 0);
    }

    #[test]
    fn test_refugee_flotilla_requests_asylum_at_colony() {
        let mut app = App::new();
        app.add_event::<AsylumRequestEvent>();
        app.add_systems(Update, refugee_arrival_system);

        let target_colony = app.world_mut().spawn((Colony, Faction)).id();

        // Spawn a flotilla that has 'arrived' at the colony
        let flotilla = app
            .world_mut()
            .spawn((
                Fleet,
                RefugeeFlotilla {
                    population: 2000,
                    target_colony: Some(target_colony),
                },
            ))
            .id();

        app.update();

        // Verify AsylumRequest event fired
        let asylum_events = app.world().resource::<Events<AsylumRequestEvent>>();
        let mut reader = asylum_events.get_cursor();
        let mut found = false;
        for event in reader.read(asylum_events) {
            if event.flotilla == flotilla && event.target == target_colony {
                found = true;
            }
        }

        assert!(
            found,
            "Refugee flotilla arriving at a colony should trigger an Asylum Request event."
        );
    }

    #[test]
    fn test_refugee_flotilla_no_target_colony() {
        let mut app = App::new();
        app.add_event::<AsylumRequestEvent>();
        app.add_systems(Update, refugee_arrival_system);

        let _flotilla = app
            .world_mut()
            .spawn((
                Fleet,
                RefugeeFlotilla {
                    population: 2000,
                    target_colony: None,
                },
            ))
            .id();

        app.update();

        let asylum_events = app.world().resource::<Events<AsylumRequestEvent>>();
        let mut reader = asylum_events.get_cursor();
        assert!(reader.read(asylum_events).next().is_none());
    }
}

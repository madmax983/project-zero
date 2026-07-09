use crate::layer1::colony::{CollapseEvent, Colony};
use crate::layer1::diplomacy::AsylumRequestEvent;
use crate::layer2::fleet::Fleet;
use bevy::prelude::*;

#[derive(Component)]
pub struct RefugeeFlotilla {
    pub population: u32,
    pub target_colony: Option<Entity>,
}

#[derive(Event, Debug)]
pub struct AsylumRejectedEvent {
    pub flotilla: Entity,
}

// Minimal stub for Piracy in this context
#[derive(Component)]
pub struct PirateFleet;

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

// Navigation AI: find nearest/random colony
pub fn refugee_navigation_ai_system(
    mut query: Query<&mut RefugeeFlotilla, Without<PirateFleet>>,
    colonies: Query<Entity, With<Colony>>,
) {
    for mut flotilla in query.iter_mut() {
        if flotilla.target_colony.is_none() {
            // Pick the first colony as target
            if let Some(colony) = colonies.iter().next() {
                flotilla.target_colony = Some(colony);
            }
        }
    }
}

// Depletion AI
pub fn refugee_depletion_system(
    mut query: Query<(Entity, &mut RefugeeFlotilla)>,
    mut commands: Commands,
) {
    for (entity, mut flotilla) in query.iter_mut() {
        if flotilla.population > 0 {
            flotilla.population -= 1; // slow depletion
        }
        if flotilla.population == 0 {
            commands.entity(entity).despawn();
        }
    }
}

// Piracy conversion if rejected
pub fn refugee_rejection_system(
    mut commands: Commands,
    mut rejection_events: EventReader<AsylumRejectedEvent>,
) {
    for event in rejection_events.read() {
        commands.entity(event.flotilla).remove::<RefugeeFlotilla>();
        commands.entity(event.flotilla).insert(PirateFleet);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::colony::{CollapseEvent, Colony};
    use crate::layer1::diplomacy::{AsylumRequestEvent, Faction};
    use crate::layer2::fleet::Fleet;

    #[test]
    fn test_colony_collapse_spawns_refugee_flotilla() {
        let mut app = App::new();
        app.add_event::<CollapseEvent>();
        app.add_systems(Update, spawn_refugees_on_collapse_system);

        let collapsed_colony = app.world_mut().spawn((Colony, Faction::Independent)).id();

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
    fn test_refugee_flotilla_requests_asylum_at_colony() {
        let mut app = App::new();
        app.add_event::<AsylumRequestEvent>();
        app.add_systems(Update, refugee_arrival_system);

        let target_colony = app.world_mut().spawn((Colony, Faction::Player)).id();

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
    fn test_refugee_navigation() {
        let mut app = App::new();
        app.add_systems(Update, refugee_navigation_ai_system);

        let target_colony = app.world_mut().spawn(Colony).id();
        let flotilla = app
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

        let flotilla_comp = app.world().get::<RefugeeFlotilla>(flotilla).unwrap();
        assert_eq!(flotilla_comp.target_colony, Some(target_colony));
    }

    #[test]
    fn test_refugee_depletion() {
        let mut app = App::new();
        app.add_systems(Update, refugee_depletion_system);

        let flotilla = app
            .world_mut()
            .spawn((
                Fleet,
                RefugeeFlotilla {
                    population: 2,
                    target_colony: None,
                },
            ))
            .id();

        app.update();

        let flotilla_comp = app.world().get::<RefugeeFlotilla>(flotilla).unwrap();
        assert_eq!(flotilla_comp.population, 1);

        app.update();

        assert!(
            app.world().get::<RefugeeFlotilla>(flotilla).is_none(),
            "Flotilla should despawn when population reaches 0"
        );
    }

    #[test]
    fn test_refugee_piracy_conversion() {
        let mut app = App::new();
        app.add_event::<AsylumRejectedEvent>();
        app.add_systems(Update, refugee_rejection_system);

        let flotilla = app
            .world_mut()
            .spawn((
                Fleet,
                RefugeeFlotilla {
                    population: 2000,
                    target_colony: None,
                },
            ))
            .id();

        app.world_mut()
            .resource_mut::<Events<AsylumRejectedEvent>>()
            .send(AsylumRejectedEvent { flotilla });

        app.update();

        assert!(app.world().get::<RefugeeFlotilla>(flotilla).is_none());
        assert!(app.world().get::<PirateFleet>(flotilla).is_some());
    }
}

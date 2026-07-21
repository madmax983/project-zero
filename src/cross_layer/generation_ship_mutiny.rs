use crate::layer2::fleet::InTransit;
use crate::layer3::fleets::generation_ship::GenerationShip;
use bevy::prelude::*;

#[derive(Component)]
pub struct GenerationShipAge {
    pub age_in_years: u32,
}

#[derive(Component)]
pub struct Faction {
    pub name: String,
    pub host_ship: Entity,
}

#[derive(Component, Default)]
pub struct Radicalization {
    pub level: f32,
}

#[derive(Event)]
pub struct MutinyEvent {
    pub ship: Entity,
}

pub fn generation_ship_radicalization_system(
    ships: Query<&GenerationShipAge, With<GenerationShip>>,
    mut factions: Query<(&Faction, &mut Radicalization)>,
) {
    for (faction, mut radicalization) in factions.iter_mut() {
        if let Ok(ship_age) = ships.get(faction.host_ship) {
            // Slowly increase Radicalization based on ship age
            radicalization.level += ship_age.age_in_years as f32 * 0.01;
        }
    }
}

pub fn evaluate_mutiny_system(
    mut events: EventWriter<MutinyEvent>,
    mut ships: Query<(Entity, &mut InTransit), With<GenerationShip>>,
    factions: Query<(&Faction, &Radicalization)>,
) {
    for (faction, radicalization) in factions.iter() {
        if radicalization.level >= 100.0 {
            events.send(MutinyEvent {
                ship: faction.host_ship,
            });

            // Alter ship destination on mutiny
            if let Ok((_ship_entity, mut transit)) = ships.get_mut(faction.host_ship) {
                transit.destination = Entity::PLACEHOLDER;
            }
        }
    }
}

/// INT-982: Bridges MutinyEvent to AddChronicleEvent (Chronicle)
pub fn generation_ship_mutiny_chronicle_bridge(
    mut events: bevy_ecs::event::EventReader<
        crate::cross_layer::generation_ship_mutiny::MutinyEvent,
    >,
    mut chronicle_events: bevy_ecs::event::EventWriter<
        crate::layer1::core::chronicle::AddChronicleEvent,
    >,
) {
    for _event in events.read() {
        chronicle_events.send(crate::layer1::core::chronicle::AddChronicleEvent {
            text: "A mutiny has occurred on the generation ship! Descendants of the original crew have radicalized and altered the mission parameters.".to_string(),
            importance: crate::layer1::core::chronicle::EventImportance::Major,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generation_ship_generates_radical_factions_over_time() {
        let mut app = App::new();
        app.add_systems(Update, generation_ship_radicalization_system);

        // Spawn a Generation Ship entity
        let ship_entity = app
            .world_mut()
            .spawn((
                GenerationShip,
                GenerationShipAge { age_in_years: 100 },
                crate::layer2::fleet::InTransit {
                    origin: Entity::PLACEHOLDER,
                    destination: Entity::PLACEHOLDER,
                    progress: 0.0,
                    duration: 100.0,
                },
            ))
            .id();

        // Spawn a faction aboard the ship
        let faction_entity = app
            .world_mut()
            .spawn((
                Faction {
                    name: "Original Mission".to_string(),
                    host_ship: ship_entity,
                },
                Radicalization { level: 0.0 },
            ))
            .id();

        app.update();

        let radicalization = app.world().get::<Radicalization>(faction_entity).unwrap();
        assert!(
            radicalization.level > 0.0,
            "Factions on generation ships should radicalize over time."
        );
    }

    #[test]
    fn test_high_radicalization_triggers_mutiny_event() {
        let mut app = App::new();
        app.add_event::<MutinyEvent>();
        app.add_systems(Update, evaluate_mutiny_system);

        let ship_entity = app
            .world_mut()
            .spawn((
                GenerationShip,
                GenerationShipAge { age_in_years: 200 },
                crate::layer2::fleet::InTransit {
                    origin: Entity::PLACEHOLDER,
                    destination: Entity::from_raw(12345),
                    progress: 0.0,
                    duration: 100.0,
                },
            ))
            .id();

        // Spawn a highly radical faction
        app.world_mut().spawn((
            Faction {
                name: "Ship Worshippers".to_string(),
                host_ship: ship_entity,
            },
            Radicalization { level: 100.0 }, // threshold met
        ));

        app.update();

        let events = app.world().resource::<Events<MutinyEvent>>();
        let mut reader = events.get_cursor();
        let mutiny_events: Vec<_> = reader.read(events).collect();

        assert_eq!(
            mutiny_events.len(),
            1,
            "High radicalization should trigger a MutinyEvent."
        );
        assert_eq!(mutiny_events[0].ship, ship_entity);
    }
}

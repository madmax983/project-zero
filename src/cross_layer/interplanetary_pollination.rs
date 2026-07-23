const GRAVITY_THRESHOLD: f32 = 1.0;
const MIN_FLORA_AMOUNT: f32 = 500.0;
const SPORE_STRENGTH_RATIO: f32 = 0.05;

#[allow(unused_imports)]
use bevy::prelude::*;

#[derive(Component)]
pub struct Planet {
    pub gravity: f32,
}

#[derive(Component)]
pub struct FloraCultivation {
    pub amount: f32,
    pub is_gmo: bool,
}

#[derive(Component)]
pub struct Biome {
    pub invasive_flora: f32,
}

#[derive(Event)]
pub struct SporeReleaseEvent {
    pub source_planet: Entity,
    pub target_planet: Option<Entity>,
    pub spore_strength: f32,
}

pub fn spore_escape_system(
    query: Query<(Entity, &Planet, &FloraCultivation)>,
    mut event_writer: EventWriter<SporeReleaseEvent>,
) {
    for (entity, planet, cultivation) in query.iter() {
        if planet.gravity < GRAVITY_THRESHOLD
            && cultivation.is_gmo
            && cultivation.amount > MIN_FLORA_AMOUNT
        {
            event_writer.send(SporeReleaseEvent {
                source_planet: entity,
                target_planet: None,
                spore_strength: cultivation.amount * SPORE_STRENGTH_RATIO,
            });
        }
    }
}

pub fn spore_infection_system(
    mut events: EventReader<SporeReleaseEvent>,
    mut query: Query<&mut Biome>,
) {
    for event in events.read() {
        if let Some(target) = event.target_planet {
            if let Ok(mut biome) = query.get_mut(target) {
                biome.invasive_flora += event.spore_strength;
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[allow(unused_imports)]
    use bevy::prelude::*;

    #[test]
    fn test_spore_escape_to_orbit() {
        let mut app = App::new();
        app.add_systems(Update, spore_escape_system);

        let planet_entity = app
            .world_mut()
            .spawn((
                Planet { gravity: 0.5 },
                FloraCultivation {
                    amount: 1000.0,
                    is_gmo: true,
                },
            ))
            .id();

        app.add_event::<SporeReleaseEvent>();

        app.update();

        let events = app
            .world()
            .get_resource::<Events<SporeReleaseEvent>>()
            .unwrap();
        let mut reader = events.get_cursor();
        let release_events: Vec<_> = reader.read(events).collect();

        assert_eq!(release_events.len(), 1);
        assert_eq!(release_events[0].source_planet, planet_entity);
    }

    #[test]
    fn test_spore_infection_on_neighbor() {
        let mut app = App::new();
        app.add_systems(Update, spore_infection_system);

        let source_planet = app.world_mut().spawn_empty().id();
        let target_planet = app
            .world_mut()
            .spawn((
                Planet { gravity: 1.0 },
                Biome {
                    invasive_flora: 0.0,
                },
            ))
            .id();

        app.add_event::<SporeReleaseEvent>();
        app.world_mut().send_event(SporeReleaseEvent {
            source_planet,
            target_planet: Some(target_planet),
            spore_strength: 50.0,
        });

        app.update();

        let biome = app.world().get::<Biome>(target_planet).unwrap();
        assert_eq!(biome.invasive_flora, 50.0);
    }
}

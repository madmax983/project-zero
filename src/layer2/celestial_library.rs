use bevy_ecs::prelude::*;
use rand::Rng;

#[derive(Resource)]
pub struct LibrarySpawnConfig {
    pub spawn_chance: f32,
}

impl Default for LibrarySpawnConfig {
    fn default() -> Self {
        Self { spawn_chance: 0.05 }
    }
}

#[derive(Component)]
pub struct CelestialLibrary {
    pub required_donation: u32,
}

#[derive(Event)]
pub struct LibraryDonationEvent {
    pub library: Entity,
    pub resources_donated: u32,
}

#[derive(Clone, Debug)]
pub struct LibraryLogEntry {
    pub reward_granted: bool,
}

#[derive(Resource, Default)]
pub struct LibraryEventLog {
    pub events: Vec<LibraryLogEntry>,
}

pub fn spawn_library_system(
    mut commands: Commands,
    config: Res<LibrarySpawnConfig>,
    query: Query<&CelestialLibrary>,
) {
    if query.is_empty() {
        let mut rng = rand::thread_rng();
        if rng.gen::<f32>() < config.spawn_chance {
            commands.spawn(CelestialLibrary {
                required_donation: 1000,
            });
        }
    }
}

pub fn process_library_donations_system(
    mut events: EventReader<LibraryDonationEvent>,
    mut log: ResMut<LibraryEventLog>,
    query: Query<&CelestialLibrary>,
) {
    for event in events.read() {
        if let Ok(library) = query.get(event.library) {
            if event.resources_donated >= library.required_donation {
                log.events.push(LibraryLogEntry { reward_granted: true });
            } else {
                log.events.push(LibraryLogEntry { reward_granted: false });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::App;
    use bevy_app::Update;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_systems(Update, (
            spawn_library_system,
            process_library_donations_system,
        ));
        app.insert_resource(LibrarySpawnConfig { spawn_chance: 1.0 });
        app.init_resource::<LibraryEventLog>();
        app.add_event::<LibraryDonationEvent>();
        app
    }

    #[test]
    fn test_library_spawns() {
        let mut app = setup_app();
        app.update();

        let library_count = app.world_mut().query::<&CelestialLibrary>().iter(app.world()).count();
        assert_eq!(library_count, 1, "The Celestial Library should spawn based on the config.");
    }

    #[test]
    fn test_library_accepts_donation() {
        let mut app = setup_app();
        app.update();

        // Find the library
        let library_entity = app.world_mut().query_filtered::<Entity, With<CelestialLibrary>>().single(app.world());

        // Add a donation
        app.world_mut().send_event(LibraryDonationEvent {
            library: library_entity,
            resources_donated: 1000,
        });

        app.update();

        let log = app.world().resource::<LibraryEventLog>();
        assert_eq!(log.events.len(), 1, "A donation event should be logged.");
        assert_eq!(log.events[0].reward_granted, true, "A reward should be granted for a sufficient donation.");
    }
}

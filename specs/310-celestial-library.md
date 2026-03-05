# Specification 310: The Celestial Library

## 1. Overview
This feature introduces "The Celestial Library," an ancient and neutral orbital structure (Layer 2) that randomly appears. It offers profound technological blueprints or ancient lore in exchange for a massive "Donation" of specific resources or highly educated Pops (who are permanently removed).

## 2. Dependencies
- `chronicle` system (`src/layer1/chronicle.rs`) for historical tracking.
- Layer 1/2 Cross-layer event system.
- `tech` system for unlocking blueprints (assuming it exists, otherwise general resource rewards).

## 3. RED Phase: Tests First

```rust
// src/layer2/celestial_library.rs
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_systems(Update, (
            spawn_library_system,
            process_library_donations_system,
        ));
        app.insert_resource(LibrarySpawnConfig { spawn_chance: 1.0 });
        app.init_resource::<LibraryEventLog>();
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
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/layer2/celestial_library.rs
use bevy::prelude::*;

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
    if query.iter().count() == 0 {
        if config.spawn_chance >= 1.0 { // Simplified chance
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
```

## 5. REFACTOR Phase: Quality & Design
- **Integration**: The library needs to hook into the actual resource manager (`ColonyResources`) to deduct donations properly.
- **Lore**: Integrate with `chronicle` to log when the library appears and when a donation is made.
- **Complexity**: The required donation should dynamically scale based on colony wealth or progress.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >=85% for `celestial_library.rs`.
- [ ] Library spawns according to configuration.
- [ ] Library grants a logged reward when the donation threshold is met.

## 7. Technical Guidance
- Ensure that the spawning system uses a reliable random generator (e.g., `rand` crate or Bevy's built-in utilities) instead of a simple float check in the final implementation.
- Consider adding a lifespan to the library so it disappears if ignored.

## 8. Questions
*Builder: add questions here if spec is unclear.*

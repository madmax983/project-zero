# The Black Box

## 1. Overview
**Layer:** Cross-layer (Meta)
**Fantasy:** "This civilization may fall, but our knowledge will remain."
**Mechanic:** A buildable "Archive" structure that saves Tech/Lore state. If the colony fails (Game Over), a future colony on the same world (next run) can discover and decrypt it to regain lost progress or bonuses.
**Emergence:** Players realizing a run is doomed switch objectives from "Survival" to "Preservation," frantically uploading data while the base burns around them.
**Tension:** Spend resources saving yourself (now) or saving your legacy (future)?

## 2. Dependencies
- `004-basic-building` (Building placement and destruction)
- `010-chronicle-system` (Lore and event tracking)
- `011-tech-tree-backend` (Tech unlocks to save)
- Basic file I/O or persistent state management across simulation instances.

## 3. RED Phase: Tests First

```rust
// specs/414-the-black-box.rs

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::tech::TechState;
    use crate::layer1::chronicle::Chronicle;
    use crate::meta::archive::{Archive, BlackBoxData, save_black_box_system, load_black_box_system};

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<TechState>();
        app.init_resource::<Chronicle>();
        app.init_resource::<BlackBoxData>();
        app.add_systems(Update, save_black_box_system);
        app
    }

    #[test]
    fn test_black_box_preserves_tech_on_game_over() {
        // Arrange
        let mut app = setup_app();

        // Unlock a tech
        app.world_mut().resource_mut::<TechState>().unlock_tech("fusion_power".to_string());

        // Place an Archive building
        app.world_mut().spawn((
            Building { building_type: BuildingType::Archive },
            Archive { data_uploaded: true },
        ));

        // Act: trigger game over and save
        app.world_mut().send_event(crate::meta::GameOverEvent);
        app.update();

        // Assert: The BlackBoxData resource should now contain the unlocked tech
        let black_box = app.world().resource::<BlackBoxData>();
        assert!(black_box.unlocked_techs.contains("fusion_power"));
    }

    #[test]
    fn test_black_box_requires_archive_building() {
        let mut app = setup_app();

        app.world_mut().resource_mut::<TechState>().unlock_tech("fusion_power".to_string());

        // No Archive building spawned!

        app.world_mut().send_event(crate::meta::GameOverEvent);
        app.update();

        // Assert: Without an archive, nothing is saved.
        let black_box = app.world().resource::<BlackBoxData>();
        assert!(!black_box.unlocked_techs.contains("fusion_power"));
    }

    #[test]
    fn test_new_run_can_discover_black_box() {
        let mut app = setup_app();
        app.add_systems(Update, load_black_box_system);

        // Simulate a new run where previous data exists
        app.world_mut().resource_mut::<BlackBoxData>().unlocked_techs.push("fusion_power".to_string());

        // Spawn a 'RuinedArchive' that the player can interact with
        let ruin_id = app.world_mut().spawn(crate::meta::archive::RuinedArchive).id();

        // Act: interact with the ruin
        app.world_mut().send_event(crate::meta::archive::DecryptArchiveEvent { entity: ruin_id });
        app.update();

        // Assert: Tech is restored
        let tech_state = app.world().resource::<TechState>();
        assert!(tech_state.is_unlocked("fusion_power"));

        // Assert: Ruin is consumed/changed state
        assert!(app.world().get::<crate::meta::archive::RuinedArchive>(ruin_id).is_none());
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/meta/archive.rs
use bevy::prelude::*;
use crate::layer1::building::{Building, BuildingType};
use crate::layer1::tech::TechState;
use crate::layer1::chronicle::Chronicle;
use serde::{Serialize, Deserialize};

#[derive(Component)]
pub struct Archive {
    pub data_uploaded: bool,
}

#[derive(Component)]
pub struct RuinedArchive;

#[derive(Resource, Default, Serialize, Deserialize)]
pub struct BlackBoxData {
    pub unlocked_techs: Vec<String>,
    // Could add chronicle events here later
}

#[derive(Event)]
pub struct GameOverEvent;

#[derive(Event)]
pub struct DecryptArchiveEvent {
    pub entity: Entity,
}

pub fn save_black_box_system(
    mut events: EventReader<GameOverEvent>,
    archive_query: Query<&Archive>,
    tech_state: Res<TechState>,
    mut black_box: ResMut<BlackBoxData>,
) {
    for _ in events.read() {
        // Only save if an Archive exists and has uploaded data
        let mut can_save = false;
        for archive in archive_query.iter() {
            if archive.data_uploaded {
                can_save = true;
                break;
            }
        }

        if can_save {
            black_box.unlocked_techs = tech_state.unlocked.clone();
            // In a real implementation, this would trigger actual file I/O to disk.
        }
    }
}

pub fn load_black_box_system(
    mut events: EventReader<DecryptArchiveEvent>,
    mut commands: Commands,
    black_box: Res<BlackBoxData>,
    mut tech_state: ResMut<TechState>,
) {
    for event in events.read() {
        // Restore tech
        for tech in &black_box.unlocked_techs {
            tech_state.unlock_tech(tech.clone());
        }

        // Consume the ruin
        commands.entity(event.entity).remove::<RuinedArchive>();
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Disk I/O:** The `BlackBoxData` resource needs actual persistent storage (e.g., writing to a `.json` or `.dat` file in the user's save directory) so it survives across process restarts, not just ECS world resets.
- **Upload Mechanic:** The `data_uploaded` boolean is a placeholder. Implementing an active "Upload" job/action where pops must physically work at the Archive to secure data before the base falls adds more gameplay tension.
- **Partial Recovery:** Instead of saving everything perfectly, maybe data is corrupted and the next run only gets a tech *boost* (e.g., 50% research progress) rather than an instant unlock.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] `BlackBoxData` correctly captures state only when an active `Archive` is present upon `GameOverEvent`.
- [ ] Discovering a `RuinedArchive` correctly restores data from `BlackBoxData`.

## 7. Technical Guidance
- Create a new module `src/meta/archive.rs` for this cross-run persistence.
- You will need to mock the file saving/loading in tests, or rely on Bevy's resource system for the unit tests and implement the actual file I/O in a separate platform/startup system.
- Hook `GameOverEvent` into whatever existing system detects colony failure.

## 8. Questions
*Builder: add questions here if spec is unclear.*

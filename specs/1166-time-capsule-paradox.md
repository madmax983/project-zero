# 1166: The Time-Capsule Paradox

## 1. Overview
The Time-Capsule Paradox allows the player to receive a desperate message and advanced technology from the future (or past), instantly skipping years of research. However, doing so splits the timeline and spawns a "Mirror Empire" on the edge of the galaxy that is heavily armed with the exact same technology and intent on destroying the player.

## 2. Dependencies
- Cross-layer support (Layer 1 -> 3)
- Anomaly System (`src/layer2/anomalies.rs` or similar)
- Empire Generation System (`src/layer3/empire_generation.rs` or similar)
- Technology System (`src/layer1/tech.rs`)

## 3. RED Phase: Tests First

```rust
// src/layer3/time_capsule_tests.rs
#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use super::*;

    #[test]
    fn test_time_capsule_grants_tech() {
        let mut app = App::new();
        app.add_systems(Update, process_time_capsule_opening);

        let player_empire = app.world_mut().spawn(TechProgress { points: 100.0 }).id();

        // Spawn capsule event
        app.world_mut().send_event(TimeCapsuleOpenedEvent {
            empire_id: player_empire,
            tech_bonus: 5000.0,
        });

        app.update();

        let progress = app.world().get::<TechProgress>(player_empire).unwrap();
        assert_eq!(progress.points, 5100.0, "Time capsule should grant massive tech bonus");
    }

    #[test]
    fn test_time_capsule_spawns_mirror_empire() {
        let mut app = App::new();
        app.add_systems(Update, process_time_capsule_opening);

        let player_empire = app.world_mut().spawn(TechProgress { points: 100.0 }).id();

        // Spawn capsule event
        app.world_mut().send_event(TimeCapsuleOpenedEvent {
            empire_id: player_empire,
            tech_bonus: 5000.0,
        });

        app.update();

        let mut query = app.world_mut().query::<(&MirrorEmpire, &TechProgress)>();
        let mut found = false;
        for (_, mirror_tech) in query.iter(app.world()) {
            found = true;
            assert_eq!(mirror_tech.points, 5100.0, "Mirror Empire should have equivalent technology");
        }

        assert!(found, "Mirror Empire should have been spawned");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// The SIMPLEST code that makes tests pass
// No optimization, no extras
// Just enough to turn RED → GREEN
```

## 5. REFACTOR Phase: Quality & Design
- **Event Handling:** Ensure `TimeCapsuleOpenedEvent` correctly hooks into both `Layer 1` (tech updates) and `Layer 3` (spawning a new faction).
- **Mirror Empire Identity:** The newly spawned empire should have an ideology and name reflecting its paradoxical nature (e.g., "The Alternative Timeline").
- **Chronicle Integration:** Spawning the Mirror Empire and receiving the capsule should be monumental events in the Chronicle.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Opening the capsule grants a massive technology boost to the player.
- [ ] Opening the capsule spawns a hostile "Mirror Empire" faction.
- [ ] The Mirror Empire inherits the same technological level as the player's updated state.

## 7. Technical Guidance
- Integrate with existing Layer 3 faction generation tools. You may need to create a helper to clone the player's tech state for the Mirror Empire.
- The capsule itself can be triggered via the anomaly system when surveying Layer 2 nodes.

## 8. Questions
*Builder: add questions here if spec is unclear.*

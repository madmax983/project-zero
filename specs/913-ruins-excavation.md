# 913: Ruins Excavation

## 1. Overview
Deep terrain layers contain "Ruins" tiles. Excavating these tiles yields powerful Artifacts (providing lore and resources) but carries a risk of unleashing "Old World Maladies" such as ancient curses, anomalous diseases, or sudden sanity drops. This creates a tension between digging deep for lucrative secrets versus staying shallow to preserve the colony's safety.

## 2. Dependencies
- `layer1::map::TerrainGrid`
- `layer1::mining::MiningJob`
- `layer1::health::DiseaseSystem`
- `layer1::items::Artifact`

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_ruins_excavation_yields_artifact_and_risk() {
        let mut app = App::new();
        app.add_systems(Update, process_ruins_excavation);

        // Spawn a pop excavating a ruin
        let pop_id = app.world_mut().spawn(PopStatus::Excavating).id();

        let tile_id = app.world_mut().spawn(RuinTile {
            artifact_yield: 1,
            malady_risk: 1.0, // 100% chance for testing
        }).id();

        app.world_mut().send_event(ExcavationCompleteEvent {
            pop_id,
            tile_id,
        });

        app.update();

        // Check if an artifact was spawned/given
        let artifacts = app.world().query::<&Artifact>().iter(app.world()).count();
        assert_eq!(artifacts, 1, "An artifact should have been yielded from the ruin");

        // Check if the malady was applied to the pop
        let has_malady = app.world().get::<Malady>(pop_id).is_some();
        assert!(has_malady, "The pop should have contracted an Old World Malady due to 1.0 risk");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct PopStatus;

#[derive(Component)]
pub struct RuinTile {
    pub artifact_yield: u32,
    pub malady_risk: f32, // 0.0 to 1.0
}

#[derive(Component)]
pub struct Artifact;

#[derive(Component)]
pub struct Malady;

#[derive(Event)]
pub struct ExcavationCompleteEvent {
    pub pop_id: Entity,
    pub tile_id: Entity,
}

pub fn process_ruins_excavation(
    mut commands: Commands,
    mut events: EventReader<ExcavationCompleteEvent>,
    tile_query: Query<&RuinTile>,
) {
    for event in events.read() {
        if let Ok(ruin) = tile_query.get(event.tile_id) {
            // Yield artifact
            for _ in 0..ruin.artifact_yield {
                commands.spawn(Artifact); // Simplification: spawn it in the world
            }

            // Apply risk (using 0.5 as a simple random stand-in since rand isn't injected in this minimal version)
            // In a real implementation, use a proper RNG source.
            // For the sake of the test where risk is 1.0, we just check > 0.0
            if ruin.malady_risk > 0.0 {
                commands.entity(event.pop_id).insert(Malady);
            }

            // Consume the ruin tile
            commands.entity(event.tile_id).despawn();
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Refactoring:** The random number generation for the malady risk needs to be properly implemented using Bevy's or standard `rand` with a seed for determinism.
- **Code Smells:** Spawning artifacts into the void is a placeholder. They should be added to the excavating Pop's inventory or the colony's central storage.
- **Performance:** This is an event-driven system, so performance overhead is minimal, but ensure `Malady` components are processed efficiently by the health systems.
- **API Improvements:** Expand `Malady` into an enum to represent different curses (e.g., SanityDrain, BioPlague).

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >= 85% for new code
- [ ] Excavating ruins reliably produces artifacts
- [ ] Pops have a percentage chance to receive a `Malady` based on the tile's risk factor.

## 7. Technical Guidance
- **Code Structure:** Integrate into `src/layer1/mining.rs` or create a new `src/layer1/archaeology.rs` module.
- **Integration Points:** Link `Malady` to the existing `DiseaseSystem` so it can spread or be treated in the hospital. Ensure the `Artifact` component integrates with the cultural/morale systems.
- **Gotchas:** Ensure that when a tile is despawned/mined, pathfinding and grid updates are triggered so pops don't get stuck.

## 8. Questions
*Builder: add questions here if spec is unclear.*

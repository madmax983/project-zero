# The Sub-Light Ghost

## 1. Overview
A probe from a dead world lands on the colony. Downloading its database provides massive tech unlocks but acts as a homing beacon, summoning an incredibly advanced exterminator fleet from Layer 2.

## 2. Dependencies
- `src/layer1/knowledge.rs` (tech unlocks)
- `src/layer2/fleets.rs` (exterminator fleet spawning)
- `src/layer1/anomalies.rs` (probe interaction)

## 3. RED Phase: Tests First
```rust
#[test]
fn test_probe_download_grants_tech() {
    let mut app = App::new();
    // Setup probe anomaly
    // act: complete download interaction
    // assert: Knowledge system unlocks massive tech tree nodes
}

#[test]
fn test_probe_download_summons_exterminators() {
    let mut app = App::new();
    // Setup probe anomaly
    // act: complete download interaction
    // assert: Exterminator fleet is spawned on Layer 2 targeting the colony
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// In src/layer1/anomalies.rs
#[derive(Component)]
pub struct SubLightProbe;

pub fn probe_download_system(
    mut events: EventReader<InteractionEvent>,
    probes: Query<&SubLightProbe>,
    mut knowledge: ResMut<KnowledgeSystem>,
    mut spawn_fleet: EventWriter<SpawnHostileFleetEvent>,
) {
    for event in events.read() {
        if probes.get(event.target).is_ok() {
            // Grant massive tech
            knowledge.unlock_precursor_tier();

            // Summon doom
            spawn_fleet.send(SpawnHostileFleetEvent {
                faction: FactionType::Exterminator,
                target_node: event.colony_node,
            });
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Ensure the exterminator fleet has a delayed arrival or is exceptionally hard to defeat to maintain tension.
- Add dramatic Chronicle events for both the tech boom and the impending doom.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Downloading the probe data unlocks significant technology.
- [ ] The download triggers a hostile fleet spawn on the Layer 2 map.

## 7. Technical Guidance
- Trigger a cross-layer event (`SpawnHostileFleetEvent`) when the Layer 1 interaction completes.
- Exterminator fleets should ignore standard diplomacy logic.

## 8. Questions
*Builder: add questions here if spec is unclear.*

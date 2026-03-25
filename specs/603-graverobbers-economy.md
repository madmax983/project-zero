# The Grave-Robber's Economy

## 1. Overview
A cross-layer mechanic involving plundering ancient artifacts. Removing artifacts from catacombs provides immense wealth on Layer 3 markets but increases a hidden "Vengeance" tracker. When maximized, a precursor empire declares a holy war of extermination against the colony.

## 2. Dependencies
- `src/layer1/artifacts.rs` (or similar for looting)
- `src/layer3/diplomacy.rs` (precursor empire faction and war declaration)

## 3. RED Phase: Tests First
```rust
#[test]
fn test_artifact_removal_increases_vengeance() {
    let mut app = App::new();
    // Setup a precursor artifact in catacombs
    // act: Pop removes artifact
    // assert: Vengeance tracker for Precursor faction increases
}

#[test]
fn test_max_vengeance_triggers_holy_war() {
    let mut app = App::new();
    // Setup Vengeance tracker near max
    // act: Pop removes final artifact
    // assert: Precursor faction diplomacy stance becomes War
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// In src/layer1/artifacts.rs
#[derive(Component)]
pub struct SacredArtifact {
    pub faction_id: Entity,
}

#[derive(Resource, Default)]
pub struct VengeanceTracker {
    pub level: f32,
}

pub fn artifact_looted_system(
    mut events: EventReader<ItemLootedEvent>,
    artifacts: Query<&SacredArtifact>,
    mut vengeance: ResMut<VengeanceTracker>,
) {
    for event in events.read() {
        if artifacts.get(event.item).is_ok() {
            vengeance.level += 10.0;
        }
    }
}

// In src/layer3/diplomacy.rs
pub fn vengeance_war_system(
    vengeance: Res<VengeanceTracker>,
    mut diplomacy: ResMut<DiplomacyState>,
    precursor: Res<PrecursorFaction>,
) {
    if vengeance.level >= 100.0 {
        diplomacy.set_stance(precursor.entity, DiplomaticStance::War);
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Add UI warning indicating the rising anger of the precursor empire.
- Introduce an event when Holy War is declared to record it in the Chronicle.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Looting a sacred artifact increases a vengeance tracker.
- [ ] High vengeance triggers a war declaration from a precursor faction.

## 7. Technical Guidance
- Tie artifact looting events into the global tracking resource.
- Check `VengeanceTracker` during Layer 3 diplomacy ticks.

## 8. Questions
*Builder: add questions here if spec is unclear.*

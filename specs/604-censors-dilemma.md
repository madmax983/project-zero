# The Censor's Dilemma

## 1. Overview
When a disaster occurs, players can use an "Information Blackout" edict to suppress the news, preventing a colony-wide morale drop. However, the true story circulates as a "Banned Rumor." If heard, trust in the Governor plummets, causing far more unrest.

## 2. Dependencies
- `src/layer1/chronicle.rs` (events and rumors)
- `src/layer1/morale.rs` (trust and morale impact)
- `src/layer1/edicts.rs` (Information Blackout edict)

## 3. RED Phase: Tests First
```rust
#[test]
fn test_blackout_edict_prevents_initial_morale_drop() {
    let mut app = App::new();
    // Setup active Information Blackout
    // act: Trigger Disaster event
    // assert: No immediate Morale drop occurs
}

#[test]
fn test_banned_rumor_destroys_trust() {
    let mut app = App::new();
    // Setup a Pop
    // act: Pop receives a Banned Rumor
    // assert: Pop's Trust metric significantly drops
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// In src/layer1/rumors.rs
#[derive(Clone)]
pub struct BannedRumor;

pub fn blackout_suppression_system(
    mut disasters: EventReader<DisasterEvent>,
    edicts: Res<ActiveEdicts>,
    mut rumors: EventWriter<SpawnRumorEvent>,
) {
    for disaster in disasters.read() {
        if edicts.has("Information Blackout") {
            rumors.send(SpawnRumorEvent::new(BannedRumor));
            // Prevent standard morale drop
        } else {
            // Apply normal morale drop
        }
    }
}

pub fn banned_rumor_impact_system(
    mut events: EventReader<RumorHeardEvent>,
    mut pops: Query<&mut Trust>,
) {
    for event in events.read() {
        if let RumorType::Banned(_) = event.rumor_type {
            if let Ok(mut trust) = pops.get_mut(event.listener) {
                trust.value -= 50.0;
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Integrate `BannedRumor` cleanly into the existing rumor web system.
- Show trust drops in the UI inspector for affected Pops.
- Add an achievement for surviving a Banned Rumor cascade.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Information Blackout edict suppresses immediate disaster morale drops.
- [ ] Banned Rumors drastically reduce Trust when heard.

## 7. Technical Guidance
- Add `BannedRumor` to the `RumorType` enum.
- Intercept the `DisasterEvent` pipeline to check for the edict before applying morale drops.

## 8. Questions
*Builder: add questions here if spec is unclear.*

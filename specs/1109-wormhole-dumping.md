# 1109 - Wormhole Dumping

## 1. Overview
Making your garbage someone else"s problem. A high-tech "Disposal Gate" voids waste instantly. It goes to a random location in the galaxy (Layer 3). High usage increases "Diplomatic Threat" with random factions who are receiving your trash.

## 2. Dependencies
- Trade System (`specs/039-trade-system.md`)
- Diplomacy System
- Waste Management

## 3. RED Phase: Tests First
```rust
#[test]
fn test_disposal_gate_removes_waste() {
    let mut app = App::new();
    app.add_plugins(DisposalPlugin);
    let gate = app.world_mut().spawn(DisposalGate).id();
    app.world_mut().send_event(DumpWasteEvent { gate, amount: 100.0 });
    app.update();

    // Assert waste is removed (assuming a resource or component tracks total colony waste)
    let waste = app.world().resource::<ColonyWaste>();
    assert_eq!(waste.amount, 0.0);
}

#[test]
fn test_dumping_increases_diplomatic_threat() {
    let mut app = App::new();
    app.add_plugins((DisposalPlugin, DiplomacyPlugin));

    let gate = app.world_mut().spawn(DisposalGate).id();
    app.world_mut().send_event(DumpWasteEvent { gate, amount: 500.0 });
    app.update();

    // Assert a diplomatic threat event or modifier was generated
    let threat_events = app.world().resource::<Events<DiplomaticThreatEvent>>();
    assert!(!threat_events.is_empty());
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
#[derive(Component)]
pub struct DisposalGate;
#[derive(Event)]
pub struct DumpWasteEvent { pub gate: Entity, pub amount: f32 }
// Systems for dumping and threat...
```

## 5. REFACTOR Phase: Quality & Design
- Add cooldowns to gate usage.
- Randomize faction threat distribution.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >=85% for new code.
- [ ] Specific feature behavior verified

## 7. Technical Guidance
- Use existing diplomatic threat systems.

## 8. Questions
*Builder: add questions here if spec is unclear.*

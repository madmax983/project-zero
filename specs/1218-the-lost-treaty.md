# 1218: The Lost Treaty

## 1. Overview
**Layer:** 3

**Fantasy:** Bureaucracy from the grave. A piece of paper that stops a fleet.

**Mechanic:** You discover an ancient legal claim to a sector. Enforcing it gives you "Legitimacy" (Diplomacy buff) but angers the current occupants. Ignoring it looks weak.

**Emergence:** You find a deed to the Warlord's home system. Do you evict him with a lawyer?

**Tension:** Law (Legitimacy) vs. Reality (War).

## 2. Dependencies
- Diplomacy system
- Artifact/Discovery system

## 3. RED Phase: Tests First
```rust
#[test]
fn test_enforcing_treaty_grants_legitimacy_and_angers_occupants() {
    let mut app = App::new();
    app.add_systems(Update, process_treaty_decisions);

    let mut diplomacy = DiplomacyData::default();
    diplomacy.set_relation(FactionId::Warlords, 50.0);
    app.world_mut().insert_resource(diplomacy);
    app.world_mut().insert_resource(Economy { influence: 10.0 });
    app.world_mut().insert_resource(Events::<TreatyDecisionEvent>::default());

    // Enforce the treaty against the Warlords
    app.world_mut().send_event(TreatyDecisionEvent {
        faction_target: FactionId::Warlords,
        decision: TreatyDecision::Enforce,
    });

    app.update();

    let diplomacy = app.world().resource::<DiplomacyData>();
    let economy = app.world().resource::<Economy>();

    // Legitimacy (Influence) goes up
    assert!(economy.influence > 10.0);
    // Relations go down
    assert!(diplomacy.get_relation(FactionId::Warlords) < 50.0);
}

#[test]
fn test_ignoring_treaty_loses_legitimacy() {
    let mut app = App::new();
    app.add_systems(Update, process_treaty_decisions);

    app.world_mut().insert_resource(DiplomacyData::default());
    app.world_mut().insert_resource(Economy { influence: 50.0 });
    app.world_mut().insert_resource(Events::<TreatyDecisionEvent>::default());

    // Ignore the treaty
    app.world_mut().send_event(TreatyDecisionEvent {
        faction_target: FactionId::Warlords,
        decision: TreatyDecision::Ignore,
    });

    app.update();

    let economy = app.world().resource::<Economy>();

    // Legitimacy (Influence) goes down for looking weak
    assert!(economy.influence < 50.0);
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
fn process_treaty_decisions(
    mut events: EventReader<TreatyDecisionEvent>,
    mut diplomacy: ResMut<DiplomacyData>,
    mut economy: ResMut<Economy>,
) {
    for ev in events.read() {
        match ev.decision {
            TreatyDecision::Enforce => {
                economy.influence += 50.0;
                let current_rel = diplomacy.get_relation(ev.faction_target);
                diplomacy.set_relation(ev.faction_target, current_rel - 100.0);
            }
            TreatyDecision::Ignore => {
                economy.influence -= 20.0;
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Hook the discovery of the treaty into the Layer 1 anomaly scanning system.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >=85% for new code

## 7. Technical Guidance
- Ensure events are processed correctly in the update schedule.

## 8. Questions
*Builder: add questions here if spec is unclear.*

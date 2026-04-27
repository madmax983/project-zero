# 1210: Weaponized Tourism

## 1. Overview
**Layer:** Cross-layer

**Fantasy:** Killing them with kindness.

**Mechanic:** You send your own "Tourist" pops to rival colonies. They pay well but are programmed to be "Difficult" (complain, break things, spread contrary Ethics). If the rival harms them, you get a Casus Belli.

**Emergence:** You send a wave of "Food Critics" to a starving enemy colony. They eat the reserve rations and complain about the texture, causing a riot that topples the enemy government.

**Tension:** Soft Power (culture victory) vs. Hostage risk (your pops are there).

## 2. Dependencies
- Faction system
- Diplomacy system
- Pop Needs

## 3. RED Phase: Tests First
```rust
#[test]
fn test_difficult_tourist_increases_unrest_in_host_colony() {
    let mut app = App::new();
    app.add_systems(Update, process_tourist_behavior);

    // Host colony unrest
    app.world_mut().insert_resource(Unrest { level: 0.0 });

    // Spawn a tourist
    app.world_mut().spawn((
        Pop,
        Tourist { host_faction: FactionId::Rival, is_difficult: true },
    ));

    app.update();

    // Difficult tourists raise unrest
    let unrest = app.world().resource::<Unrest>();
    assert!(unrest.level > 0.0);
}

#[test]
fn test_tourist_harm_grants_casus_belli() {
    let mut app = App::new();
    app.add_systems(Update, process_tourist_harm);

    // Setup diplomacy
    app.world_mut().insert_resource(DiplomacyData::default());
    app.world_mut().insert_resource(Events::<PopDeathEvent>::default());

    // Spawn a tourist from OUR faction in the RIVAL faction
    let tourist = app.world_mut().spawn((
        Pop,
        Tourist { host_faction: FactionId::Rival, is_difficult: true },
    )).id();

    // Tourist dies
    app.world_mut().send_event(PopDeathEvent { target: tourist, cause: DeathCause::Murder });

    app.update();

    let diplomacy = app.world().resource::<DiplomacyData>();
    // We now have a Casus Belli against the Rival
    assert!(diplomacy.has_casus_belli(FactionId::Rival));
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
fn process_tourist_behavior(
    query: Query<&Tourist>,
    mut unrest: ResMut<Unrest>,
) {
    for tourist in query.iter() {
        if tourist.is_difficult {
            unrest.level += 1.0;
        }
    }
}

fn process_tourist_harm(
    mut death_events: EventReader<PopDeathEvent>,
    query: Query<&Tourist>,
    mut diplomacy: ResMut<DiplomacyData>,
) {
    for ev in death_events.read() {
        if let Ok(tourist) = query.get(ev.target) {
            // Harmed in foreign land!
            diplomacy.add_casus_belli(tourist.host_faction, "Harm to Citizens");
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Create specific "Tourist Actions" (like "Complain" or "Hog Resources") via the Utility AI rather than just applying flat unrest every tick.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >=85% for new code

## 7. Technical Guidance
- Integrate into the Factions and Diplomacy resources.

## 8. Questions
*Builder: add questions here if spec is unclear.*

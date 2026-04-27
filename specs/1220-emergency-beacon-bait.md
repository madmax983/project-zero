# 1220: Emergency Beacon Bait

## 1. Overview
**Layer:** 2

**Fantasy:** The spider in the web.

**Mechanic:** You can deploy a fake "Distress Beacon" in deep space. It attracts "Heroes" (Friendly/Neutral ships) and "Scavengers" (Pirates). You can ambush the pirates for bounty, or ambush the heroes for loot.

**Emergence:** You set a trap for pirates. A "Hospital Ship" answers the beacon instead. Do you let them go and waste the trap, or...

**Tension:** Altruism (saving real distress) vs. Predation (faking it).

## 2. Dependencies
- Layer 2 Fleet/Ship system
- Event bus

## 3. RED Phase: Tests First
```rust
#[test]
fn test_fake_beacon_attracts_ships() {
    let mut app = App::new();
    app.add_systems(Update, process_fake_beacons);

    app.world_mut().insert_resource(Events::<ShipArrivalEvent>::default());

    // Spawn a fake beacon
    app.world_mut().spawn((
        Beacon { is_fake: true, active: true },
        GridPosition { x: 50, y: 50 },
    ));

    app.update();

    // A ship should be attracted (spawned/arrived)
    let events = app.world().resource::<Events<ShipArrivalEvent>>();
    let mut reader = events.get_reader();
    assert!(reader.read(events).count() > 0);
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
fn process_fake_beacons(
    query: Query<&Beacon>,
    mut arrivals: EventWriter<ShipArrivalEvent>,
) {
    for beacon in query.iter() {
        if beacon.is_fake && beacon.active {
            arrivals.send(ShipArrivalEvent { ship_type: ShipType::Random });
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Create actual logic to determine if the arriving ship is a Pirate or a Hero, perhaps using a `rand::Rng` or based on local sector crime rates.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >=85% for new code

## 7. Technical Guidance
- Ensure events are processed correctly in the update schedule.

## 8. Questions
*Builder: add questions here if spec is unclear.*

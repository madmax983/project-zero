# 1208: The Cadet Branch

## 1. Overview
**Layer:** 3 -> 1

**Fantasy:** Babysitting the Emperor's nephew.

**Mechanic:** You accept "Noble Scions" from the Homeworld. They have terrible stats and "Snob" traits but come with a monthly "Allowance" (Funding) from their rich families. If they die, funding stops and relations tank.

**Emergence:** You build a luxurious, safe playground for the idiots just to keep the funding flowing, while the real workers live in squalor.

**Tension:** Free money vs. Incompetent/High-maintenance population.

## 2. Dependencies
- Base ECS system
- Faction relations
- Economy/Funding system

## 3. RED Phase: Tests First
```rust
#[test]
fn test_noble_scion_grants_allowance() {
    let mut app = App::new();
    app.add_systems(Update, process_scion_allowances);

    // Setup economy
    app.world_mut().insert_resource(Economy { funds: 100.0 });

    // Spawn a scion
    app.world_mut().spawn((
        Pop,
        NobleScion { allowance: 50.0, faction_id: FactionId::Empire },
    ));

    app.update();

    let economy = app.world().resource::<Economy>();
    assert_eq!(economy.funds, 150.0);
}

#[test]
fn test_noble_scion_death_tanks_relations() {
    let mut app = App::new();
    app.add_systems(Update, process_scion_deaths);

    // Setup relations
    let mut factions = FactionRelations::default();
    factions.set_relation(FactionId::Empire, 50.0);
    app.world_mut().insert_resource(factions);
    app.world_mut().insert_resource(Events::<PopDeathEvent>::default());

    // Scion dies
    let scion = app.world_mut().spawn((
        Pop,
        NobleScion { allowance: 50.0, faction_id: FactionId::Empire },
    )).id();

    app.world_mut().send_event(PopDeathEvent { target: scion, cause: DeathCause::Starvation });

    app.update();

    let factions = app.world().resource::<FactionRelations>();
    let relation = factions.get_relation(FactionId::Empire);
    // Huge relations penalty
    assert!(relation < 0.0);
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
fn process_scion_allowances(
    query: Query<&NobleScion>,
    mut economy: ResMut<Economy>,
) {
    for scion in query.iter() {
        economy.funds += scion.allowance;
    }
}

fn process_scion_deaths(
    mut death_events: EventReader<PopDeathEvent>,
    query: Query<&NobleScion>,
    mut factions: ResMut<FactionRelations>,
) {
    for ev in death_events.read() {
        if let Ok(scion) = query.get(ev.target) {
            let current = factions.get_relation(scion.faction_id);
            factions.set_relation(scion.faction_id, current - 100.0); // Tank relations
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Add Snob trait that prevents them from doing manual labor jobs.
- Refactor the allowance payment to be monthly/periodic instead of every tick.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >=85% for new code

## 7. Technical Guidance
- Integrate carefully with the Layer 3 Faction system.
- Utilize the existing `PopDeathEvent`.

## 8. Questions
*Builder: add questions here if spec is unclear.*

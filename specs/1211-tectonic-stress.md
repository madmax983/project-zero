# 1211: Tectonic Stress

## 1. Overview
**Layer:** 1

**Fantasy:** The ground remembers every bomb you dropped.

**Mechanic:** A global "Stress" meter for the crust. Mining, Explosions, and Heavy Industry increase it. It naturally dissipates slowly. If it hits 100%, a "Mega-Quake" occurs. Players can intentionally trigger small "Relief Quakes" to lower the meter safely.

**Emergence:** The meter is at 90%. A raid starts. You want to use grenades, but that will trigger the Mega-Quake. You have to fight hand-to-hand to save the city.

**Tension:** Controlled damage (Relief Quakes) vs. Uncontrolled catastrophe (The Big One).

## 2. Dependencies
- Environment system
- Event bus

## 3. RED Phase: Tests First
```rust
#[test]
fn test_explosions_increase_tectonic_stress() {
    let mut app = App::new();
    app.add_systems(Update, process_stress_events);

    app.world_mut().insert_resource(TectonicStress { level: 0.0 });
    app.world_mut().insert_resource(Events::<ExplosionEvent>::default());

    // Trigger explosion
    app.world_mut().send_event(ExplosionEvent { pos: GridPosition { x: 10, y: 10 }, radius: 5.0 });

    app.update();

    let stress = app.world().resource::<TectonicStress>();
    assert!(stress.level > 0.0);
}

#[test]
fn test_mega_quake_triggers_at_max_stress() {
    let mut app = App::new();
    app.add_systems(Update, check_mega_quake);

    app.world_mut().insert_resource(TectonicStress { level: 100.0 });
    app.world_mut().insert_resource(Events::<MegaQuakeEvent>::default());

    app.update();

    // Event should fire
    let events = app.world().resource::<Events<MegaQuakeEvent>>();
    let mut reader = events.get_reader();
    let iter: Vec<_> = reader.read(events).collect();
    assert_eq!(iter.len(), 1);

    // Stress resets
    let stress = app.world().resource::<TectonicStress>();
    assert_eq!(stress.level, 0.0);
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
fn process_stress_events(
    mut explosions: EventReader<ExplosionEvent>,
    mut stress: ResMut<TectonicStress>,
) {
    for ev in explosions.read() {
        stress.level += ev.radius * 2.0; // arbitrary scaling
    }
}

fn check_mega_quake(
    mut stress: ResMut<TectonicStress>,
    mut quake_events: EventWriter<MegaQuakeEvent>,
) {
    if stress.level >= 100.0 {
        quake_events.send(MegaQuakeEvent);
        stress.level = 0.0;
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Add passive dissipation logic.
- Hook MegaQuake into the `DamageEvent` pipeline for all structures.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >=85% for new code

## 7. Technical Guidance
- Integrate into the Environment and Combat logic.

## 8. Questions
*Builder: add questions here if spec is unclear.*

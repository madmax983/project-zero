# 1214: Biometric Drift

## 1. Overview
**Layer:** 1

**Fantasy:** The machine remembers who you *were*, not who you *are*. The veteran is a stranger to his own home.

**Mechanic:** Pops' biometric data (used for locks/terminals) degrades as they Age or gain Scars/Trauma. Must be "Recalibrated" (Admin action). If ignored, they get "Access Denied" errors.

**Emergence:** Your scarred war hero runs for the armory during a raid, but the door rejects him because his face doesn't match the file. He dies banging on the door.

**Tension:** High security (Biometrics) vs. Maintenance (Drift).

## 2. Dependencies
- Biometric Security / Access Control system
- Pop Aging / Traits

## 3. RED Phase: Tests First
```rust
#[test]
fn test_aging_increases_biometric_drift() {
    let mut app = App::new();
    app.add_systems(Update, process_biometric_drift);

    // Create an old pop with a biometric profile
    let pop = app.world_mut().spawn((
        Pop,
        Age { ticks: 100000 },
        BiometricProfile { drift: 0.0 },
    )).id();

    app.update();

    // Drift increases with age
    let profile = app.world().get::<BiometricProfile>(pop).unwrap();
    assert!(profile.drift > 0.0);
}

#[test]
fn test_high_drift_causes_access_denied() {
    let mut app = App::new();
    app.add_systems(Update, evaluate_door_access);

    // Create a pop with high drift
    let pop = app.world_mut().spawn((
        Pop,
        BiometricProfile { drift: 100.0 },
    )).id();

    // Create a secure door
    let door = app.world_mut().spawn((
        Door { is_locked: true, requires_biometrics: true },
    )).id();

    // Request access
    app.world_mut().insert_resource(Events::<AccessRequestEvent>::default());
    app.world_mut().insert_resource(Events::<AccessDeniedEvent>::default());
    app.world_mut().send_event(AccessRequestEvent { requester: pop, target: door });

    app.update();

    // Access is denied
    let events = app.world().resource::<Events<AccessDeniedEvent>>();
    let mut reader = events.get_reader();
    assert_eq!(reader.read(events).count(), 1);
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
fn process_biometric_drift(
    mut query: Query<(&Age, &mut BiometricProfile)>,
) {
    for (age, mut profile) in query.iter_mut() {
        profile.drift = (age.ticks as f32 / 1000.0).min(100.0); // Arbitrary scaling
    }
}

fn evaluate_door_access(
    mut requests: EventReader<AccessRequestEvent>,
    mut denials: EventWriter<AccessDeniedEvent>,
    pop_query: Query<&BiometricProfile>,
    door_query: Query<&Door>,
) {
    for req in requests.read() {
        if let Ok(door) = door_query.get(req.target) {
            if door.requires_biometrics {
                if let Ok(profile) = pop_query.get(req.requester) {
                    if profile.drift > 50.0 { // Threshold
                        denials.send(AccessDeniedEvent { requester: req.requester, door: req.target });
                    }
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Implement the "Recalibrate" action in the Utility AI for Admin workers.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >=85% for new code

## 7. Technical Guidance
- Integrate into `src/layer1/access_control.rs` and the `Age` component.

## 8. Questions
*Builder: add questions here if spec is unclear.*

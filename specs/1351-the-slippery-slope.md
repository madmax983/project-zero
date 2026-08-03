# 1351: The Slippery Slope

## 1. Overview
**Layer:** Cross-layer

**Fantasy:** We did what we had to do.

**Mechanic:** "Desensitization" tracker. Committing atrocities (Cannibalism, Purges) reduces the Stress penalty for future atrocities. But it also raises the threshold for positive Mood buffs. The colony becomes numb.

**Emergence:** You survive the winter by eating the dead. Next winter, no one complains about eating the dead. But they also don't care when you build a statue. They are hollow.

**Tension:** Survival resilience vs. Loss of humanity.

## 2. Dependencies
- Taboo/Atrocity event system
- Morale / Stress modifiers

## 3. RED Phase: Tests First
```rust
#[test]
fn test_atrocity_increases_desensitization() {
    let mut app = App::new();
    app.add_systems(Update, process_atrocities);

    app.world_mut().insert_resource(Desensitization { level: 0.0 });
    app.world_mut().insert_resource(Events::<AtrocityEvent>::default());

    // Send an atrocity
    app.world_mut().send_event(AtrocityEvent { severity: 10.0 });

    app.update();

    let d_level = app.world().resource::<Desensitization>().level;
    assert!(d_level > 0.0);
}

#[test]
fn test_high_desensitization_dampens_morale_buffs() {
    let mut app = App::new();
    app.add_systems(Update, apply_morale_buffs);

    // Colony is numb
    app.world_mut().insert_resource(Desensitization { level: 1.0 }); // 100% numb
    app.world_mut().insert_resource(Events::<MoraleBuffEvent>::default());

    let pop = app.world_mut().spawn((
        Pop,
        Morale { value: 50.0 },
    )).id();

    // Send a massive buff
    app.world_mut().send_event(MoraleBuffEvent { target: pop, amount: 50.0 });

    app.update();

    let morale = app.world().get::<Morale>(pop).unwrap();
    // The buff should be heavily dampened
    assert!(morale.value < 100.0);
    assert_eq!(morale.value, 50.0); // 100% dampening means no effect
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
fn process_atrocities(
    mut events: EventReader<AtrocityEvent>,
    mut desens: ResMut<Desensitization>,
) {
    for ev in events.read() {
        desens.level = (desens.level + (ev.severity * 0.01)).min(1.0);
    }
}

fn apply_morale_buffs(
    mut events: EventReader<MoraleBuffEvent>,
    mut query: Query<&mut Morale>,
    desens: Res<Desensitization>,
) {
    for ev in events.read() {
        if let Ok(mut morale) = query.get_mut(ev.target) {
            let actual_buff = ev.amount * (1.0 - desens.level);
            morale.value += actual_buff;
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Expand the logic so `Desensitization` also dampens Stress penalties (the beneficial part of the slope).

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >=85% for new code

## 7. Technical Guidance
- Link to the `TabooEvent` system.

## 8. Questions
*Builder: add questions here if spec is unclear.*

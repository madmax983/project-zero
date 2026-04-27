# 1221: The 'Good' War

## 1. Overview
**Layer:** 3 -> 1

**Fantasy:** War is good for business.

**Mechanic:** Your economy relies on "War Profiteering". Factories get bonuses when the galaxy is at war. If peace breaks out, your economy crashes ("Recession"). You must conduct "False Flag" operations to keep the war going.

**Emergence:** You sabotage peace talks between two rival empires just to keep your ammunition factory running at 100% efficiency.

**Tension:** Perpetual Conflict (Profit) vs. Stagnant Peace (Safety).

## 2. Dependencies
- Diplomacy System
- Economy / Factories

## 3. RED Phase: Tests First
```rust
#[test]
fn test_war_profiteering_buffs_factories() {
    let mut app = App::new();
    app.add_systems(Update, process_war_profiteering);

    // Set galaxy state to War
    app.world_mut().insert_resource(GalacticState { is_war: true });

    // Spawn a factory
    let factory = app.world_mut().spawn((
        Building { type_: BuildingType::Factory },
        ProductionEfficiency { multiplier: 1.0 },
    )).id();

    app.update();

    let efficiency = app.world().get::<ProductionEfficiency>(factory).unwrap();
    assert!(efficiency.multiplier > 1.0); // Buffed!
}

#[test]
fn test_peace_causes_recession() {
    let mut app = App::new();
    app.add_systems(Update, process_war_profiteering);

    // Set galaxy state to Peace
    app.world_mut().insert_resource(GalacticState { is_war: false });

    // Spawn a factory
    let factory = app.world_mut().spawn((
        Building { type_: BuildingType::Factory },
        ProductionEfficiency { multiplier: 1.0 },
    )).id();

    app.update();

    let efficiency = app.world().get::<ProductionEfficiency>(factory).unwrap();
    assert!(efficiency.multiplier < 1.0); // Debuffed!
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
fn process_war_profiteering(
    state: Res<GalacticState>,
    mut query: Query<&mut ProductionEfficiency, With<Building>>,
) {
    for mut eff in query.iter_mut() {
        if state.is_war {
            eff.multiplier = 1.5; // 50% bonus
        } else {
            eff.multiplier = 0.5; // 50% penalty
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Create specific "False Flag" missions that cost influence to trigger a war state.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >=85% for new code

## 7. Technical Guidance
- Ensure events are processed correctly in the update schedule.

## 8. Questions
*Builder: add questions here if spec is unclear.*

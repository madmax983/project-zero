# 591: The Fallow Cycle

## 1. Overview
**Layer:** 1
**Fantasy:** A planet that demands respect and periodic rest, punishing those who relentlessly strip-mine its surface.
**Mechanic:** The entire planetary crust has a hidden "Stress" metric. Intensive, constant mining and high-yield farming increase this stress. If the global stress gets too high, the planet enters a "Fallow Cycle": tectonic activity skyrockets, soil fertility drops to zero, and hostile native fauna go into a frenzy to "cleanse" the surface.
**Emergence:** You aggressively strip-mine the continent to build a massive fleet, ignoring the minor tremors. The planet reaches critical stress and enters a Fallow Cycle right as an enemy fleet arrives. Your automated turrets sink into the collapsing ground, and your starving, terrified militia are attacked by both the enemy and massive, enraged tunneling beasts.
**Tension:** Relentless, rapid industrial expansion vs. managing the biological and geological tolerance of the living world you inhabit.

## 2. Dependencies
- 012-farming
- 014-mining
- 150-planetary-hazards

## 3. RED Phase: Tests First
```rust
#[test]
fn test_mining_increases_crust_stress() {
    let mut app = setup_test_app();
    app.world.insert_resource(PlanetaryCrust { stress: 0.0 });

    app.world.send_event(MiningActivityEvent { intensity: 10.0 });
    app.update();

    let crust = app.world.resource::<PlanetaryCrust>();
    assert!(crust.stress > 0.0);
}

#[test]
fn test_high_stress_triggers_fallow_cycle() {
    let mut app = setup_test_app();
    app.world.insert_resource(PlanetaryCrust { stress: 100.0 });

    app.update(); // State transition to FallowCycle

    assert!(app.world.contains_resource::<FallowCycleActive>());
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// src/layer1/environment.rs
pub fn update_crust_stress_system(
    mut events: EventReader<MiningActivityEvent>,
    mut crust: ResMut<PlanetaryCrust>,
    mut commands: Commands,
) {
    for event in events.iter() {
        crust.stress += event.intensity * 0.1;
    }

    if crust.stress >= 100.0 {
        commands.insert_resource(FallowCycleActive);
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Add visual indicators for rising stress (e.g., minor tremors, localized fertile soil degradation).
- Implement the actual penalties of the Fallow Cycle (zero fertility, hostile fauna spawns).

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Mining/Farming activity increases global crust stress.
- [ ] Reaching maximum stress triggers a Fallow Cycle state.

## 7. Technical Guidance
- Ensure the `FallowCycleActive` state correctly modifiers fertility systems globally.

## 8. Questions
*Builder: add questions here if spec is unclear.*

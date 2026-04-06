# Quantum Bureaucracy

## 1. Overview
Late-game "Quantum Admin Computers" process Layer 3 edicts with negative time latency—granting immediate effects before the actual cost or time to pass the edict is paid. This unmatched bureaucratic power incurs a terrifying local cost: "Causality Debt." This debt physically manifests as anomalous tiles on the Layer 1 grid. Any Pops navigating through a Causality Debt tile suffer profound reality glitches, such as looping tasks, spontaneous creation or destruction of carried items, and severe psychological stress.

## 2. Dependencies
- Needs `Grid` and `Pop` movement systems in `src/layer1/`.
- Needs `Morale` or `Stress` mechanics in `src/layer1/needs.rs`.
- Needs `ColonyEdicts` or equivalent Layer 3 administrative system.

## 3. RED Phase: Tests First

```rust
#[test]
fn test_quantum_edict_applies_instant_effect_and_creates_debt() {
    let mut app = App::new();
    // Setup Layer 3 Edicts and Layer 1 Grid

    // Act: Invoke an edict via Quantum Admin Computer
    invoke_quantum_edict(&mut app, EdictType::EmergencyDefense);
    app.update();

    // Assert: The edict effect is instantly active
    assert!(app.world.resource::<ActiveEdicts>().contains(EdictType::EmergencyDefense));

    // Assert: A CausalityDebt component has spawned on at least one tile in Layer 1
    let mut debt_query = app.world.query::<&CausalityDebt>();
    assert!(debt_query.iter(&app.world).count() > 0);
}

#[test]
fn test_pop_walking_on_causality_debt_experiences_glitch() {
    let mut app = App::new();
    // Setup Grid with Causality Debt at (5, 5)
    let debt_entity = app.world.spawn((Position { x: 5, y: 5 }, CausalityDebt)).id();

    // Spawn a Pop at (5, 5)
    let pop = app.world.spawn((Position { x: 5, y: 5 }, Pop, Task::Hauling(Item::Food))).id();

    // Act: Run causality interaction system
    app.add_systems(Update, process_causality_debt_system);
    app.update();

    // Assert: The Pop's task might be looped/reset or their item deleted/duplicated
    // This checks for the glitch consequence
    let pop_state = app.world.get::<PopGlitchState>(pop);
    assert!(pop_state.is_some());
    assert!(app.world.get::<Stress>(pop).unwrap().value > 0.0);
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
#[derive(Component)]
pub struct CausalityDebt;

#[derive(Component)]
pub struct PopGlitchState {
    pub loops_remaining: u32,
}

pub fn invoke_quantum_edict(app: &mut App, edict: EdictType) {
    // 1. Immediately apply the edict's benefits.
    let mut active_edicts = app.world.resource_mut::<ActiveEdicts>();
    active_edicts.add(edict);

    // 2. Spawn causality debt in the environment.
    // In a real implementation, pick a random valid grid tile.
    app.world.spawn((Position { x: 0, y: 0 }, CausalityDebt));
}

pub fn process_causality_debt_system(
    mut commands: Commands,
    debt_query: Query<&Position, With<CausalityDebt>>,
    mut pop_query: Query<(Entity, &Position, &mut Stress), With<Pop>>,
) {
    for debt_pos in debt_query.iter() {
        for (entity, pop_pos, mut stress) in pop_query.iter_mut() {
            if debt_pos == pop_pos {
                // Apply glitch state and stress penalty
                commands.entity(entity).insert(PopGlitchState { loops_remaining: 3 });
                stress.value += 20.0;
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Random Distribution**: Improve the logic that places `CausalityDebt` to ensure it spawns in valid, reachable areas (like hospital floors or major corridors) to guarantee tension.
- **Glitch Variety**: The `PopGlitchState` should have different specific effects (e.g., dropping items, wandering backwards, temporary paralysis) to sell the "reality breakdown" narrative.
- **Visuals/UI**: Ensure `CausalityDebt` is communicated to the player via terminal visual effects (e.g., flickering characters) so they know *why* their Pops are acting strange.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Quantum Edicts instantly grant benefits but spawn Causality Debt
- [ ] Pops walking through Causality Debt suffer glitches and stress

## 7. Technical Guidance
- Integrate with the existing `MovementSystem` or `ActionSystem` to evaluate when a Pop enters a debt tile.
- Consider making Causality Debt temporary but very long-lasting, or require a specific "Reality Anchoring" task to clear it.
- Ensure the negative latency Edict system correctly hooks into Layer 3 costs (even if applied later) to prevent infinite free edicts.

## 8. Questions
*Builder: add questions here if spec is unclear.*

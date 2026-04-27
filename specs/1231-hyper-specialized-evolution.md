# 1231: Hyper-Specialized Evolution

## 1. Overview
**Layer:** 1

**Fantasy:** The body adapts to the task. "He was born to be a hauler."

**Mechanic:** Pops who work the same job for years (or generations) physically mutate to become better at it but worse at everything else. Miners develop night vision and hunchbacks; Diplomats lose muscle mass but gain pheromone control.

**Emergence:** Your colony becomes a caste system of "Mole People" (Miners) and "Tower Elites" (Admins) who can no longer physically interact without discomfort.

**Tension:** Extreme efficiency (Specialization) vs. Flexibility (Generalist workforce).

## 2. Dependencies
- Job assignment system
- Pop Traits system
- Time progression (aging/experience)

## 3. RED Phase: Tests First
```rust
#[test]
fn test_long_term_job_causes_mutation() {
    let mut app = App::new();
    app.add_systems(Update, process_job_experience);

    // Spawn a pop
    let pop = app.world_mut().spawn((
        Pop,
        Job { title: "Miner".to_string(), duration_ticks: 9999 },
        Traits::default(),
    )).id();

    // Run one tick to cross the threshold (10000)
    app.world_mut().entity_mut(pop).get_mut::<Job>().unwrap().duration_ticks += 1;
    app.update();

    let traits = app.world().get::<Traits>(pop).unwrap();

    // Assert mutation occurred
    assert!(traits.has_trait("Mole Person"));
    assert!(traits.has_trait("Light Sensitive"));
}

#[test]
fn test_mutation_gives_job_bonus_and_general_penalty() {
    let mut app = App::new();
    app.add_systems(Update, evaluate_action_efficiency);

    // Spawn mutated pop
    let mut traits = Traits::default();
    traits.add_trait("Mole Person");
    let pop = app.world_mut().spawn((
        Pop,
        traits,
        ActionEfficiency { base: 1.0, current: 1.0, action_type: ActionType::Mine },
    )).id();

    app.update();

    let efficiency = app.world().get::<ActionEfficiency>(pop).unwrap();
    // Huge mining boost
    assert!(efficiency.current > 1.5);

    // Test a different action
    app.world_mut().entity_mut(pop).insert(ActionEfficiency { base: 1.0, current: 1.0, action_type: ActionType::Haul });
    app.update();

    let efficiency2 = app.world().get::<ActionEfficiency>(pop).unwrap();
    // General penalty for other actions
    assert!(efficiency2.current < 0.8);
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
fn process_job_experience(
    mut query: Query<(&mut Job, &mut Traits)>,
) {
    for (mut job, mut traits) in query.iter_mut() {
        if job.title == "Miner" && job.duration_ticks >= 10000 {
            if !traits.has_trait("Mole Person") {
                traits.add_trait("Mole Person");
                traits.add_trait("Light Sensitive");
            }
        }
    }
}

fn evaluate_action_efficiency(
    mut query: Query<(&Traits, &mut ActionEfficiency)>,
) {
    for (traits, mut efficiency) in query.iter_mut() {
        if traits.has_trait("Mole Person") {
            if efficiency.action_type == ActionType::Mine {
                efficiency.current = efficiency.base * 2.0;
            } else {
                efficiency.current = efficiency.base * 0.5;
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Create a data-driven mapping for Job -> Mutation trait thresholds rather than hardcoding "Miner" and "Mole Person".
- Implement progressive stages of mutation rather than a single sudden leap.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >=85% for new code

## 7. Technical Guidance
- Hook into the existing `Traits` component if available, or create a `Mutations` component.
- Ensure the mutation event triggers a Chronicle entry or notification.

## 8. Questions
*Builder: add questions here if spec is unclear.*

# 1165: The Nostalgia Cult

# Overview
What: Introduce a cultural movement known as "The Nostalgia Cult". Pops who experience high stress or rapid technological shifts form this cult, romantically idealizing the founding era of the colony. They refuse to work in high-tech buildings, demand primitive housing, and actively sabotage advanced infrastructure in favor of manual labor.
Why: This introduces a social dynamic where extreme technological advancement creates a counter-culture, forcing players to balance progress with social harmony.

# Dependencies
- Needs `034-pop-health.md` for Stress/Morale tracking.
- Needs `084-pop-traits.md` for Pop psychology and trait assignment.
- Needs `016-utility-ai-system.md` for Utility AI action evaluation and job assignment filtering.

# RED Phase: Tests First

```rust
#[test]
fn test_nostalgia_cult_formation_from_high_stress() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Setup pop with extreme stress
    let pop_id = app.world.spawn((
        Pop,
        Psychology { stress: 90.0, ..Default::default() },
    )).id();

    // Run system simulating time/cult formation
    app.world.resource_mut::<SimulationTime>().tick();
    app.update();

    // Assert that the pop has joined the Nostalgia Cult
    assert!(app.world.get::<NostalgiaCult>(pop_id).is_some(), "Pop with extreme stress should join the Nostalgia Cult");
}

#[test]
fn test_nostalgia_cult_job_refusal() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Setup pop in the Nostalgia Cult
    let pop_id = app.world.spawn((
        Pop,
        NostalgiaCult,
    )).id();

    // Setup a high-tech job
    let high_tech_job = Job { tech_level: TechLevel::Advanced, ..Default::default() };

    // Evaluate action score for the job
    let score = evaluate_job_score(&app.world, pop_id, &high_tech_job);

    // Assert that the cult member refuses the high tech job (score 0.0)
    assert_eq!(score, 0.0, "Nostalgia Cult pops should refuse high-tech jobs");
}

#[test]
fn test_nostalgia_cult_sabotage_action() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Setup pop in the Nostalgia Cult
    let pop_id = app.world.spawn((
        Pop,
        NostalgiaCult,
    )).id();

    // Setup an advanced building target
    let building_id = app.world.spawn((
        Building,
        TechLevel::Advanced,
        Integrity { current: 100.0, max: 100.0 },
    )).id();

    // Force pop to execute sabotage action
    execute_sabotage_action(&mut app.world, pop_id, building_id);

    // Assert that the building took damage
    let integrity = app.world.get::<Integrity>(building_id).unwrap();
    assert!(integrity.current < integrity.max, "Nostalgia Cult pop should damage advanced buildings during sabotage");
}
```

# GREEN Phase: Minimal Implementation

```rust
pub fn nostalgia_cult_formation_system(
    mut commands: Commands,
    query: Query<(Entity, &Psychology), (With<Pop>, Without<NostalgiaCult>)>,
) {
    for (entity, psychology) in query.iter() {
        if psychology.stress >= 85.0 {
            commands.entity(entity).insert(NostalgiaCult);
        }
    }
}

pub fn evaluate_job_score(world: &World, pop_id: Entity, job: &Job) -> f32 {
    if world.get::<NostalgiaCult>(pop_id).is_some() && job.tech_level == TechLevel::Advanced {
        return 0.0;
    }

    // Default score logic...
    1.0
}

pub fn execute_sabotage_action(world: &mut World, pop_id: Entity, target_id: Entity) {
    if world.get::<NostalgiaCult>(pop_id).is_some() {
        if let Some(mut integrity) = world.get_mut::<Integrity>(target_id) {
            integrity.current -= 10.0;
        }
    }
}
```

# REFACTOR Phase: Quality & Design

- **Cult Spreading**: The `NostalgiaCult` trait should spread socially, similar to `055-rumor-web.md`. Cult members interacting with other stressed pops should have a high chance of converting them.
- **Sabotage Utility**: Integrate the sabotage action fully into the `Utility AI` system, scoring it based on the presence of advanced buildings and the pop's current cult fanaticism.
- **UI Enhancements**: Add visual indicators or distinct clothing colors to clearly identify Nostalgia Cult pops in the UI.

# Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >= 85% for new code.
- [ ] Specific feature behavior verified: Pops under extreme stress can join the Nostalgia Cult, refuse advanced jobs, and perform sabotage.

# Technical Guidance
- The `NostalgiaCult` component can be a marker component, or eventually carry data like `fanaticism_level`.
- Integration into the Utility AI requires adding a `SabotageAction` variant to the `ActionType` enum.
- Consider adding a `TechLevel` enum or property to all `Building` components if not already present.

# Questions
*Builder: add questions here if spec is unclear.*

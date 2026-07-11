# The Echo Plague

## 1. Overview
The Echo Plague is a psychological contagion that affects colonies that have suffered high casualties and stress. Pops afflicted with this memetic infection abandon their assigned jobs and instead compulsively mimic the tasks that recently deceased Pops were performing immediately prior to their deaths. This creates a dangerous loop of compulsive mimicry, often drawing untrained or unequipped Pops into hazardous situations.

## 2. Dependencies
- Job System (`specs/009-job-system.md`)
- Utility AI System (`specs/016-utility-ai-system.md`)
- Pop Entity (`specs/004-pop-entity.md`)

## 3. RED Phase: Tests First
```rust
#[test]
fn test_echo_plague_spawn_conditions() {
    let mut app = App::new();
    app.add_plugins((JobPlugin, EchoPlaguePlugin));

    // Arrange: Setup colony with high casualties and stress
    let colony = app.world_mut().spawn((
        Colony,
        RecentCasualties(10), // Threshold exceeded
        ColonyStress(80.0),   // Threshold exceeded
    )).id();

    // Act: Run disease spawner
    app.update();

    // Assert: Echo Plague event/status is triggered
    let has_plague = app.world().entity(colony).contains::<EchoPlagueActive>();
    assert!(has_plague, "Echo Plague should trigger under high casualty and stress conditions");
}

#[test]
fn test_pop_compulsive_mimicry() {
    let mut app = App::new();
    app.add_plugins((JobPlugin, UtilityAiPlugin, EchoPlaguePlugin));

    // Arrange: Record a dead pop's last job and infect a living pop
    let last_job = JobType::RepairReactor;
    let dead_pop_record = app.world_mut().spawn(DeadPopRecord { last_job }).id();

    let living_pop = app.world_mut().spawn((
        Pop,
        Job::new(JobType::Farm),
        EchoPlagueInfected { target_record: dead_pop_record },
    )).id();

    // Act: Utility AI tick
    app.update();

    // Assert: Pop's active behavior is overridden to the dead pop's job
    let ai_state = app.world().get::<CurrentAction>(living_pop).unwrap();
    assert_eq!(ai_state.0, ActionType::PerformJob(JobType::RepairReactor));
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
#[derive(Component)]
pub struct RecentCasualties(pub u32);

#[derive(Component)]
pub struct ColonyStress(pub f32);

#[derive(Component)]
pub struct EchoPlagueActive;

#[derive(Component)]
pub struct DeadPopRecord {
    pub last_job: JobType,
}

#[derive(Component)]
pub struct EchoPlagueInfected {
    pub target_record: Entity,
}

pub struct EchoPlaguePlugin;

impl Plugin for EchoPlaguePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            check_echo_plague_outbreak,
            enforce_compulsive_mimicry,
        ));
    }
}

fn check_echo_plague_outbreak(
    mut commands: Commands,
    query: Query<(Entity, &RecentCasualties, &ColonyStress), Without<EchoPlagueActive>>,
) {
    for (entity, casualties, stress) in query.iter() {
        if casualties.0 >= 10 && stress.0 >= 75.0 {
            commands.entity(entity).insert(EchoPlagueActive);
        }
    }
}

fn enforce_compulsive_mimicry(
    mut query: Query<(&EchoPlagueInfected, &mut CurrentAction)>,
    records: Query<&DeadPopRecord>,
) {
    for (infected, mut current_action) in query.iter_mut() {
        if let Ok(record) = records.get(infected.target_record) {
            current_action.0 = ActionType::PerformJob(record.last_job.clone());
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Code Smells**: The `enforce_compulsive_mimicry` system hard-overrides `CurrentAction`. This might fight with the Utility AI system if not properly integrated into the AI's scoring mechanism (e.g., scoring the mimicry action extremely high instead of hardcoding).
- **Performance**: We need a way to clean up `DeadPopRecord` entities if they are no longer being mimicked to prevent memory bloat over long play sessions.
- **API Improvements**: Introduce an `InfectPopEvent` to handle the spread of the plague naturally among populations rather than relying on direct component insertion in larger scopes.
- **Integration**: The Job System needs to allow Pops to attempt jobs they are unequipped or unqualified for (e.g., a Farmer trying to repair a Reactor without a hazmat suit) to fulfill the danger aspect of this mechanic.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Pops correctly abandon assigned tasks to mimic the dead when infected.

## 7. Technical Guidance
- Ensure that the Utility AI system respects the override. It's recommended to implement this as a high-priority Utility AI action evaluator (e.g., `MimicDeadEvaluator` that returns a score of 1.0 when infected) rather than a hard overwrite to maintain the integrity of the `UtilityAiPlugin`.
- When a Pop dies, their `DeadPopRecord` needs to be spawned by the death/casualty handling system. You will need to hook into those existing events.

## 8. Questions
*Builder: add questions here if spec is unclear.*

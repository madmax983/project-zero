# 1230: Title Inflation

## 1. Overview
**Layer:** 1

**Fantasy:** Everyone wants to be a Manager.

**Mechanic:** Middle-class Pops demand "Promotions". You can give them fancy titles ("Senior Executive Miner") that cost Admin but give no authority, just Mood.

**Emergence:** You have 50 "Vice Presidents of Hauling" and only 1 actual Hauler. The Vice Presidents refuse to carry rocks.

**Tension:** Ego stroking (Mood) vs. Organizational efficiency.

## 2. Dependencies
- Job / Assignment system
- Morale system
- Administration Resource system

## 3. RED Phase: Tests First
```rust
#[test]
fn test_title_promotion_boosts_morale_but_costs_admin() {
    let mut app = App::new();
    app.add_systems(Update, process_title_promotions);

    // Setup Admin resource
    app.world_mut().insert_resource(AdminPoints { points: 100.0 });
    app.world_mut().insert_resource(Events::<PromotionEvent>::default());

    // Spawn a worker
    let pop = app.world_mut().spawn((
        Pop,
        Job { title: "Miner".to_string(), is_executive: false },
        Morale { value: 50.0 },
    )).id();

    // Trigger promotion
    app.world_mut().send_event(PromotionEvent { target: pop, new_title: "Executive Miner".to_string() });

    app.update();

    let job = app.world().get::<Job>(pop).unwrap();
    let morale = app.world().get::<Morale>(pop).unwrap();
    let admin = app.world().resource::<AdminPoints>();

    // Assert promotion success
    assert_eq!(job.title, "Executive Miner");
    assert!(job.is_executive);
    // Morale went up
    assert!(morale.value > 50.0);
    // Admin went down
    assert!(admin.points < 100.0);
}

#[test]
fn test_executives_refuse_manual_labor() {
    let mut app = App::new();
    app.add_systems(Update, evaluate_hauling_job);

    // Spawn an executive pop
    let pop = app.world_mut().spawn((
        Pop,
        Job { title: "VP of Hauling".to_string(), is_executive: true },
    )).id();

    // Try to evaluate utility for hauling
    let mut cand_eval = CandidateEvaluator::new(0.0, false);
    // Pretend we have a valid hauling target
    cand_eval.evaluate_and_consider(
        Some((10.0, Entity::PLACEHOLDER)),
        ActionType::Haul,
        &WorldContext::default(),
        0.0
    );

    // However, if we write a wrapper or modifier for the scoring based on job...
    // Let's assert a system actively drops the ActionType if they are executive
    app.world_mut().entity_mut(pop).insert(PopAction { current: ActionType::Haul, ..Default::default() });

    app.update();

    let action = app.world().get::<PopAction>(pop).unwrap();
    // Executive immediately aborts manual labor
    assert_eq!(action.current, ActionType::Idle);
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
fn process_title_promotions(
    mut events: EventReader<PromotionEvent>,
    mut query: Query<(&mut Job, &mut Morale)>,
    mut admin: ResMut<AdminPoints>,
) {
    for ev in events.read() {
        if admin.points >= 10.0 {
            if let Ok((mut job, mut morale)) = query.get_mut(ev.target) {
                admin.points -= 10.0;
                job.title = ev.new_title.clone();
                job.is_executive = true;
                morale.value += 20.0; // Big ego boost
            }
        }
    }
}

fn evaluate_hauling_job(
    mut query: Query<(&Job, &mut PopAction)>,
) {
    for (job, mut action) in query.iter_mut() {
        if job.is_executive && action.current == ActionType::Haul {
            // Executives refuse to haul!
            action.current = ActionType::Idle;
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Hook the refusal logic into the `UtilityWeights` so they don't even consider the job in the first place, saving CPU cycles instead of cancelling it post-facto.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >=85% for new code

## 7. Technical Guidance
- Integrate into the Job and Utility AI systems.

## 8. Questions
*Builder: add questions here if spec is unclear.*

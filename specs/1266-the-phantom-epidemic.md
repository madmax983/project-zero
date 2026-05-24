# Overview
What: Introduce a new psychological condition called "The Phantom Epidemic" (Hypochondria) where pops believe they are infected with a deadly disease that does not exist on their world.
Why: This realizes the fantasy of a plague of the mind that paralyzes infrastructure without a single actual infection. It creates tension by forcing the player to waste resources validating the delusion or enforcing quotas at the cost of high stress.

# Dependencies
- Needs `034-pop-health.md` for `Health` and `Condition`.
- Needs `127-stress-breakdowns.md` for `StressTracker`.
- Needs `018-mining-resources.md` for `ColonyResources` (to waste medical supplies).

# RED Phase: Tests First
```rust
#[test]
fn test_phantom_epidemic_condition_added() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Setup pop with high stress and no current conditions
    let pop_id = app.world.spawn((
        Pop,
        Health::default(),
        StressTracker { accumulated_stress: 90.0 }
    )).id();

    // Add communication event/flag that triggers the epidemic check
    app.world.insert_resource(ExternalCommsEvent { has_disease_warning: true });

    // Run the system
    check_phantom_epidemic_system(&mut app.world);

    let health = app.world.get::<Health>(pop_id).unwrap();
    assert!(health.has_condition(Condition::PhantomEpidemic));
}

#[test]
fn test_phantom_epidemic_wastes_medical_supplies() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    let mut resources = ColonyResources::default();
    resources.add_medicine(50.0);
    app.world.insert_resource(resources);

    // Setup pop WITH Phantom Epidemic
    let mut health = Health::default();
    health.add_condition(Condition::PhantomEpidemic);
    let pop_id = app.world.spawn((Pop, health)).id();

    // Run the consumption system
    consume_medicine_for_phantom_epidemic_system(&mut app.world);

    let current_resources = app.world.get_resource::<ColonyResources>().unwrap();
    assert!(current_resources.medicine < 50.0, "Medicine should be consumed to treat the fake disease");
}
```

# GREEN Phase: Minimal Implementation
```rust
// In Health or similar module
#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum Condition {
    // Existing...
    RustLung,
    PhantomEpidemic,
}

#[derive(Resource, Default)]
pub struct ExternalCommsEvent {
    pub has_disease_warning: bool,
}

pub fn check_phantom_epidemic_system(world: &mut World) {
    let warning = world.get_resource::<ExternalCommsEvent>().map(|e| e.has_disease_warning).unwrap_or(false);
    if !warning { return; }

    let mut pops_to_infect = Vec::new();
    for (entity, stress, health) in world.query::<(Entity, &StressTracker, &Health)>().iter(world) {
        if stress.accumulated_stress > 80.0 && !health.has_condition(Condition::PhantomEpidemic) {
            pops_to_infect.push(entity);
        }
    }

    for entity in pops_to_infect {
        if let Some(mut health) = world.get_mut::<Health>(entity) {
            health.add_condition(Condition::PhantomEpidemic);
        }
    }
}

pub fn consume_medicine_for_phantom_epidemic_system(world: &mut World) {
    let mut medicine_to_consume = 0.0;
    for health in world.query::<&Health>().iter(world) {
        if health.has_condition(Condition::PhantomEpidemic) {
            medicine_to_consume += 1.0;
        }
    }

    if medicine_to_consume > 0.0 {
        if let Some(mut resources) = world.get_resource_mut::<ColonyResources>() {
            let cost = ColonyResources::default().with_medicine(medicine_to_consume);
            resources.try_deduct(&cost); // Ignores if not enough, but consumes what it can
        }
    }
}
```

# REFACTOR Phase: Quality & Design
- Integrate `check_phantom_epidemic_system` and `consume_medicine_for_phantom_epidemic_system` into the main simulation loop under a new module `src/layer1/psychology/delusions.rs`.
- Link the `ExternalCommsEvent` to the chronicle system so it triggers when another colony falls to Petrification Sickness.
- Adjust `ColonyResources` to support `medicine` if it doesn't already, or map to `rations`/`chemicals` if `medicine` is not a core resource.
- Ensure pops with `PhantomEpidemic` have a work speed penalty.

# Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Specific feature behavior verified

# Technical Guidance
- Verify if `medicine` exists in `ColonyResources`. If not, use `tools` or add `medicine` as a valid resource type based on `DESIGN.md`.
- Ensure the `Condition` enum is updated properly.
- The phantom epidemic should eventually fade if stress is reduced below a certain threshold.

# Questions
*Builder: add questions here if spec is unclear.*

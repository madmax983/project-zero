# 1173: Echoes of the First Star

## 1. Overview
At a random point in the late game, the star of the civilization's original founding system goes supernova. Even if that system is halfway across the galaxy and currently controlled by an enemy, every Pop in your empire receives an "Echo of the First Star" event. Pops gain a massive but temporary boost to unity and creative output, driven by existential reflection, but suffer a deep, lingering "Ancestral Grief" debuff that lowers combat effectiveness and increases the chance of Pops abandoning their jobs to become "Stargazers".

## 2. Dependencies
- Cross-layer event system (Layer 3/2 -> Layer 1).
- `Pop` mood, traits, and job assignment systems.
- Chronicle and Event broadcasting systems.

## 3. RED Phase: Tests First

```rust
#[test]
fn test_first_star_supernova_event_broadcast() {
    // Arrange
    let mut app = App::new();
    app.add_systems(Update, handle_supernova_event_system);
    app.add_event::<FirstStarSupernovaEvent>();

    // Spawn some pops
    let pop1 = app.world_mut().spawn(Pop).id();
    let pop2 = app.world_mut().spawn(Pop).id();

    // Act
    app.world_mut().send_event(FirstStarSupernovaEvent);
    app.update();

    // Assert: Pops should receive the AncestralGrief component
    assert!(app.world().get::<AncestralGrief>(pop1).is_some(), "Pop1 should have Ancestral Grief");
    assert!(app.world().get::<AncestralGrief>(pop2).is_some(), "Pop2 should have Ancestral Grief");
}

#[test]
fn test_ancestral_grief_effects() {
    // Arrange
    let mut app = App::new();
    app.add_systems(Update, apply_ancestral_grief_effects_system);

    // Spawn a pop with grief
    let pop = app.world_mut().spawn((
        Pop,
        AncestralGrief { duration: 10.0 },
        CombatEffectiveness(1.0),
        CreativeOutput(1.0)
    )).id();

    // Act
    app.update();

    // Assert: Combat goes down, Creativity goes up
    let combat = app.world().get::<CombatEffectiveness>(pop).unwrap().0;
    let creativity = app.world().get::<CreativeOutput>(pop).unwrap().0;

    assert!(combat < 1.0, "Combat effectiveness should be reduced by grief");
    assert!(creativity > 1.0, "Creative output should be boosted by existential reflection");
}

#[test]
fn test_stargazer_job_abandonment() {
    // Arrange
    let mut app = App::new();
    app.add_systems(Update, stargazer_conversion_system);

    // Spawn a pop with grief and a job
    let pop = app.world_mut().spawn((
        Pop,
        AncestralGrief { duration: 10.0 },
        Job::Miner,
        StargazerChance(1.0) // 100% chance for test
    )).id();

    // Act
    app.update();

    // Assert: Pop abandoned job to become Stargazer
    assert!(app.world().get::<Job>(pop).is_none(), "Pop should have abandoned their job");
    assert!(app.world().get::<Stargazer>(pop).is_some(), "Pop should be a Stargazer");
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Pop;

#[derive(Event)]
pub struct FirstStarSupernovaEvent;

#[derive(Component)]
pub struct AncestralGrief {
    pub duration: f32,
}

#[derive(Component)]
pub struct CombatEffectiveness(pub f32);

#[derive(Component)]
pub struct CreativeOutput(pub f32);

#[derive(Component)]
pub enum Job {
    Miner,
    Builder,
    // ...
}

#[derive(Component)]
pub struct Stargazer;

#[derive(Component)]
pub struct StargazerChance(pub f32);

pub fn handle_supernova_event_system(
    mut commands: Commands,
    mut events: EventReader<FirstStarSupernovaEvent>,
    pops: Query<Entity, With<Pop>>,
) {
    for _ in events.read() {
        for pop_entity in pops.iter() {
            commands.entity(pop_entity).insert(AncestralGrief { duration: 100.0 });
        }
    }
}

pub fn apply_ancestral_grief_effects_system(
    mut pops: Query<(&AncestralGrief, &mut CombatEffectiveness, &mut CreativeOutput)>,
) {
    for (_, mut combat, mut creativity) in pops.iter_mut() {
        combat.0 = 0.5; // 50% penalty
        creativity.0 = 2.0; // 100% boost
    }
}

pub fn stargazer_conversion_system(
    mut commands: Commands,
    pops: Query<(Entity, &AncestralGrief, Option<&StargazerChance>), With<Job>>,
) {
    for (entity, _, chance_cmp) in pops.iter() {
        let chance = chance_cmp.map(|c| c.0).unwrap_or(0.01);
        if chance >= 1.0 || rand::random::<f32>() < chance {
            commands.entity(entity)
                .remove::<Job>()
                .insert(Stargazer);
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Event Trigger**: The `FirstStarSupernovaEvent` needs to be hooked into the global timeline or Layer 3 star lifecycle simulation.
- **Duration Decay**: `AncestralGrief.duration` is currently static. It should be decremented via `Time` and removed when it reaches 0.
- **Modifiers**: Hardcoding `.0 = 0.5` is bad practice. This should use the central modifier system so it stacks correctly with other effects.

## 6. Acceptance Criteria (Testable!)
- [ ] All RED phase tests pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage $\ge$ 85% for new code.
- [ ] Supernova event successfully applies the grief component to all Pops, altering their stats and potentially causing job abandonment.

## 7. Technical Guidance
- Ensure `Stargazer` is recognized as a valid 'state' by the utility AI, perhaps making them wander to surface tiles at night to look up.
- This is a prime event for a major UI popup and Chronicle entry.

## 8. Questions
*Builder: add questions here if spec is unclear.*
*Architect:* Implement the simplest possible version for the MVP. Advanced behaviors and edge cases will be deferred to future specifications.

# The Sleepwalker's Rebellion

## 1. Overview
In highly oppressive or micromanaged colonies, Pops develop "Subconscious Dissent." While they remain obedient during their waking hours, their repressed anger manifests while they are asleep. Pops with high dissent may sleepwalk and perform minor acts of sabotage, vandalism, or unauthorized consumption, only to wake up with no memory of their actions.

## 2. Dependencies
- Base simulation framework (`App`, `World`)
- Sleep Need system (`RestTracker`, `Sleeping` state)
- Emotional state / Memory system (`SubconsciousDissent` component)
- Action/Utility AI system for executing sabotage during sleep without breaking the normal `Sleeping` state.

## 3. RED Phase: Tests First

```rust
#[test]
fn test_oppression_increases_subconscious_dissent() {
    // Arrange
    let mut app = App::new();
    app.insert_resource(ColonyOppressionLevel(5.0)); // High oppression
    app.add_systems(Update, accumulate_dissent_system);

    let pop = app.world_mut().spawn(SubconsciousDissent { amount: 0.0 }).id();

    // Act
    app.update();

    // Assert: Pop's dissent should increase due to high oppression
    let dissent = app.world().get::<SubconsciousDissent>(pop).unwrap();
    assert!(dissent.amount > 0.0);
}

#[test]
fn test_sleeping_pop_with_high_dissent_sleepwalks() {
    // Arrange
    let mut app = App::new();
    app.add_systems(Update, trigger_sleepwalking_system);

    let pop = app.world_mut().spawn((
        SubconsciousDissent { amount: 100.0 }, // Very high
        Sleeping, // Pop is asleep
        Position { x: 0.0, y: 0.0 },
    )).id();

    // Act
    app.update();

    // Assert: Pop should now have the Sleepwalking status, but still be Sleeping
    assert!(app.world().get::<Sleepwalking>(pop).is_some());
    assert!(app.world().get::<Sleeping>(pop).is_some());
}

#[test]
fn test_sleepwalking_sabotage_and_dissent_relief() {
    // Arrange
    let mut app = App::new();
    app.add_event::<SabotageEvent>();
    app.add_systems(Update, execute_sleepwalking_sabotage_system);

    let pop = app.world_mut().spawn((
        SubconsciousDissent { amount: 100.0 },
        Sleepwalking,
        Position { x: 0.0, y: 0.0 },
    )).id();

    // Act
    app.update();

    // Assert: Sabotage event emitted, dissent reduced, sleepwalking state removed
    let events = app.world().resource::<Events<SabotageEvent>>();
    let mut reader = events.get_reader();
    let ev = reader.read(events).next().expect("Expected SabotageEvent");
    assert_eq!(ev.perpetrator, pop);

    let dissent = app.world().get::<SubconsciousDissent>(pop).unwrap();
    assert!(dissent.amount < 100.0);
    assert!(app.world().get::<Sleepwalking>(pop).is_none());
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use rand::Rng;

#[derive(Resource)]
pub struct ColonyOppressionLevel(pub f32);

#[derive(Component)]
pub struct SubconsciousDissent {
    pub amount: f32,
}

#[derive(Component)]
pub struct Sleeping;

#[derive(Component)]
pub struct Sleepwalking;

#[derive(Component)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(Event)]
pub struct SabotageEvent {
    pub perpetrator: Entity,
    pub target_x: f32,
    pub target_y: f32,
}

pub fn accumulate_dissent_system(
    oppression: Res<ColonyOppressionLevel>,
    mut pops: Query<&mut SubconsciousDissent>,
) {
    let increase = oppression.0 * 0.1; // Simple linear scaling
    for mut dissent in pops.iter_mut() {
        dissent.amount += increase;
    }
}

pub fn trigger_sleepwalking_system(
    mut commands: Commands,
    query: Query<(Entity, &SubconsciousDissent), (With<Sleeping>, Without<Sleepwalking>)>,
) {
    let mut rng = rand::thread_rng();
    for (entity, dissent) in query.iter() {
        if dissent.amount > 50.0 {
            // Chance increases as dissent goes higher
            let chance = (dissent.amount - 50.0) / 100.0;
            if rng.gen::<f32>() < chance {
                commands.entity(entity).insert(Sleepwalking);
            }
        }
    }
}

pub fn execute_sleepwalking_sabotage_system(
    mut commands: Commands,
    mut pops: Query<(Entity, &Position, &mut SubconsciousDissent), With<Sleepwalking>>,
    mut sabotage_events: EventWriter<SabotageEvent>,
) {
    for (entity, pos, mut dissent) in pops.iter_mut() {
        // Sabotage nearby
        sabotage_events.send(SabotageEvent {
            perpetrator: entity,
            target_x: pos.x + 1.0, // Arbitrary nearby spot for now
            target_y: pos.y,
        });

        // Relief from the act
        dissent.amount -= 50.0;
        if dissent.amount < 0.0 {
            dissent.amount = 0.0;
        }

        // Return to normal sleep
        commands.entity(entity).remove::<Sleepwalking>();
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Pathing/Movement**: `execute_sleepwalking_sabotage_system` currently executes sabotage immediately and adjacently. A full implementation should push an action into the Pop's `ActionQueue` to physically walk them to a high-value target (like an authority symbol) while in the `Sleepwalking` state.
- **Waking Up**: The simulation needs logic to interrupt `Sleepwalking` if the Pop is forcibly woken (e.g., attacked, loud noise), which should perhaps induce heavy `Stress` or `Confusion` instead of relieving `SubconsciousDissent`.
- **Oppression Modifiers**: `ColonyOppressionLevel` should be calculated dynamically from active laws, enforcer presence, and extreme work shifts, rather than being a static hardcoded resource.

## 6. Acceptance Criteria
- [ ] All RED phase tests pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage $\ge$ 85% for new code.
- [ ] Sleepwalking only triggers on sleeping pops, and successfully resets the sleepwalking state after sabotage.

## 7. Technical Guidance
- Make sure the `UtilityAI` doesn't overwrite the `Sleepwalking` state with a standard `Work` action just because they are up and moving. `Sleepwalking` must temporarily override normal utility weighting.
- Avoid cascading wakeups if a sleepwalker sabotages a loud machine near other sleeping pops!

## 8. Questions
*Builder: How should we treat a Pop who is arrested while sleepwalking? Do they get an "innocent confusion" modifier, or are they treated as a standard rebel?*

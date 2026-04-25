# 1182: The Hyper-Empathy Virus

## 1. Overview
A contagion spreads through the colony that links the nervous systems of infected Pops. If one infected Pop stubs their toe, every infected Pop feels the pain. If one is depressed, they all are. However, if one is ecstatic, everyone shares the joy.

## 2. Dependencies
- Layer 1 `Pop` mood, health, and needs system.
- An event system to detect large mood or pain swings in individual Pops.
- A disease/contagion transmission system.

## 3. RED Phase: Tests First

```rust
#[test]
fn test_sympathetic_mood_sharing() {
    // Arrange
    let mut app = App::new();
    app.add_systems(Update, process_sympathetic_mood_system);
    app.add_event::<MoodChangeEvent>();

    // Spawn 3 infected pops with base mood
    let pop1 = app.world_mut().spawn((Pop, HyperEmpathyInfected, Mood(50.0))).id();
    let pop2 = app.world_mut().spawn((Pop, HyperEmpathyInfected, Mood(50.0))).id();
    let pop3 = app.world_mut().spawn((Pop, HyperEmpathyInfected, Mood(50.0))).id();

    // Fire an event indicating pop1 gained 20 mood (e.g. from an amazing meal)
    app.world_mut().send_event(MoodChangeEvent {
        pop: pop1,
        delta: 20.0,
    });

    // Act
    app.update();

    // Assert: ALL infected pops should gain the mood
    // Pop1 gained it normally (simulated by the system for the test, or just checking others)
    let m2 = app.world().get::<Mood>(pop2).unwrap().0;
    let m3 = app.world().get::<Mood>(pop3).unwrap().0;
    assert_eq!(m2, 70.0, "Pop2 should sympathetically gain mood");
    assert_eq!(m3, 70.0, "Pop3 should sympathetically gain mood");
}

#[test]
fn test_sympathetic_pain_shock() {
    // Arrange
    let mut app = App::new();
    app.add_systems(Update, process_sympathetic_pain_system);
    app.add_event::<DamageEvent>();

    // Spawn 2 infected pops
    let pop1 = app.world_mut().spawn((Pop, HyperEmpathyInfected)).id();
    let pop2 = app.world_mut().spawn((Pop, HyperEmpathyInfected)).id();

    // Pop1 takes massive damage (e.g., rockfall)
    app.world_mut().send_event(DamageEvent {
        pop: pop1,
        amount: 50.0,
    });

    // Act
    app.update();

    // Assert: Pop2 should gain a SympatheticShock component
    assert!(app.world().get::<SympatheticShock>(pop2).is_some(), "Pop2 should suffer sympathetic shock from Pop1's damage");
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Pop;

#[derive(Component)]
pub struct HyperEmpathyInfected;

#[derive(Component)]
pub struct Mood(pub f32);

#[derive(Component)]
pub struct SympatheticShock;

#[derive(Event)]
pub struct MoodChangeEvent {
    pub pop: Entity,
    pub delta: f32,
}

#[derive(Event)]
pub struct DamageEvent {
    pub pop: Entity,
    pub amount: f32,
}

pub fn process_sympathetic_mood_system(
    mut events: EventReader<MoodChangeEvent>,
    mut pops: Query<(Entity, &mut Mood), With<HyperEmpathyInfected>>,
) {
    for event in events.read() {
        // Is the source infected?
        let mut source_is_infected = false;
        for (entity, _) in pops.iter() {
            if entity == event.pop {
                source_is_infected = true;
                break;
            }
        }

        if source_is_infected {
            // Apply delta to all OTHER infected pops
            for (entity, mut mood) in pops.iter_mut() {
                if entity != event.pop {
                    mood.0 = (mood.0 + event.delta).clamp(0.0, 100.0);
                } else {
                    // For the test simplicity, apply to self as well if we assume the event itself
                    // is just a notification and hasn't been applied yet.
                    mood.0 = (mood.0 + event.delta).clamp(0.0, 100.0);
                }
            }
        }
    }
}

pub fn process_sympathetic_pain_system(
    mut commands: Commands,
    mut events: EventReader<DamageEvent>,
    pops: Query<Entity, With<HyperEmpathyInfected>>,
) {
    for event in events.read() {
        // Is the source infected?
        if pops.iter().any(|e| e == event.pop) {
            // High damage threshold for shock
            if event.amount >= 30.0 {
                // Apply shock to all OTHER infected pops
                for entity in pops.iter() {
                    if entity != event.pop {
                        commands.entity(entity).insert(SympatheticShock);
                    }
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Event Loops**: Be extremely careful that sympathetic mood changes don't fire *new* `MoodChangeEvent`s, creating an infinite recursive loop of joy or depression. Sympathetic changes must be applied silently.
- **Distance Modifier**: The shared feeling might be stronger the closer the Pops are geographically, scaling down with distance.
- **Shock Effects**: `SympatheticShock` should force the Pop to drop their current task, fall down, and slowly recover over a few seconds.

## 6. Acceptance Criteria (Testable!)
- [ ] All RED phase tests pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage $\ge$ 85% for new code.
- [ ] Mood swings and high-damage events correctly propagate to all other infected Pops without causing infinite loops.

## 7. Technical Guidance
- Integrate this directly with existing Layer 1 needs/damage evaluation systems.
- You may need to create a dedicated UI overlay to let the player easily see who is connected to the "empathy web".

## 8. Questions
*Builder: add questions here if spec is unclear.*

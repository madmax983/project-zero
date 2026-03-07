# Cognitive Hazards

## 1. Overview
**Layer:** 1
**Fantasy:** The planet whispers to you, and the darkness stares back.
**Mechanic:** Exposure to "Horror" (corpses, aliens, deep darkness) increases "Insanity". High Insanity causes hallucinations: fake fires, fake enemies, or hearing non-existent orders.
**Emergence:** The player receives a notification "Raid Detected!", mobilizes the militia, and they shoot at empty air, wasting ammo and causing panic.
**Tension:** Trust your sensors (objective) or your pops (subjective)?

## 2. Dependencies
- `005-pop-needs` (Needs component structure)
- `034-pop-health` (Status effects and traits)
- `046-notifications-system` (Triggering false notifications)
- `060-acoustic-simulation` or `053-lighting-system` (Darkness as a horror source)

## 3. RED Phase: Tests First

```rust
// specs/416-cognitive-hazards.rs

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::pop::{Pop, Needs, Traits};
    use crate::layer1::hazards::{HorrorSource, Insanity, exposure_system, hallucination_system};

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, (exposure_system, hallucination_system));
        app
    }

    #[test]
    fn test_exposure_increases_insanity() {
        // Arrange
        let mut app = setup_app();

        let pop_id = app.world_mut().spawn((
            Pop,
            Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            Insanity { level: 0.0 },
        )).id();

        // Spawn a horror source nearby
        app.world_mut().spawn((
            HorrorSource { intensity: 10.0, radius: 5.0 },
            Transform::from_translation(Vec3::new(2.0, 0.0, 0.0)),
        ));

        // Act
        app.update(); // Tick the exposure system

        // Assert: Insanity should increase
        let insanity = app.world().get::<Insanity>(pop_id).unwrap();
        assert!(insanity.level > 0.0);
    }

    #[test]
    fn test_high_insanity_triggers_hallucinations() {
        // Arrange
        let mut app = setup_app();

        let pop_id = app.world_mut().spawn((
            Pop,
            Insanity { level: 95.0 }, // High enough to trigger
        )).id();

        // Act
        app.update(); // Tick hallucination system

        // Assert: A hallucination event or notification was generated.
        let events = app.world().resource::<Events<crate::layer1::notifications::NotificationEvent>>();
        assert!(events.get_reader().len(&events) > 0);
    }

    #[test]
    fn test_insanity_decays_passively() {
        // Arrange
        let mut app = setup_app();

        let pop_id = app.world_mut().spawn((
            Pop,
            Transform::from_translation(Vec3::new(100.0, 0.0, 0.0)), // Far away from horror
            Insanity { level: 50.0 },
        )).id();

        // Act
        app.update(); // Tick systems

        // Assert: Insanity should decay slightly when not near horror
        let insanity = app.world().get::<Insanity>(pop_id).unwrap();
        assert!(insanity.level < 50.0);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/layer1/hazards.rs
use bevy::prelude::*;
use crate::layer1::pop::{Pop, Needs};
use crate::layer1::notifications::NotificationEvent;

#[derive(Component)]
pub struct Insanity {
    pub level: f32, // 0.0 to 100.0
}

#[derive(Component)]
pub struct HorrorSource {
    pub intensity: f32,
    pub radius: f32,
}

pub fn exposure_system(
    mut pop_query: Query<(&Transform, &mut Insanity)>,
    horror_query: Query<(&Transform, &HorrorSource)>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();

    for (pop_tf, mut insanity) in pop_query.iter_mut() {
        let mut total_exposure = 0.0;

        for (horror_tf, horror) in horror_query.iter() {
            let dist = pop_tf.translation.distance(horror_tf.translation);
            if dist < horror.radius {
                // Closer = more intense
                let exposure_rate = (1.0 - (dist / horror.radius)) * horror.intensity;
                total_exposure += exposure_rate;
            }
        }

        if total_exposure > 0.0 {
            insanity.level += total_exposure * dt;
            insanity.level = insanity.level.min(100.0);
        } else {
            // Passive decay
            insanity.level -= 1.0 * dt;
            insanity.level = insanity.level.max(0.0);
        }
    }
}

pub fn hallucination_system(
    pop_query: Query<&Insanity, With<Pop>>,
    mut events: EventWriter<NotificationEvent>,
    mut local_timer: Local<f32>,
    time: Res<Time>,
) {
    *local_timer += time.delta_secs();

    // Only check periodically to prevent spam
    if *local_timer > 5.0 {
        *local_timer = 0.0;

        for insanity in pop_query.iter() {
            if insanity.level > 80.0 {
                // High chance of hallucination
                if rand::random::<f32>() < 0.1 { // 10% chance every 5 seconds per insane pop
                    events.send(NotificationEvent {
                        message: "Raid Detected! (False Alarm)".to_string(),
                        severity: crate::layer1::notifications::Severity::Critical,
                        location: None,
                    });
                    // Only trigger one hallucination per tick to avoid overwhelming the player
                    break;
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Spatial Grid:** Using `O(N*M)` distance checks between Pops and HorrorSources is inefficient. Use the existing spatial partition grid or `TerrainGrid` to check local tiles for horror sources (e.g., corpses, darkness).
- **Hallucination Types:** "False Raid" is just one type. Consider a generic `HallucinationEvent` that the UI system picks up, or spawning invisible "Phantom Enemy" entities that only the insane pop's utility AI can target, causing them to shoot at nothing.
- **Traits Interaction:** Add traits like "Iron Will" (slower insanity gain) or "Fragile Mind" (faster gain).

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Pops near a `HorrorSource` accumulate `Insanity` over time.
- [ ] Pops far away from `HorrorSource` slowly lose `Insanity`.
- [ ] Pops with high `Insanity` periodically trigger false notification events.

## 7. Technical Guidance
- Integrate with `src/layer1/notifications.rs` or the equivalent system handling UI alerts.
- Ensure `HorrorSource` components are automatically added to relevant entities (e.g., corpses, specific alien types) upon their creation.
- Be mindful of performance in `exposure_system` if there are many horror sources.

## 8. Questions
*Builder: add questions here if spec is unclear.*

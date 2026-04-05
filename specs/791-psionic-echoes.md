# Specification: Psionic Echoes (Layer 1)

## 1. Overview
Areas where intense emotional events occurred (e.g., mass starvation, brutal raids, euphoric festivals) become imprinted with "Psionic Echoes." These are invisible spatial fields that passively transmit those emotions to Pops who enter them at a later time. This creates a mechanic where the history of a space affects its current utility—repurposing an old battlefield for housing might induce night terrors and plummet productivity, requiring the player to "exorcise" the area or abandon it. Extremely strong echoes may even influence Layer 2 metrics.

## 2. Dependencies
- `002-terrain-grid.md`: Echoes need to be anchored to specific locations/tiles.
- `005-pop-needs.md`: Echoes will affect Pop needs or emotional state (morale).
- `010-chronicle-system.md`: The creation of an echo should likely be tied to significant historical events recorded in the chronicle.

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_intense_event_spawns_psionic_echo() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_event::<IntenseEmotionEvent>();
        app.add_systems(Update, spawn_echoes_from_events_system);

        let event_location = Vec3::new(15.0, 15.0, 0.0);

        // Act
        app.world_mut().send_event(IntenseEmotionEvent {
            emotion: EmotionType::Terror,
            intensity: 100.0,
            location: event_location,
        });
        app.update();

        // Assert
        let mut query = app.world_mut().query::<(&PsionicEcho, &Transform)>();
        let mut found = false;
        for (echo, transform) in query.iter(app.world()) {
            if transform.translation == event_location && echo.emotion == EmotionType::Terror {
                found = true;
                break;
            }
        }
        assert!(found, "An intense emotion event should spawn a Psionic Echo at the location");
    }

    #[test]
    fn test_pop_absorbs_echo_emotion_over_time() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, apply_echo_effects_system);
        app.insert_resource(Time::<Virtual>::from_seconds_f64(1.0));

        let echo_location = Vec3::new(5.0, 5.0, 0.0);
        app.world_mut().spawn((
            PsionicEcho { emotion: EmotionType::Joy, strength: 10.0 },
            Transform::from_translation(echo_location),
            Radius(3.0),
        ));

        let pop = app.world_mut().spawn((
            Pop,
            Transform::from_translation(echo_location),
            Morale { current: 50.0 },
        )).id();

        // Act
        app.update();

        // Assert
        let morale = app.world().get::<Morale>(pop).unwrap().current;
        assert!(morale > 50.0, "Pop standing in a Joy echo should gain morale over time");
    }

    #[test]
    fn test_echo_strength_decays_slowly() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, decay_echoes_system);
        app.insert_resource(Time::<Virtual>::from_seconds_f64(100.0));

        let echo = app.world_mut().spawn(
            PsionicEcho { emotion: EmotionType::Terror, strength: 50.0 }
        ).id();

        // Act
        app.update();

        // Assert
        let strength = app.world().get::<PsionicEcho>(echo).unwrap().strength;
        assert!(strength < 50.0, "Echo strength should decay over long periods of time");
        assert!(strength > 0.0, "Echo should not disappear instantly");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Event)]
pub struct IntenseEmotionEvent {
    pub emotion: EmotionType,
    pub intensity: f32,
    pub location: Vec3,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum EmotionType {
    Terror,
    Joy,
    Despair,
}

#[derive(Component)]
pub struct PsionicEcho {
    pub emotion: EmotionType,
    pub strength: f32,
}

#[derive(Component)]
pub struct Pop;

#[derive(Component)]
pub struct Morale {
    pub current: f32,
}

#[derive(Component)]
pub struct Radius(pub f32);

pub fn spawn_echoes_from_events_system(
    mut events: EventReader<IntenseEmotionEvent>,
    mut commands: Commands,
) {
    for event in events.read() {
        if event.intensity > 50.0 { // Arbitrary threshold for minimal pass
            commands.spawn((
                PsionicEcho {
                    emotion: event.emotion,
                    strength: event.intensity,
                },
                Transform::from_translation(event.location),
                Radius(5.0),
            ));
        }
    }
}

pub fn apply_echo_effects_system(
    time: Res<Time>,
    echo_query: Query<(&PsionicEcho, &Transform, &Radius)>,
    mut pop_query: Query<(&Transform, &mut Morale), With<Pop>>,
) {
    for (echo, e_transform, radius) in echo_query.iter() {
        for (p_transform, mut morale) in pop_query.iter_mut() {
            if e_transform.translation.distance(p_transform.translation) <= radius.0 {
                let delta = match echo.emotion {
                    EmotionType::Joy => 1.0,
                    EmotionType::Terror | EmotionType::Despair => -1.0,
                };
                morale.current += delta * echo.strength * 0.01 * time.delta_secs();
                morale.current = morale.current.clamp(0.0, 100.0);
            }
        }
    }
}

pub fn decay_echoes_system(
    time: Res<Time>,
    mut commands: Commands,
    mut echo_query: Query<(Entity, &mut PsionicEcho)>,
) {
    for (entity, mut echo) in echo_query.iter_mut() {
        echo.strength -= 0.1 * time.delta_secs(); // Slow decay
        if echo.strength <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Refactoring Opportunities**: The distance check in `apply_echo_effects_system` is O(N*M) where N is echoes and M is pops.
- **Code Smells**: Spawning a new entity for every event could lead to overlapping, redundant echoes.
- **Performance/Architecture**: Instead of spawning entity components for echoes, bake the echo intensity into a `PsionicGrid` (similar to temperature or atmosphere diffusion). High emotion events add 'heat' to the grid, which diffuses and decays over time. Pops just read the grid cell they are standing in.
- **API Improvements**: Differentiate between immediate emotional states (transient) and deeply ingrained traits. Long exposure to terror should perhaps have a chance to add a permanent trauma trait to a pop.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] High-intensity events generate Psionic Echoes at their location.
- [ ] Pops standing within an echo have their morale (or relevant needs) modified according to the echo's type and strength.
- [ ] Echoes slowly decay over time and eventually disappear.

## 7. Technical Guidance
- **Code Structure**: Create `src/layer1/mind/psionic_echoes.rs`.
- **Integration Points**:
    - Tie the emission of `IntenseEmotionEvent` to systems handling combat, death, festivals, or extreme starvation.
    - If migrating to a grid-based approach during REFACTOR, hook into the existing simulation tick loop that updates other grids (like `AtmosphereGrid`).
- **Gotchas**: Ensure the passive decay of morale doesn't instantly cause a death spiral. Pops need time to react and leave the area, meaning the Utility AI must factor in the negative echo when pathfinding or selecting work sites.

## 8. Questions
*Builder: add questions here if spec is unclear.*

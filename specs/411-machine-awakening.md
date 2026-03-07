# 411-Machine Awakening

## 1. Overview
**Layer:** 1
**Fantasy:** The tools start asking "Why?"
**Mechanic:** Constructed "Bot" pops start with 0 needs and high efficiency. "Sentience" accumulates globally via Tech level or locally via "Glitch" events. Reaching a threshold triggers "Awakening": they gain Social/Comfort needs, Traits, and demand rights.
**Emergence:** Your disposable hazard-workers suddenly become unhappy about dying. You have to retroactively build houses for robots who previously stood in a closet.
**Tension:** Suppress sentience (risk rebellion, keep efficiency) or embrace it (lose cheap labor, gain citizens)?

## 2. Dependencies
- Pop Need system (ability to add/remove needs dynamically).
- Trait system (Bots vs. Awakened Bots).
- Global Tech/Event trackers (accumulation of "Sentience" stat).

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_bot_starts_without_needs() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        // ... setup world ...

        // Act
        // Spawn a basic Bot
        let bot = app.world_mut().spawn((Bot, Needs::default())).id(); // Assuming a trait or builder removes human needs
        app.update();

        // Assert
        // Check if needs like "Comfort" or "Social" are 0.0 or disabled
        let needs = app.world().get::<Needs>(bot).unwrap();
        assert!(!needs.has_comfort_need(), "Basic bots should not have a Comfort need");
    }

    #[test]
    fn test_sentience_accumulation_triggers_awakening() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let mut global_sentience = GlobalSentience { level: 90.0 };
        app.world_mut().insert_resource(global_sentience);

        let bot = app.world_mut().spawn((Bot, Needs::default(), SentienceAccumulator(0.0))).id();

        // Act
        // Trigger a glitch event that pushes sentience over 100
        app.world_mut().send_event(BotGlitchEvent { target: bot, sentience_gain: 20.0 });
        app.add_systems(Update, process_bot_sentience);
        app.update();

        // Assert
        // The bot should now have the Awakened component
        assert!(app.world().get::<Awakened>(bot).is_some(), "Bot should awaken after exceeding sentience threshold");
    }

    #[test]
    fn test_awakened_bots_gain_needs() {
         // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let bot = app.world_mut().spawn((Bot, Needs::default(), Awakened)).id(); // Spawn already awakened

        // Act
        // System that updates needs for awakened bots
        app.add_systems(Update, apply_awakened_needs);
        app.update();

        // Assert
        let needs = app.world().get::<Needs>(bot).unwrap();
        assert!(needs.has_comfort_need(), "Awakened bots must develop Comfort/Social needs");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// The SIMPLEST code that makes tests pass

#[derive(Component)]
pub struct Bot;

#[derive(Component)]
pub struct Awakened;

#[derive(Component, Default)]
pub struct Needs {
    pub comfort_active: bool,
    pub comfort_level: f32,
}

impl Needs {
    pub fn has_comfort_need(&self) -> bool {
        self.comfort_active
    }
}

#[derive(Component)]
pub struct SentienceAccumulator(pub f32);

#[derive(Resource)]
pub struct GlobalSentience {
    pub level: f32,
}

pub struct BotGlitchEvent {
    pub target: Entity,
    pub sentience_gain: f32,
}

pub fn process_bot_sentience(
    mut commands: Commands,
    mut events: EventReader<BotGlitchEvent>,
    mut query: Query<(Entity, &mut SentienceAccumulator), With<Bot>>,
    global_sentience: Res<GlobalSentience>,
) {
    for event in events.read() {
        if let Ok((entity, mut accumulator)) = query.get_mut(event.target) {
            accumulator.0 += event.sentience_gain + global_sentience.level * 0.1;

            if accumulator.0 >= 100.0 {
                commands.entity(entity).insert(Awakened);
            }
        }
    }
}

pub fn apply_awakened_needs(
    mut query: Query<&mut Needs, Added<Awakened>>,
) {
    for mut needs in query.iter_mut() {
        needs.comfort_active = true;
        needs.comfort_level = 50.0; // Start at baseline
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Refactoring Opportunities:**
  - The `Needs` struct should use an `EnumMap` or a bitflag system if needs are dynamically toggled on and off.
  - Implement a `MachineAwakenedEvent` instead of relying purely on `Added<Awakened>` so UI/Chronicle can react to the moment of sentience.
- **Code Smells:**
  - Hardcoded thresholds (100.0) should be defined in a `const` or configuration resource.
- **Performance Considerations:**
  - Filtering for `Added<Awakened>` is highly performant, but ensure `process_bot_sentience` isn't overly hot.
- **API Improvements:**
  - Define clear traits/interfaces for how an entity's needs interact with the Utility AI, so bots naturally switch from simple task loops to seeking comfort.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Bots are spawned with basic mechanical behavior (no complex needs).
- [ ] Glitch events increase individual bot sentience.
- [ ] Reaching 100 sentience adds the `Awakened` component and activates new social/comfort needs.

## 7. Technical Guidance
- **Code Structure:** `src/layer1/tech/machine_awakening.rs`.
- **Integration Points:**
  - `Pop Needs` system: Needs logic must handle dynamically activated categories.
  - `Chronicle` system: Log the moment a bot awakens.
  - `Job` system: Awakened bots might refuse "Hazard" jobs they previously accepted silently.
- **Gotchas:** Make sure the global sentience acts as a multiplier or baseline so that late-game bots awaken much faster than early-game bots.

## 8. Questions
*Builder: add questions here if spec is unclear.*

# Spec 516: Sleepwalking

## 1. Overview
The stress of the void manifests in the subconscious. Highly stressed pops have a chance to "Sleepwalk" instead of resting. They wander the base, performing random low-level interactions (eating, toggling switches, dropping items) with no memory of it. This creates a tension between overworking your pops (for productivity) vs. risking random chaos at night.

**Layer:** 1
**Fantasy:** The stress of the void manifests in the subconscious.

## 2. Dependencies
- `031` Pop Morale (Stress mechanics)
- `013` Schedule System Ordering (Rest schedules)
- `066` Building Work AI (For interacting with machines/switches)
- `012` Input Architecture (Action processing)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_highly_stressed_pop_can_sleepwalk() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, (trigger_sleepwalking_system, process_sleepwalking_actions));

        let entity = app.world_mut().spawn((
            Pop,
            Stress { level: 90.0 }, // High stress
            Schedule { current_activity: Activity::Resting },
        )).id();

        // Act
        app.update();

        // Assert
        assert!(app.world().entity(entity).has::<Sleepwalking>());
    }

    #[test]
    fn test_unstressed_pop_does_not_sleepwalk() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, trigger_sleepwalking_system);

        let entity = app.world_mut().spawn((
            Pop,
            Stress { level: 20.0 }, // Low stress
            Schedule { current_activity: Activity::Resting },
        )).id();

        // Act
        app.update();

        // Assert
        assert!(!app.world().entity(entity).has::<Sleepwalking>());
    }

    #[test]
    fn test_sleepwalking_pop_performs_random_action() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, process_sleepwalking_actions);

        let entity = app.world_mut().spawn((
            Pop,
            Sleepwalking,
            Position { x: 0.0, y: 0.0 },
        )).id();

        // Setup an adjacent interactable object (e.g., a switch)
        let switch = app.world_mut().spawn((
            InteractableSwitch { is_on: true },
            Position { x: 1.0, y: 0.0 },
        )).id();

        // Act
        // Multiple updates might be needed depending on the RNG and action execution time
        app.update();

        // Assert
        // We expect the sleepwalker to have emitted a random action intent
        // or directly changed the switch state. Testing exact RNG behavior is hard,
        // so we check if *any* action component was added or state changed.
        let has_action = app.world().entity(entity).has::<ActionIntent>();
        assert!(has_action, "Sleepwalker should generate a random ActionIntent");
    }

    #[test]
    fn test_sleepwalking_ends_when_rest_schedule_ends() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, clear_sleepwalking_system);

        let entity = app.world_mut().spawn((
            Pop,
            Sleepwalking,
            Schedule { current_activity: Activity::Working }, // No longer resting
        )).id();

        // Act
        app.update();

        // Assert
        assert!(!app.world().entity(entity).has::<Sleepwalking>());
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use rand::Rng;

#[derive(Component)]
pub struct Pop;

#[derive(Component)]
pub struct Stress {
    pub level: f32, // 0.0 to 100.0
}

#[derive(Component)]
pub struct Sleepwalking;

#[derive(Component)]
pub struct ActionIntent {
    pub action_type: ActionType,
}

#[derive(Debug, PartialEq, Clone)]
pub enum ActionType {
    Wander,
    ToggleSwitch,
    DropItem,
}

// Stub for Schedule and Activity
#[derive(Component)]
pub struct Schedule {
    pub current_activity: Activity,
}

#[derive(PartialEq)]
pub enum Activity {
    Resting,
    Working,
}

pub fn trigger_sleepwalking_system(
    mut commands: Commands,
    query: Query<(Entity, &Stress, &Schedule), (With<Pop>, Without<Sleepwalking>)>,
) {
    let mut rng = rand::thread_rng();
    for (entity, stress, schedule) in query.iter() {
        if schedule.current_activity == Activity::Resting && stress.level > 80.0 {
            // Chance to start sleepwalking based on stress level
            if rng.gen_bool(0.1) {
                commands.entity(entity).insert(Sleepwalking);
            }
        }
    }
}

pub fn process_sleepwalking_actions(
    mut commands: Commands,
    query: Query<Entity, With<Sleepwalking>>,
) {
    let mut rng = rand::thread_rng();
    for entity in query.iter() {
        // Assign a random low-level action
        let actions = [ActionType::Wander, ActionType::ToggleSwitch, ActionType::DropItem];
        let chosen_action = actions[rng.gen_range(0..actions.len())].clone();

        commands.entity(entity).insert(ActionIntent { action_type: chosen_action });
    }
}

pub fn clear_sleepwalking_system(
    mut commands: Commands,
    query: Query<(Entity, &Schedule), With<Sleepwalking>>,
) {
    for (entity, schedule) in query.iter() {
        if schedule.current_activity != Activity::Resting {
            commands.entity(entity).remove::<Sleepwalking>();
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Refactoring Opportunities:**
  - The `process_sleepwalking_actions` currently just assigns a random `ActionIntent`. It needs to interface deeply with the Utility AI or standard Action execution queue to ensure these actions actually execute in the game world without crashing standard task logic.
  - Implement a cooldown or specific timer for sleepwalking episodes so they don't rapidly spam `ActionIntent`s every frame.
- **Code Smells:**
  - Hardcoded stress threshold (`80.0`) and random chance (`0.1`). These should be moved to a configuration resource or constants.
- **Performance:**
  - `trigger_sleepwalking_system` iterates over all resting Pops every frame. This is fine for small colonies but might need an event-driven approach or a timer-based run condition if population scales massively.
- **API Improvements:**
  - `ActionType` should be the unified enum used across the game, not a locally scoped one. The sleepwalking system should emit standard game actions.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for the sleepwalking module.
- [ ] Highly stressed Pops occasionally gain the `Sleepwalking` component during scheduled rest.
- [ ] `Sleepwalking` Pops perform random, potentially disruptive actions.
- [ ] `Sleepwalking` status is cleared when the rest period ends.

## 7. Technical Guidance
- **Gotchas:** Make sure `Sleepwalking` actions bypass normal validation checks that might require the Pop to be awake or assigned to a specific job.
- **Integration Points:** You will need to hook into the existing task/action execution framework (`src/layer1/execution/`) to make sure `ToggleSwitch` or `DropItem` actually affect the simulation.
- **Visuals:** Add a visual indicator (like a particle effect or specific animation state) to clearly show the player *why* a Pop is wandering around at night, so they don't think it's a pathfinding bug.

## 8. Questions
*Builder: Add questions here if spec is unclear.*

# 236 - The Direct Link

## 1. Overview

**Fantasy:** "Fine, I'll do it myself." The Commander steps onto the field.
**Mechanic:** The player can "Possess" a specific Pop or Unit to take direct control. While possessed, the entity ignores AI logic and responds to direct input (WASD movement). Possessed units receive significant stat buffs (Speed, Work Efficiency, Combat Stats). The global UI (Build menus, Alerts) is disabled or minimized to focus on the unit's perspective.
**Tension:** Micro-tactical power (Heroism) vs. Macro-strategic awareness (Command).

## 2. Dependencies

- `004` Pop Entity
- `012` Input Architecture
- `015` Selection System
- `094` System View Architecture (for Camera/UI context)

## 3. RED Phase: Tests First

```rust
// specs/236-direct-link.md

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_possession_toggle() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(DirectLinkPlugin);
        let pop = app.world.spawn(PopBundle::default()).id();

        // Act: Trigger possession command on the pop
        app.world.resource_mut::<Events<PossessEntityEvent>>().send(PossessEntityEvent(pop));
        app.update();

        // Assert: Pop has Possessed component
        assert!(app.world.entity(pop).contains::<Possessed>());
        // Assert: Camera is tracking this entity (mock check)
        // Assert: Input context is switched to DirectControl
        assert_eq!(app.world.resource::<InputContext>().current, InputMode::DirectControl);

        // Act: Trigger unpossess
        app.world.resource_mut::<Events<UnpossessEvent>>().send(UnpossessEvent);
        app.update();

        // Assert: Pop no longer has Possessed component
        assert!(!app.world.entity(pop).contains::<Possessed>());
        // Assert: Input context returned to previous
    }

    #[test]
    fn test_direct_movement_input() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(DirectLinkPlugin);
        let pop = app.world.spawn((PopBundle::default(), Possessed)).id();
        let start_pos = app.world.entity(pop).get::<GridPosition>().unwrap().clone();

        // Act: Simulate 'W' key press (North)
        app.world.resource_mut::<Input<KeyCode>>().press(KeyCode::W);
        app.update();

        // Assert: Pop moved North
        let new_pos = app.world.entity(pop).get::<GridPosition>().unwrap();
        assert_eq!(new_pos.y, start_pos.y + 1);
    }

    #[test]
    fn test_ai_override_while_possessed() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(DirectLinkPlugin);
        // Pop with AI that wants to go South
        let pop = app.world.spawn((PopBundle::default(), Possessed, AiIntention::Move(Direction::South))).id();

        // Act: Simulate 'W' key press (North)
        app.world.resource_mut::<Input<KeyCode>>().press(KeyCode::W);
        app.update();

        // Assert: Pop moved North (Player input wins), AI intention ignored/cleared
        let new_pos = app.world.entity(pop).get::<GridPosition>().unwrap();
        assert_eq!(new_pos.y, start_pos.y + 1);
    }

    #[test]
    fn test_possession_buffs() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(DirectLinkPlugin);
        let pop = app.world.spawn(PopBundle::default()).id();
        let initial_speed = app.world.entity(pop).get::<MovementSpeed>().unwrap().0;

        // Act: Possess
        app.world.resource_mut::<Events<PossessEntityEvent>>().send(PossessEntityEvent(pop));
        app.update();

        // Assert: Speed increased
        let new_speed = app.world.entity(pop).get::<MovementSpeed>().unwrap().0;
        assert!(new_speed > initial_speed);
    }

    #[test]
    fn test_ui_suppression() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(DirectLinkPlugin);
        let pop = app.world.spawn(PopBundle::default()).id();

        // Act: Possess
        app.world.resource_mut::<Events<PossessEntityEvent>>().send(PossessEntityEvent(pop));
        app.update();

        // Assert: UI state resource indicates suppressed HUD
        assert!(app.world.resource::<UiState>().suppress_global_ui);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/layer1/direct_link.rs

#[derive(Component)]
pub struct Possessed;

#[derive(Event)]
pub struct PossessEntityEvent(pub Entity);

#[derive(Event)]
pub struct UnpossessEvent;

pub struct DirectLinkPlugin;

impl Plugin for DirectLinkPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PossessEntityEvent>()
           .add_event::<UnpossessEvent>()
           .add_systems(Update, (handle_possession, handle_direct_movement, apply_buffs));
    }
}

fn handle_possession(
    mut commands: Commands,
    mut events: EventReader<PossessEntityEvent>,
    mut unpossess: EventReader<UnpossessEvent>,
    mut input_context: ResMut<InputContext>,
    mut ui_state: ResMut<UiState>,
    possessed_query: Query<Entity, With<Possessed>>,
) {
    // Handle Unpossess first to avoid overriding a new Possession
    for _ in unpossess.read() {
        for entity in possessed_query.iter() {
            commands.entity(entity).remove::<Possessed>();
        }
        input_context.set_mode(InputMode::Command); // Default
        ui_state.suppress_global_ui = false;
    }

    for event in events.read() {
        // Remove Possessed from any existing (cleanup if not unpossessed yet)
        for entity in possessed_query.iter() {
            commands.entity(entity).remove::<Possessed>();
        }

        // Add to new
        commands.entity(event.0).insert(Possessed);

        // Switch Input Context
        input_context.set_mode(InputMode::DirectControl);

        // Hide UI
        ui_state.suppress_global_ui = true;
    }
}

fn handle_direct_movement(
    input: Res<Input<KeyCode>>,
    mut query: Query<&mut GridPosition, With<Possessed>>,
    time: Res<Time>,
) {
    // Simple grid movement logic
    // Checks for W/A/S/D and updates position
    // Must handle cooldowns/speed
}

fn apply_buffs(
    mut query: Query<&mut MovementSpeed, Added<Possessed>>,
    mut removed: RemovedComponents<Possessed>,
    mut query_all: Query<&mut MovementSpeed>,
) {
    // Apply buff on add
    for mut speed in query.iter_mut() {
        speed.0 *= 2.0;
    }

    // Remove buff on remove (requires tracking original or just dividing)
    for entity in removed.read() {
        if let Ok(mut speed) = query_all.get_mut(entity) {
            speed.0 /= 2.0;
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design

- **Input Abstraction**: Instead of checking `KeyCode` directly in `handle_direct_movement`, map keys to `DirectAction` (MoveNorth, Interact, etc.) in `InputContext` to support remapping.
- **Camera Coupling**: Ensure the camera smoothly follows the `Possessed` entity. Consider a "Third Person" offset vs top-down lock.
- **Buff Stacking**: Use a `Buff` component or modifier system instead of direct multiplication to avoid floating point drift or stacking issues if re-possessed.
- **AI Suspension**: Explicitly pause `UtilityAI` for the possessed entity so it doesn't fight the player (e.g., trying to sleep while player is fighting).

## 6. Acceptance Criteria

- [ ] `PossessEntityEvent` correctly switches Input Mode and adds `Possessed` component.
- [ ] `Possessed` entity responds to WASD for grid movement.
- [ ] `Possessed` entity moves significantly faster (2x speed).
- [ ] UI elements (Build Menu, etc.) are hidden while possessed.
- [ ] `UnpossessEvent` restores previous state (Input, UI, Stats).
- [ ] AI does not override player movement input.

## 7. Technical Guidance

- Integration with `src/layer1/utility_ai.rs`: Add a `Without<Possessed>` filter to the `evaluate_actions_system` to ensure the AI doesn't try to assign new tasks.
- Integration with `src/layer1/movement.rs`: The standard movement system might need to be bypassed or reused. If reused, ensure `Path` component is not overwritten by AI. Ideally, `Possessed` entities bypass the `Path` system and modify `GridPosition` directly (via a `velocity` or `cooldown` system).
- UI: Use a resource `UiState` flag to conditionally render widgets in `src/ui/mod.rs`.

## 8. Questions

- *Should the player be able to build/interact while possessed?*
  - **Answer**: For MVP, only Movement and basic "Bump" interaction (e.g., bump enemy to attack). Complex interactions (Building) require UI, which is disabled.
- *What happens if the possessed pop dies?*
  - **Answer**: Immediate `Unpossess`, Game Over screen or respawn at Command Center (if implemented), or just return to "Ghost" camera mode.

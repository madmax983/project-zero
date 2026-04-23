# 1142 - The Scapegoat

## 1. Overview
When things go wrong, the mob needs someone to blame. It doesn't matter if it's true. High Unrest generates a "Scapegoat" target (a specific Pop or minority group). Banishing or Punishing them lowers Unrest immediately but adds "Guilt" or "Injustice" traits to others.

This spec introduces the mechanic where `Unrest` crossing a threshold triggers a `ScapegoatEvent` targeting a specific `Pop`. The player (or a system) can take action to punish the scapegoat, which reduces unrest but applies a `Guilt` trait to other pops in the colony.

## 2. Dependencies
- `Pop` component
- `Unrest` resource or component (acting at a colony/grid level)
- `Trait` system (specifically `Guilt` and `ScapegoatTarget`)
- Event system for `ScapegoatEvent` and `ScapegoatPunishedEvent`

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_high_unrest_generates_scapegoat() {
        // Arrange
        let mut app = App::new();
        app.add_event::<ScapegoatEvent>();
        app.init_resource::<Unrest>();
        app.add_systems(Update, generate_scapegoat_system);

        let pop_id = app.world_mut().spawn(Pop).id();
        app.world_mut().resource_mut::<Unrest>().level = 80.0; // High unrest threshold

        // Act
        app.update();

        // Assert
        let events = app.world().resource::<Events<ScapegoatEvent>>();
        let mut reader = events.get_reader();
        let evs: Vec<_> = reader.read(events).collect();

        assert_eq!(evs.len(), 1, "Should generate exactly one ScapegoatEvent");
        assert_eq!(evs[0].target, pop_id, "The spawned pop should be the target");

        // Also check if the pop gets a ScapegoatTarget component/trait
        assert!(app.world().get::<ScapegoatTarget>(pop_id).is_some(), "Target pop must have the ScapegoatTarget component");
    }

    #[test]
    fn test_punishing_scapegoat_reduces_unrest_and_adds_guilt() {
        // Arrange
        let mut app = App::new();
        app.add_event::<ScapegoatPunishedEvent>();
        app.init_resource::<Unrest>();
        app.add_systems(Update, punish_scapegoat_system);

        let scapegoat_id = app.world_mut().spawn((Pop, ScapegoatTarget)).id();
        let bystander_id = app.world_mut().spawn(Pop).id();

        app.world_mut().resource_mut::<Unrest>().level = 80.0;

        // Act
        app.world_mut().send_event(ScapegoatPunishedEvent { target: scapegoat_id });
        app.update();

        // Assert
        let unrest = app.world().resource::<Unrest>();
        assert!(unrest.level < 80.0, "Unrest should be reduced after punishing the scapegoat");

        assert!(app.world().get::<Guilt>(bystander_id).is_some(), "Bystander pop should receive the Guilt trait");

        // The scapegoat should be punished (e.g., dead/despawned or banished)
        // For minimal implementation, we'll assume they get a Banished component.
        assert!(app.world().get::<Banished>(scapegoat_id).is_some(), "Scapegoat should receive the Banished component");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Pop;

#[derive(Component)]
pub struct ScapegoatTarget;

#[derive(Component)]
pub struct Guilt;

#[derive(Component)]
pub struct Banished;

#[derive(Resource, Default)]
pub struct Unrest {
    pub level: f32,
}

#[derive(Event)]
pub struct ScapegoatEvent {
    pub target: Entity,
}

#[derive(Event)]
pub struct ScapegoatPunishedEvent {
    pub target: Entity,
}

pub fn generate_scapegoat_system(
    unrest: Res<Unrest>,
    pops: Query<Entity, (With<Pop>, Without<ScapegoatTarget>)>,
    mut commands: Commands,
    mut events: EventWriter<ScapegoatEvent>,
) {
    if unrest.level >= 80.0 {
        if let Some(target) = pops.iter().next() {
            commands.entity(target).insert(ScapegoatTarget);
            events.send(ScapegoatEvent { target });
        }
    }
}

pub fn punish_scapegoat_system(
    mut events: EventReader<ScapegoatPunishedEvent>,
    mut unrest: ResMut<Unrest>,
    mut commands: Commands,
    bystanders: Query<Entity, (With<Pop>, Without<ScapegoatTarget>)>,
) {
    for event in events.read() {
        unrest.level -= 30.0;
        commands.entity(event.target).insert(Banished);

        for bystander in bystanders.iter() {
            commands.entity(bystander).insert(Guilt);
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Selection Logic:** The `generate_scapegoat_system` currently picks the first available Pop. This should be refined to select Pops based on specific traits (e.g., minority status, low social standing, existing unpopular traits) to fit the narrative.
- **Unrest Caps:** Ensure `unrest.level` does not drop below zero when subtracting the unrest penalty.
- **Cooldown:** Implement a cooldown for `generate_scapegoat_system` so it doesn't repeatedly fire multiple events while unrest is hovering around the threshold.
- **Guilt Stacking:** If a Pop already has `Guilt`, consider adding a severity level instead of just re-inserting the marker component.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] High unrest triggers the scapegoat event.
- [ ] Punishing a scapegoat successfully reduces unrest but applies Guilt to remaining Pops.

## 7. Technical Guidance
- Integrate with the existing `Pop` and `Unrest` system. If `Unrest` is a component on a colony entity instead of a global resource, adjust queries accordingly.
- Keep the `Guilt` component simple initially, but consider how it integrates with the mood/needs system (e.g., causing a steady drain on happiness).
- Consider logging these events to the Chronicle for lore integration ("The Scapegoat of Year 42").

## 8. Questions
*Builder: add questions here if spec is unclear.*

# 742 - The Guilt-Eater Fungi

## 1. Overview
A miraculous cure for stress that also removes your colonists' sense of self-preservation. Cultivating a specific alien fungus produces a consumable that instantly sets Unrest to zero and maximizes Morale. However, it applies a permanent "Apathetic" trait. Apathetic pops ignore all hazard warnings, refuse to flee from combat, and won't seek medical attention.

## 2. Dependencies
- Layer 1 Agriculture and Consumables systems.
- Pop Morale and Needs tracking.
- Pop trait system and Utility AI (for ignoring hazards).

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_consuming_guilt_eater_fungi_maximizes_morale_and_adds_apathetic_trait() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_event::<ConsumeItemEvent>();
        app.add_systems(Update, process_guilt_eater_consumption_system);

        let pop = app.world_mut().spawn((
            Pop,
            Needs { morale: 10.0, unrest: 80.0 },
        )).id();

        // Act
        app.world_mut().send_event(ConsumeItemEvent {
            pop_entity: pop,
            item_type: ConsumableType::GuiltEaterFungi,
        });
        app.update();

        // Assert
        let needs = app.world().get::<Needs>(pop).unwrap();
        assert_eq!(needs.morale, 100.0, "Morale should be maximized");
        assert_eq!(needs.unrest, 0.0, "Unrest should be zeroed out");
        assert!(app.world().get::<ApatheticTrait>(pop).is_some(), "Pop should gain the Apathetic trait");
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Pop;

#[derive(Component)]
pub struct Needs {
    pub morale: f32,
    pub unrest: f32,
}

#[derive(Component)]
pub struct ApatheticTrait;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ConsumableType {
    StandardRation,
    GuiltEaterFungi,
}

#[derive(Event)]
pub struct ConsumeItemEvent {
    pub pop_entity: Entity,
    pub item_type: ConsumableType,
}

pub fn process_guilt_eater_consumption_system(
    mut commands: Commands,
    mut events: EventReader<ConsumeItemEvent>,
    mut query: Query<&mut Needs, With<Pop>>,
) {
    for event in events.read() {
        if event.item_type == ConsumableType::GuiltEaterFungi {
            if let Ok(mut needs) = query.get_mut(event.pop_entity) {
                needs.morale = 100.0;
                needs.unrest = 0.0;
                commands.entity(event.pop_entity).insert(ApatheticTrait);
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Integrate the `ApatheticTrait` into the Utility AI scoring functions (`evaluate_actions_system`). Actions like "Flee," "Seek Healing," or "React to Hazard" should have their scores multiplied by 0 for pops with this trait.
- Ensure the `ConsumableType::GuiltEaterFungi` is added to the valid outputs of the farming/foraging systems.

## 6. Acceptance Criteria (Testable!)
- [ ] RED phase tests pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Pops consuming the fungi gain max morale, zero unrest, and the `ApatheticTrait`.

## 7. Technical Guidance
- Be careful with how `ApatheticTrait` interacts with essential survival tasks (like eating standard food). The trait should primarily block *reactive* survival behaviors (hazards, combat, medicine), not routine metabolism.

## 8. Questions
*Builder: add questions here if spec is unclear.*

# 1059: System Sovereignty

## 1. Overview
As the colony transitions from a small survival outpost to an established power, it can declare "System Sovereignty." This action changes the colony's status from a corporate/vassal asset to an independent entity on the Layer 2/3 diplomatic map. Doing so stops mandatory tribute payments and unlocks advanced diplomacy, but immediately triggers hostility from the previous overlord faction.

## 2. Dependencies
- Base simulation framework (`App`, `World`)
- Faction relationships (Layer 3 diplomacy abstraction)
- Event system (`DeclarationOfIndependenceEvent`)
- Economy/Tribute systems

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_declaration_changes_colony_status() {
        // Arrange
        let mut app = App::new();
        app.insert_resource(ColonyStatus { is_sovereign: false, overlord_id: Some(1) });
        app.add_event::<DeclarationOfIndependenceEvent>();
        app.add_systems(Update, process_sovereignty_declaration);

        // Act
        app.world_mut().send_event(DeclarationOfIndependenceEvent);
        app.update();

        // Assert
        let status = app.world().resource::<ColonyStatus>();
        assert!(status.is_sovereign);
        assert_eq!(status.overlord_id, None);
    }

    #[test]
    fn test_declaration_triggers_overlord_hostility() {
        // Arrange
        let mut app = App::new();
        app.insert_resource(ColonyStatus { is_sovereign: false, overlord_id: Some(2) });
        app.add_event::<DeclarationOfIndependenceEvent>();

        // Mock relations map: Faction 2 starts Neutral (0)
        let mut relations = FactionRelations::default();
        relations.scores.insert(2, 0);
        app.insert_resource(relations);

        app.add_systems(Update, process_sovereignty_declaration);

        // Act
        app.world_mut().send_event(DeclarationOfIndependenceEvent);
        app.update();

        // Assert: Overlord should now be deeply hostile (-100)
        let relations = app.world().resource::<FactionRelations>();
        assert_eq!(*relations.scores.get(&2).unwrap(), -100);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use bevy::utils::HashMap;

#[derive(Resource)]
pub struct ColonyStatus {
    pub is_sovereign: bool,
    pub overlord_id: Option<u32>,
}

#[derive(Resource, Default)]
pub struct FactionRelations {
    pub scores: HashMap<u32, i32>, // -100 (Hostile) to 100 (Allied)
}

#[derive(Event)]
pub struct DeclarationOfIndependenceEvent;

pub fn process_sovereignty_declaration(
    mut events: EventReader<DeclarationOfIndependenceEvent>,
    mut status: ResMut<ColonyStatus>,
    mut relations: ResMut<FactionRelations>,
) {
    for _ in events.read() {
        if status.is_sovereign {
            continue; // Already sovereign
        }

        if let Some(overlord) = status.overlord_id {
            // Instant hostility
            relations.scores.insert(overlord, -100);
        }

        status.is_sovereign = true;
        status.overlord_id = None;
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Diplomatic Broadcast**: Emitting a `WarDeclarationEvent` alongside the relationship change would allow Layer 2 fleet generators to immediately spawn an attack fleet.
- **Tribute Disconnect**: Ensure that the `process_sovereignty_declaration` system removes any pending tribute quests/timers from the `ColonyResources` or quest log.
- **Prerequisites**: In reality, declaring sovereignty should require certain conditions (Population size, Military power). The UI invoking the event should enforce this, but the system could also validate it.

## 6. Acceptance Criteria
- [ ] All RED phase tests pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage for new code is >= 85%.
- [ ] Declaring sovereignty severs the overlord connection.
- [ ] The former overlord's relation score is set to maximum hostility.

## 7. Technical Guidance
- Faction IDs (`u32`) are placeholders for proper `Entity` references to faction data components, but work fine for MVP structural logic.
- Ensure the event is only processed once per declaration to avoid redundant logging or trigger spam.

## 8. Questions
*Builder: add questions here if spec is unclear.*

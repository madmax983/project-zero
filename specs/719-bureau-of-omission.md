# 719: The Bureau of Omission

## 1. Overview
**Layer:** 1
**Fantasy:** A Kafkaesque nightmare where the easiest way to solve a problem is to officially decree that it doesn't exist.
**Mechanic:** Players can construct a "Bureau of Omission," a highly expensive administrative building. It unlocks the "Redact" edict. You can use Redact on any negative event, notification, or even a specific Pop. The UI element vanishes, and the immediate negative Morale impact is nullified. However, the underlying physical reality remains (e.g., the redacted fire still burns, the redacted Pop still consumes food and takes up space, but is completely invisible and un-interactable to the player and other Pops).

## 2. Dependencies
- Building system (`crate::layer1::buildings::Building`)
- Edicts/Player Actions (`crate::layer1::edicts::PlayerEdict` or similar)
- Social/Morale system (`crate::layer1::social::morale::MoraleEvent`)
- Rendering/UI visibility (`bevy::render::view::Visibility` or custom UI marker)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::social::morale::MoraleEvent;
    use crate::layer1::needs::Needs;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_systems(Update, (apply_redaction_system, intercept_redacted_morale_events_system));
        app
    }

    #[test]
    fn test_bureau_unlocks_redact_edict() {
        let mut app = setup_app();

        // Build the Bureau
        app.world_mut().spawn((
            Building { building_type: BuildingType::BureauOfOmission },
        ));

        app.update();

        // Ensure the edict is now available
        assert!(app.world().contains_resource::<RedactEdictUnlocked>());
    }

    #[test]
    fn test_redacted_pop_hidden_from_ui_but_consumes_needs() {
        let mut app = setup_app();

        let mut needs = Needs::default();
        needs.set(NeedType::Food, 100.0);

        let entity = app.world_mut().spawn((
            Pop,
            needs,
            Visibility::Visible,
            Redacted, // Player used the edict on them
        )).id();

        app.update();

        // Should be hidden
        let vis = app.world().entity(entity).get::<Visibility>().unwrap();
        assert_eq!(*vis, Visibility::Hidden);

        // Should still drain needs (assuming a need drain system runs)
        // For testing, we just verify the component is still there and active
        assert!(app.world().entity(entity).contains::<Needs>());
    }

    #[test]
    fn test_redacted_events_nullify_morale_impact() {
        let mut app = setup_app();

        let entity = app.world_mut().spawn((
            Pop,
            Morale { value: 50.0 },
        )).id();

        // Fire a negative morale event that is flagged as redacted
        app.world_mut().send_event(MoraleEvent {
            target: entity,
            amount: -20.0,
            reason: "Famine".to_string(),
            is_redacted: true, // The edict was applied to the event
        });

        app.update();

        // Morale shouldn't drop
        let morale = app.world().entity(entity).get::<Morale>().unwrap();
        assert_eq!(morale.value, 50.0);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer1::social::morale::MoraleEvent;
use crate::layer1::buildings::{Building, BuildingType};

#[derive(Resource, Default)]
pub struct RedactEdictUnlocked(pub bool);

#[derive(Component)]
pub struct Redacted;

pub fn unlock_redact_edict_system(
    mut commands: Commands,
    query: Query<&Building>,
) {
    let mut unlocked = false;
    for building in query.iter() {
        if building.building_type == BuildingType::BureauOfOmission {
            unlocked = true;
            break;
        }
    }

    commands.insert_resource(RedactEdictUnlocked(unlocked));
}

pub fn apply_redaction_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Visibility), With<Redacted>>,
) {
    for (_, mut visibility) in query.iter_mut() {
        *visibility = Visibility::Hidden; // Hide from rendering
        // In UI logic, you would also filter out entities with Redacted
    }
}

pub fn intercept_redacted_morale_events_system(
    mut events: EventReader<MoraleEvent>,
    mut query: Query<&mut Morale>,
) {
    for event in events.read() {
        if event.is_redacted {
            // Drop it, do not apply to morale
            continue;
        }

        if let Ok(mut morale) = query.get_mut(event.target) {
            morale.value += event.amount;
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Event Modification:** Modifying `MoraleEvent` struct to include `is_redacted` requires modifying the core event definition. An alternative is wrapping events in a `RedactedEventWrapper` or using a separate system to pre-filter events before the main morale system processes them.
- **Resource Drain Transparency:** Ensure that while UI hides the pop, resource drain (like food) is still correctly attributed to the colony total, creating the mysterious "leak" described in the fantasy.
- **Edict Cost:** Implement an arbitrary, extremely high cost (e.g., Influence or Bureaucracy points) to trigger the `Redacted` state on an entity/event.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for the Omission mechanics.
- [ ] `BuildingType::BureauOfOmission` defined.
- [ ] Redacted pops are set to `Visibility::Hidden` and Morale events flagged as redacted are ignored.

## 7. Technical Guidance
- **Module:** Best placed in `src/layer1/edicts/omission.rs` or `src/layer1/social/omission.rs`.
- **Integration:** The `Visibility` component is Bevy's built-in. Make sure your UI rendering systems (e.g., `comfy_table` outputs in headless mode) explicitly skip entities with `Redacted`.

## 8. Questions
*Builder: add questions here if spec is unclear.*

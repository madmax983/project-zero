# 907: The Architect's Signature

## 1. Overview
Buildings reflect the personality and flaws of their creators. Every structure built inherits a subtle hidden trait from the primary builder Pop (e.g., "Meticulous", "Rushed", "Paranoid"). For example, a paranoid builder might add hidden reinforced locks, making the building harder to sabotage but slower to access. This creates procedural quirks in the colony's infrastructure based on who happened to wield the hammer.

## 2. Dependencies
- `layer1::pops::PopPersonality` (or trait system)
- `layer1::construction::ConstructionJobCompletedEvent`
- `layer1::geology::Building`

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::construction::ConstructionJobCompletedEvent;

    #[test]
    fn test_building_inherits_builder_trait() {
        let mut app = App::new();
        app.add_event::<ConstructionJobCompletedEvent>();
        app.add_systems(Update, apply_architect_signature_system);

        // Arrange
        let builder = app.world_mut().spawn((
            ArchitectTrait::Paranoid,
        )).id();

        let building = app.world_mut().spawn_empty().id();

        app.world_mut().send_event(ConstructionJobCompletedEvent {
            building_entity: building,
            primary_builder: builder,
        });

        // Act
        app.update();

        // Assert
        let signature = app.world().get::<ArchitectSignature>(building).unwrap();
        assert_eq!(signature.trait_type, ArchitectTrait::Paranoid);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer1::construction::ConstructionJobCompletedEvent;

#[derive(Component, Clone, PartialEq, Debug)]
pub enum ArchitectTrait {
    Meticulous,
    Rushed,
    Paranoid,
    Claustrophobic,
}

#[derive(Component)]
pub struct ArchitectSignature {
    pub trait_type: ArchitectTrait,
}

pub fn apply_architect_signature_system(
    mut commands: Commands,
    mut completion_events: EventReader<ConstructionJobCompletedEvent>,
    builder_query: Query<&ArchitectTrait>,
) {
    for event in completion_events.read() {
        if let Ok(builder_trait) = builder_query.get(event.primary_builder) {
            commands.entity(event.building_entity).insert(ArchitectSignature {
                trait_type: builder_trait.clone(),
            });
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Integration**: The minimal implementation assigns the trait. The next refactor should involve integrating `ArchitectSignature` into the systems that evaluate building efficiency, safety, or access speed (e.g., matching `ArchitectTrait::Paranoid` to lower sabotage risk but higher entry latency).
- **Default Traits**: If a builder doesn't have an explicit `ArchitectTrait`, we should either assign a default "Standard" signature or randomly generate one based on the builder's background.

## 6. Acceptance Criteria
- [ ] All tests pass.
- [ ] Test coverage >= 85%.
- [ ] Buildings successfully inherit `ArchitectSignature` from their `primary_builder` upon construction completion.

## 7. Technical Guidance
- Ensure `ConstructionJobCompletedEvent` includes the `primary_builder` entity. If multiple pops contribute to construction, you may need logic to determine the "primary" contributor (e.g., the one who did the most work).

## 8. Questions

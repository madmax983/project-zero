# 1124 - Diplomatic Fashion

## 1. Overview

**Layer:** 3 -> 1
**Fantasy:** Dressing the part to survive the court.
**Mechanic:** Alien diplomats have "Preferred Attire" traits (e.g., "Organic Fibers Only", "Full Enviro-Suits"). Interacting with them while wearing the wrong clothes causes massive Relations penalties.

This spec implements the foundational components and systems for Diplomatic Fashion. It introduces the concept of an `Apparel` component for Pops and a `PreferredAttire` trait for Diplomat entities, and implements a relationship penalty/bonus system when interacting.

## 2. Dependencies

- Layer 1 `Pop` and `Equipment` structures.
- Layer 3 Diplomatic relationship structures (e.g., `DiplomaticRelations`).

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_diplomatic_fashion_match_provides_bonus() {
        // Arrange: Setup test data
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let ambassador_entity = app.world_mut().spawn((
            Diplomat,
            PreferredAttire { tags: vec![AttireTag::Organic] },
            DiplomaticRelations { reputation: 50 },
        )).id();

        let envoy_entity = app.world_mut().spawn((
            Pop,
            Apparel { tags: vec![AttireTag::Organic, AttireTag::Ceremonial] },
        )).id();

        app.world_mut().resource_mut::<Events<DiplomaticMeetingEvent>>().send(
            DiplomaticMeetingEvent {
                ambassador: ambassador_entity,
                envoy: envoy_entity,
            }
        );

        app.add_systems(Update, evaluate_fashion_system);

        // Act: Call the feature
        app.update();

        // Assert: Verify expected behavior (Reputation should increase because attire matches)
        let relations = app.world().get::<DiplomaticRelations>(ambassador_entity).unwrap();
        assert!(relations.reputation > 50, "Reputation should increase due to matching attire.");
    }

    #[test]
    fn test_diplomatic_fashion_mismatch_causes_penalty() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let ambassador_entity = app.world_mut().spawn((
            Diplomat,
            PreferredAttire { tags: vec![AttireTag::HeavyArmor] },
            DiplomaticRelations { reputation: 50 },
        )).id();

        let envoy_entity = app.world_mut().spawn((
            Pop,
            Apparel { tags: vec![AttireTag::Organic] },
        )).id();

        app.world_mut().resource_mut::<Events<DiplomaticMeetingEvent>>().send(
            DiplomaticMeetingEvent {
                ambassador: ambassador_entity,
                envoy: envoy_entity,
            }
        );

        app.add_systems(Update, evaluate_fashion_system);

        // Act
        app.update();

        // Assert: Verify expected behavior (Reputation should decrease because attire does not match)
        let relations = app.world().get::<DiplomaticRelations>(ambassador_entity).unwrap();
        assert!(relations.reputation < 50, "Reputation should decrease due to attire mismatch.");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Pop;

#[derive(Component)]
pub struct Diplomat;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AttireTag {
    Organic,
    HeavyArmor,
    Ceremonial,
}

#[derive(Component)]
pub struct Apparel {
    pub tags: Vec<AttireTag>,
}

#[derive(Component)]
pub struct PreferredAttire {
    pub tags: Vec<AttireTag>,
}

#[derive(Component)]
pub struct DiplomaticRelations {
    pub reputation: i32,
}

#[derive(Event)]
pub struct DiplomaticMeetingEvent {
    pub ambassador: Entity,
    pub envoy: Entity,
}

pub fn evaluate_fashion_system(
    mut events: EventReader<DiplomaticMeetingEvent>,
    mut query_ambassador: Query<(&PreferredAttire, &mut DiplomaticRelations)>,
    query_envoy: Query<&Apparel>,
) {
    for event in events.read() {
        if let Ok((preferred, mut relations)) = query_ambassador.get_mut(event.ambassador) {
            if let Ok(apparel) = query_envoy.get(event.envoy) {
                let mut matched = false;
                for tag in &preferred.tags {
                    if apparel.tags.contains(tag) {
                        matched = true;
                        break;
                    }
                }

                if matched {
                    relations.reputation += 10;
                } else {
                    relations.reputation -= 10;
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design

- **Performance**: Use `HashSet` for tags if the number of tags grows large, though `Vec` is fine for MVP.
- **Design**: The penalty/bonus values (+10/-10) should be extracted into configurable constants or tied to the severity of the `PreferredAttire` trait.
- **Integration**: The event should trigger a `ChronicleEvent` so the player sees the consequence of their fashion choices.

## 6. Acceptance Criteria (Testable!)

- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Mismatching attire during `DiplomaticMeetingEvent` penalizes relations.
- [ ] Matching attire provides a relations bonus.

## 7. Technical Guidance

- Place the events and systems in `src/layer3/diplomacy/`.
- `Apparel` can act as an extension of the existing equipment system on `Pop`s.
- Register `evaluate_fashion_system` carefully in the diplomatic evaluation phase of the Bevy schedule.

## 8. Questions

*Builder: add questions here if spec is unclear.*
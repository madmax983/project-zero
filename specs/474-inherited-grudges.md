# Inherited Grudges (Spec 474)

## Overview
When a Pop suffers a severe wrong (e.g., unjust imprisonment or extreme starvation) caused by or associated with another specific Pop (like the Governor), they generate a "Vendetta." This Vendetta is passed down genetically or culturally to their descendants. Descendants will spontaneously refuse to work with, or actively sabotage, the descendants of the original transgressor. This forces the player to manage short-term injustices knowing they will metastasize into permanent, generational structural flaws.

## Dependencies
- `047` Pop Relationships (Implemented)
- `062` Pop Lifecycle (Implemented)
- `016` Utility AI System (Implemented)

## RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use super::*;

    #[test]
    fn test_severe_wrong_creates_vendetta() {
        let mut world = World::new();

        let victim = world.spawn(PopBundle::default()).id();
        let perpetrator = world.spawn(PopBundle::default()).id();

        // Simulate a severe wrong
        world.resource_mut::<Events<SevereWrongEvent>>().send(SevereWrongEvent {
            victim,
            perpetrator,
            reason: WrongReason::UnjustImprisonment,
        });

        world.run_system_once(handle_severe_wrongs_system).unwrap();

        let vendettas = world.get::<Vendettas>(victim).unwrap();
        assert!(vendettas.targets.contains(&perpetrator));
    }

    #[test]
    fn test_vendetta_is_inherited_by_offspring() {
        let mut world = World::new();

        let perpetrator = world.spawn(PopBundle::default()).id();

        let parent = world.spawn((
            PopBundle::default(),
            Vendettas { targets: vec![perpetrator] },
        )).id();

        let child = world.spawn(PopBundle::default()).id();

        world.resource_mut::<Events<ChildBornEvent>>().send(ChildBornEvent {
            parent,
            child,
        });

        world.run_system_once(inherit_vendettas_system).unwrap();

        let child_vendettas = world.get::<Vendettas>(child).unwrap();
        // Child should inherit the grudge against the perpetrator
        assert!(child_vendettas.targets.contains(&perpetrator));
    }
}
```

## GREEN Phase: Minimal Implementation

```rust
use bevy_ecs::prelude::*;

#[derive(Clone, PartialEq, Debug)]
pub enum WrongReason {
    UnjustImprisonment,
    StarvationNeglect,
}

#[derive(Event)]
pub struct SevereWrongEvent {
    pub victim: Entity,
    pub perpetrator: Entity,
    pub reason: WrongReason,
}

#[derive(Event)]
pub struct ChildBornEvent {
    pub parent: Entity,
    pub child: Entity,
}

#[derive(Component, Default)]
pub struct Vendettas {
    pub targets: Vec<Entity>,
}

pub fn handle_severe_wrongs_system(
    mut commands: Commands,
    mut events: EventReader<SevereWrongEvent>,
    mut query: Query<&mut Vendettas>,
) {
    for event in events.read() {
        if let Ok(mut vendettas) = query.get_mut(event.victim) {
            if !vendettas.targets.contains(&event.perpetrator) {
                vendettas.targets.push(event.perpetrator);
            }
        } else {
            commands.entity(event.victim).insert(Vendettas {
                targets: vec![event.perpetrator],
            });
        }
    }
}

pub fn inherit_vendettas_system(
    mut commands: Commands,
    mut events: EventReader<ChildBornEvent>,
    parent_query: Query<&Vendettas>,
    mut child_query: Query<&mut Vendettas>,
) {
    for event in events.read() {
        if let Ok(parent_vendettas) = parent_query.get(event.parent) {
            if let Ok(mut child_vendettas) = child_query.get_mut(event.child) {
                for target in &parent_vendettas.targets {
                    if !child_vendettas.targets.contains(target) {
                        child_vendettas.targets.push(*target);
                    }
                }
            } else {
                commands.entity(event.child).insert(Vendettas {
                    targets: parent_vendettas.targets.clone(),
                });
            }
        }
    }
}
```

## REFACTOR Phase: Quality & Design
- **Sabotage Implementation**: Hook `Vendettas` into the `Utility AI System`. If a Pop is evaluating a task that involves a target of their vendetta (e.g., treating their wounds, working in the same building), the score should be zeroed out or a "Sabotage" action should be prioritized instead.
- **Target Lineage**: Vendettas should not just target the original perpetrator, but the *descendants* of the perpetrator. This requires a robust lineage tracking system to map entity relationships.
- **Resolution**: Add a mechanism for "Blood Money" or specific diplomatic events that can clear a Vendetta, otherwise it becomes an infinite permanent malus.

## Acceptance Criteria
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Pops generate a `Vendettas` component when subjected to a `SevereWrongEvent`.
- [ ] The `Vendettas` component is properly cloned/inherited by children during a `ChildBornEvent`.

## Technical Guidance
- Currently, this minimal implementation only tracks the specific entity of the perpetrator. To fully realize the fantasy, `Vendettas` should probably track a `LineageID` or `FamilyID` rather than raw `Entity` IDs, to ensure the grudge persists even after the original perpetrator dies.

## Questions
*Builder: add questions here if spec is unclear.*

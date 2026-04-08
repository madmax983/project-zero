# 886: The Cannibal's Empathy

## 1. Overview
Extreme survival breeds a terrifying closeness. During severe famines, Pops might consume the deceased to survive. While doing so, they inherit a fraction of the dead Pop's `Memories` and `Relationship` scores, literally internalizing the person they ate. Allowing cannibalism prevents starvation but irreversibly tangles the social web of your colony with borrowed traumas and conflicting loyalties.

## 2. Dependencies
- `036-pop-memory.md` (Memory system)
- `047-pop-relationships.md` (Affinity system)
- `005-pop-needs.md` (Hunger logic)
- `057-funeral-rites.md` (Corpse entities)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_cannibalism_inherits_memories() {
        // Arrange
        let mut app = App::new();
        // Setup deceased pop's corpse with memories
        let memory = Memory { desc: "Saved by Hero".to_string(), value: 50.0 };
        let corpse = app.world_mut().spawn((
            Corpse { origin_pop: Entity::PLACEHOLDER },
            Memories(vec![memory.clone()]),
        )).id();

        let cannibal = app.world_mut().spawn((
            Pop,
            Hunger(100.0), // Starving
            Memories(vec![]),
        )).id();

        // Act
        app.world_mut().send_event(ConsumeCorpseEvent {
            consumer: cannibal,
            corpse: corpse
        });
        app.update();

        // Assert
        let cannibal_memories = app.world().get::<Memories>(cannibal).unwrap();
        assert!(cannibal_memories.0.contains(&memory), "Cannibal must inherit the deceased pop's memories");
    }

    #[test]
    fn test_cannibalism_inherits_relationships() {
        // Arrange
        let mut app = App::new();
        let third_party = app.world_mut().spawn(Pop).id();

        let relationship = Relationship { target: third_party, affinity: 80.0 };
        let corpse = app.world_mut().spawn((
            Corpse { origin_pop: Entity::PLACEHOLDER },
            Relationships(vec![relationship]),
        )).id();

        let cannibal = app.world_mut().spawn((
            Pop,
            Hunger(100.0),
            Relationships(vec![]),
        )).id();

        // Act
        app.world_mut().send_event(ConsumeCorpseEvent {
            consumer: cannibal,
            corpse: corpse
        });
        app.update();

        // Assert
        let cannibal_rels = app.world().get::<Relationships>(cannibal).unwrap();
        assert!(cannibal_rels.0.iter().any(|r| r.target == third_party && r.affinity > 0.0),
            "Cannibal must inherit a fraction of the deceased pop's relationships");
    }

    #[test]
    fn test_cannibalism_satisfies_hunger() {
        // Arrange
        let mut app = App::new();
        let corpse = app.world_mut().spawn(Corpse { origin_pop: Entity::PLACEHOLDER }).id();
        let cannibal = app.world_mut().spawn((
            Pop,
            Hunger(100.0),
        )).id();

        // Act
        app.world_mut().send_event(ConsumeCorpseEvent { consumer: cannibal, corpse });
        app.update();

        // Assert
        let hunger = app.world().get::<Hunger>(cannibal).unwrap().0;
        assert!(hunger < 100.0, "Hunger should be reduced after consuming the corpse");
        assert!(app.world().get_entity(corpse).is_err(), "Corpse should be destroyed");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Corpse {
    pub origin_pop: Entity,
}

#[derive(Event)]
pub struct ConsumeCorpseEvent {
    pub consumer: Entity,
    pub corpse: Entity,
}

pub fn handle_cannibalism_system(
    mut commands: Commands,
    mut events: EventReader<ConsumeCorpseEvent>,
    mut consumer_query: Query<(&mut Hunger, &mut Memories, &mut Relationships)>,
    corpse_query: Query<(&Memories, &Relationships), With<Corpse>>,
) {
    for event in events.read() {
        if let Ok((mut hunger, mut c_memories, mut c_rels)) = consumer_query.get_mut(event.consumer) {
            // Satisfy hunger
            hunger.0 = (hunger.0 - 50.0).max(0.0);

            // Inherit memories and relationships
            if let Ok((corpse_memories, corpse_rels)) = corpse_query.get(event.corpse) {
                // Simplified copy of memories
                c_memories.0.extend(corpse_memories.0.clone());

                // Simplified copy of relationships (scaled down affinity)
                for rel in corpse_rels.0.iter() {
                    let inherited_rel = Relationship {
                        target: rel.target,
                        affinity: rel.affinity * 0.5, // Fraction of affinity
                    };
                    c_rels.0.push(inherited_rel);
                }
            }

            // Cleanup corpse
            commands.entity(event.corpse).despawn();
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Make sure that relationships do not duplicate if the cannibal already knows the third party. Instead, merge the affinities (e.g., average them or take the stronger one).
- Ensure the `Memories` struct has a clean way to deduplicate identical memories or append a "Cannibalized Memory" tag so it generates unique narrative events.
- Trigger a massive "Grief/Horror" mood debuff upon eating someone to represent the moral line crossed.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >=85% for new code
- [ ] Cannibalism reduces hunger and destroys the corpse.
- [ ] The consumer inherits memories from the corpse.
- [ ] The consumer inherits fractionally scaled relationships from the corpse.

## 7. Technical Guidance
- Integration with the Utility AI is required. The `ActionType::Eat` needs a fallback to target a `Corpse` if standard food stores are completely empty and `Hunger` is critically high.

## 8. Questions
*Builder: add questions here if spec is unclear.*

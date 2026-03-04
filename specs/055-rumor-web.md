# 055: The Rumor Web

## Overview

The Rumor Web makes the colony feel alive by allowing pops to share information (Rumors) when they socialize. Rumors can be true (news about events) or false (gossip/panic). Rumors spread through the population like a virus, affecting morale and relationships.

When pops interact (Socialize action), they have a chance to exchange a "Rumor".

## Dependencies

- `047` — Pop Relationships (Affinity, `AffinityChange`)
- `028` — Social Need and Tavern (Socialize Action)
- `046` — Notifications System (for rumor notifications)
- `031` — Pop Morale (Morale affects rumor generation)

## RED Phase: Tests First

Write these tests in `src/layer1/rumor_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::pop::Pop;
    use crate::layer1::social::{Relationships, AffinityChange};
    use crate::layer1::rumor::{Knowledge, Rumor, RumorTopic, exchange_rumors_system, generate_rumor_system};
    use crate::layer1::needs::Needs;

    #[test]
    fn test_knowledge_default() {
        let knowledge = Knowledge::default();
        assert!(knowledge.known_rumors.is_empty());
    }

    #[test]
    fn test_rumor_struct() {
        let rumor = Rumor {
            topic: RumorTopic::ResourceShortage("Food".to_string()),
            source: Entity::PLACEHOLDER,
            timestamp: 100,
            strength: 1.0,
        };
        // Just verify it constructs
        assert_eq!(rumor.strength, 1.0);
    }

    #[test]
    fn test_rumor_generation_negative_morale() {
        let mut world = World::new();
        // Pop with very low morale
        let pop = world.spawn((
            Pop,
            Needs { morale: 0.1, ..Default::default() },
            Knowledge::default(),
        )).id();

        // Run generation system
        let mut schedule = Schedule::default();
        schedule.add_systems(generate_rumor_system);
        schedule.run(&mut world);

        // Check if rumor was generated
        let knowledge = world.get::<Knowledge>(pop).unwrap();
        assert!(!knowledge.known_rumors.is_empty());
        // Verify it's likely a negative rumor (e.g. Doom/Shortage)
        let rumor = &knowledge.known_rumors[0];
        match &rumor.topic {
            RumorTopic::ResourceShortage(_) | RumorTopic::DoomProphecy => {}, // Pass
            _ => panic!("Expected negative rumor from low morale"),
        }
    }

    #[test]
    fn test_rumor_exchange_shares_info() {
        let mut world = World::new();

        let rumor = Rumor {
            topic: RumorTopic::EventNews("Colony Founded".to_string()),
            source: Entity::PLACEHOLDER,
            timestamp: 1,
            strength: 1.0,
        };

        // Pop A knows the rumor
        let pop_a = world.spawn((
            Pop,
            Knowledge { known_rumors: vec![rumor.clone()] },
            crate::layer1::social::Socializing, // Tag component for active socializing
        )).id();

        // Pop B knows nothing
        let pop_b = world.spawn((
            Pop,
            Knowledge::default(),
            crate::layer1::social::Socializing,
        )).id();

        // Force interaction (normally system picks random neighbor)
        // For unit test, we might need to mock the interaction or just call a helper function
        // Let's assume the system iterates over pairs.
        // Or we can invoke `share_rumor(pop_a, pop_b)` helper directly if system is too random.

        // Simulating the effect of the system:
        crate::layer1::rumor::share_rumor(&mut world, pop_a, pop_b);

        let knowledge_b = world.get::<Knowledge>(pop_b).unwrap();
        assert_eq!(knowledge_b.known_rumors.len(), 1);
        assert_eq!(knowledge_b.known_rumors[0].topic, rumor.topic);
    }

    #[test]
    fn test_rumor_gossip_affects_affinity() {
        let mut world = World::new();

        let target_pop = world.spawn(Pop).id();
        let listener_pop = world.spawn((
            Pop,
            Knowledge::default(),
            Relationships::default(),
        )).id();
        let speaker_pop = world.spawn(Pop).id();

        let rumor = Rumor {
            topic: RumorTopic::CharacterGossip(target_pop, -10.0), // Nasty gossip
            source: speaker_pop,
            timestamp: 1,
            strength: 1.0,
        };

        // Speaker tells Listener about Target
        // We expect AffinityChange event
        let mut events = world.resource_mut::<Events<AffinityChange>>();

        // Use helper to trigger logic
        crate::layer1::rumor::process_rumor_reaction(&mut world, listener_pop, &rumor);

        // Run system that processes events (from 047)
        // But here we just want to verify the event was sent.
        let events = world.resource::<Events<AffinityChange>>();
        let mut reader = events.get_reader();
        let emitted: Vec<_> = reader.read(events).collect();

        assert_eq!(emitted.len(), 1);
        assert_eq!(emitted[0].target, target_pop);
        assert!(emitted[0].amount < 0.0); // Affinity dropped
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Create `src/layer1/rumor.rs`

```rust
use bevy_ecs::prelude::*;
use crate::layer1::social::{Relationships, AffinityChange};
use crate::layer1::needs::Needs;

#[derive(Clone, Debug, PartialEq)]
pub enum RumorTopic {
    ResourceShortage(String), // "Food"
    CharacterGossip(Entity, f32), // Target, AffinityModifier
    EventNews(String), // "Ship Arrived"
    DoomProphecy,
}

#[derive(Clone, Debug)]
pub struct Rumor {
    pub topic: RumorTopic,
    pub source: Entity, // Original source
    pub timestamp: u32,
    pub strength: f32, // 0.0 to 1.0, decays over time
}

#[derive(Component, Default)]
pub struct Knowledge {
    pub known_rumors: Vec<Rumor>,
}

impl Knowledge {
    pub fn knows(&self, topic: &RumorTopic) -> bool {
        self.known_rumors.iter().any(|r| r.topic == *topic)
    }

    pub fn add_rumor(&mut self, rumor: Rumor) {
        if !self.knows(&rumor.topic) {
            self.known_rumors.push(rumor);
        }
    }
}

pub fn generate_rumor_system(
    mut query: Query<(Entity, &Needs, &mut Knowledge)>,
    time: Res<crate::simulation::SimulationTime>,
) {
    // Random chance to generate rumor based on mood
    // For MVP/Test: Deterministic check
    for (entity, needs, mut knowledge) in query.iter_mut() {
        if needs.morale < 0.2 {
            // Angry/Depressed pop invents a rumor
            // In real game, use RNG. Here, just do it.
            let rumor = Rumor {
                topic: RumorTopic::DoomProphecy,
                source: entity,
                timestamp: time.tick,
                strength: 1.0,
            };
            knowledge.add_rumor(rumor);
        }
    }
}

// Helper to share rumor
pub fn share_rumor(world: &mut World, speaker: Entity, listener: Entity) {
    // Extract rumor from speaker (requires double borrow workarounds in systems)
    // For this helper, we assume we can copy it out first.

    // Simplification for MVP: We need to clone the rumor to pass it.
    let rumor_to_share = if let Some(k) = world.get::<Knowledge>(speaker) {
        k.known_rumors.first().cloned() // Just share the first one for now
    } else {
        None
    };

    if let Some(rumor) = rumor_to_share {
        if let Some(mut listener_knowledge) = world.get_mut::<Knowledge>(listener) {
            if !listener_knowledge.knows(&rumor.topic) {
                listener_knowledge.add_rumor(rumor.clone());

                // Trigger Reaction
                process_rumor_reaction(world, listener, &rumor);
            }
        }
    }
}

pub fn process_rumor_reaction(world: &mut World, listener: Entity, rumor: &Rumor) {
    match &rumor.topic {
        RumorTopic::CharacterGossip(target, amount) => {
            world.send_event(AffinityChange {
                source: listener,
                target: *target,
                amount: *amount,
            });
        },
        RumorTopic::ResourceShortage(_) => {
            // Could lower morale directly here
             if let Some(mut needs) = world.get_mut::<Needs>(listener) {
                 needs.morale = (needs.morale - 0.05).max(0.0);
             }
        },
        _ => {}
    }
}

pub fn exchange_rumors_system(
    mut world: &mut World,
    // Note: Querying entities in `Socializing` state requires that state to exist in 028
    // If not, we iterate all pops and check proximity (expensive) or leverage 028's SocialBuff?
    // Let's assume we iterate all pops with Knowledge.
) {
    // This requires complex borrow checking to mutate two components.
    // Use `world.resource_scope` or `unsafe` (avoid) or just separate Read/Write phases.

    // Phase 1: Identify pairs to swap
    let mut interactions = Vec::new();
    // ... logic to find adjacent pops ...

    // Phase 2: Execute swaps via helper
    for (a, b) in interactions {
        share_rumor(world, a, b);
        share_rumor(world, b, a);
    }
}
```

### 2. Integration with `Socialize` Action

Ideally, this runs when pops are at the Tavern.
We can add a tag component `Socializing` to pops performing that action in `028`.
For now, `exchange_rumors_system` can just run occasionally (every 100 ticks) on all pops to simulate gossip "through the grapevine".

## REFACTOR Phase: Quality & Design

- **Decay**: Rumors should have a `strength` that decreases. Pops forget weak rumors.
- **Truth**: Rumors should have a `truth` value. Investigating a rumor (checking the stockpile) reveals the truth.
- **UI**: Add a "Rumors" tab to the Inspector to see what a pop believes.
- **Optimization**: Don't copy strings in `RumorTopic`; use `Arc<String>` or Interned strings.

## Acceptance Criteria

- [ ] `Rumor` struct and `Knowledge` component exist.
- [ ] Pops generate rumors when morale is low.
- [ ] Pops exchange rumors when interacting (or simulated via test helper).
- [ ] Gossip about a person changes Affinity (`AffinityChange` event).
- [ ] Tests pass.

## Technical Guidance

- Use `AffinityChange` event from `src/layer1/social.rs`.
- Be careful with `World` borrows in `exchange_rumors_system`. It's often better to collect interactions in one pass (Read) and apply them in another (Write).

## Questions

*Builder: add questions here if spec is unclear.*
*Architect:* See related specifications for design details. MVP implementation should follow standard conventions.

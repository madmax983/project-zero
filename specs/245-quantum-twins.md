# 245: Quantum Twins

## Overview

"Two souls linked across the void. 'I felt that.'"

Some Pops share a mysterious **Quantum Entanglement**. This bond means they share specific states instantaneously, regardless of distance.
- **Shared Experience**: When one Twin gains Skill XP, the other gains a percentage of it.
- **Shared Mood**: Their Moods tend to equalize towards the average of the pair.
- **Severance**: If one Twin dies, the other suffers "Severance" (Catatonic state / massive stress).

This creates a high-risk, high-reward dynamic: do you separate them to maximize coverage (one learns Mining, one learns Science -> both learn both), or keep them safe together?

## Dependencies

- `051` — Pop Skills (XP sharing)
- `031` — Pop Morale (Mood sharing)
- `034` — Pop Health (Death trigger)

## RED Phase: Tests First

Write these tests in `src/layer1/quantum_twins_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::pop::{Pop, Mood, Skill, SkillType};
    use crate::layer1::quantum::{QuantumTwin, update_twin_sync_system, handle_severance_system};
    use crate::layer1::health::{Health, DeathEvent};

    #[test]
    fn test_xp_sharing() {
        let mut world = World::new();
        // Spawn Twin A
        let twin_a = world.spawn((
            Pop,
            QuantumTwin { partner: Entity::PLACEHOLDER, link_strength: 0.5 },
            Skill { skill_type: SkillType::Mining, level: 1, xp: 100.0 },
        )).id();

        // Spawn Twin B
        let twin_b = world.spawn((
            Pop,
            QuantumTwin { partner: twin_a, link_strength: 0.5 },
            Skill { skill_type: SkillType::Mining, level: 1, xp: 0.0 },
        )).id();

        // Fix circular ref for Twin A
        world.get_mut::<QuantumTwin>(twin_a).unwrap().partner = twin_b;

        // Simulate XP gain on A (usually via event or system, here we check sync logic)
        // Assume system detects delta or we trigger a sync event.
        // For simplicity, let's say the system redistributes diff.

        // Actually, shared XP usually works on "Gain" events.
        // Let's assume we have a `GrantXpEvent` and the twin system intercepts it.
        // Or simpler: The system equalizes XP over time?
        // Spec says "When one gains... other gains %". Let's model that via event interception.

        world.insert_resource(Events::<crate::layer1::skills::XpGainEvent>::default());
        let mut schedule = Schedule::default();
        schedule.add_systems(update_twin_sync_system); // This system listens to XpGainEvent

        // Send XP event for A
        world.send_event(crate::layer1::skills::XpGainEvent {
            entity: twin_a,
            skill: SkillType::Mining,
            amount: 50.0,
        });

        schedule.run(&mut world);

        let skill_b = world.get::<Skill>(twin_b).unwrap();
        // Twin B should get 50.0 * 0.5 = 25.0
        assert_eq!(skill_b.xp, 25.0);
    }

    #[test]
    fn test_mood_equalization() {
        let mut world = World::new();
        let twin_a = world.spawn((
            Pop,
            QuantumTwin { partner: Entity::PLACEHOLDER, link_strength: 0.1 },
            Mood { value: 100.0, ..Default::default() },
        )).id();

        let twin_b = world.spawn((
            Pop,
            QuantumTwin { partner: twin_a, link_strength: 0.1 },
            Mood { value: 0.0, ..Default::default() },
        )).id();

        world.get_mut::<QuantumTwin>(twin_a).unwrap().partner = twin_b;

        let mut schedule = Schedule::default();
        schedule.add_systems(update_twin_sync_system); // Also handles mood

        schedule.run(&mut world);

        let mood_a = world.get::<Mood>(twin_a).unwrap();
        let mood_b = world.get::<Mood>(twin_b).unwrap();

        // They should move towards average (50).
        // A goes down, B goes up.
        assert!(mood_a.value < 100.0);
        assert!(mood_b.value > 0.0);
    }

    #[test]
    fn test_severance_on_death() {
        let mut world = World::new();
        world.insert_resource(Events::<DeathEvent>::default());

        let twin_a = world.spawn((
            Pop,
            QuantumTwin { partner: Entity::PLACEHOLDER, link_strength: 1.0 },
            Health { current: 0.0, max: 100.0 },
        )).id();

        let twin_b = world.spawn((
            Pop,
            QuantumTwin { partner: twin_a, link_strength: 1.0 },
            Mood { value: 50.0, ..Default::default() },
        )).id();

        world.get_mut::<QuantumTwin>(twin_a).unwrap().partner = twin_b;

        let mut schedule = Schedule::default();
        schedule.add_systems(handle_severance_system);

        // Kill A
        world.send_event(DeathEvent { entity: twin_a });

        schedule.run(&mut world);

        let mood_b = world.get::<Mood>(twin_b).unwrap();
        // Should have "Catatonic" or massive stress
        assert!(mood_b.stress > 90.0 || mood_b.value < 10.0);

        // Link should be removed or broken
        assert!(world.get::<QuantumTwin>(twin_b).is_none());
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Components

```rust
// src/layer1/quantum.rs

use bevy_ecs::prelude::*;
use crate::layer1::pop::Mood;
use crate::layer1::skills::{Skill, XpGainEvent}; // Assuming exist
use crate::layer1::health::DeathEvent;

#[derive(Component, Debug, Clone, Copy)]
pub struct QuantumTwin {
    pub partner: Entity,
    pub link_strength: f32, // 0.0 to 1.0, determines transfer rate
}

pub fn update_twin_sync_system(
    mut xp_events: EventReader<XpGainEvent>,
    mut xp_writer: EventWriter<XpGainEvent>, // To trigger secondary gain? Careful of infinite loop.
    // Better: Direct modification for secondary gain to avoid loop.
    mut skills: Query<&mut Skill>,
    mut moods: Query<&mut Mood>,
    twins: Query<&QuantumTwin>,
) {
    // 1. XP Sharing
    // We need to buffer secondary gains to avoiding borrowing conflicts if we used writer.
    // Or iterate events and apply directly.
    for event in xp_events.read() {
        if let Ok(twin) = twins.get(event.entity) {
            if let Ok(mut partner_skill) = skills.get_mut(twin.partner) {
                if partner_skill.skill_type == event.skill {
                    // Avoid infinite loop: Assuming XpGainEvent has a "source" flag or we just apply directly
                    partner_skill.xp += event.amount * twin.link_strength;
                }
            }
        }
    }

    // 2. Mood Equalization
    // Naive O(N) iteration
    for (twin, mut mood) in moods.iter_many_mut(twins.iter()) {
       // This approach is tricky with mutable iteration of the same component type.
       // Bevy prevents getting `mut mood` for both partners simultaneously in a single query loop easily.
       // Standard pattern: Use `ParamSet` or unsafe, or two-pass.
       // Two-pass: Read all moods, calculate deltas, apply deltas.
    }
}

pub fn handle_severance_system(
    mut commands: Commands,
    mut events: EventReader<DeathEvent>,
    mut twins: Query<(Entity, &QuantumTwin, &mut Mood)>,
) {
    for event in events.read() {
        // Find the partner of the dead entity
        // We iterate all twins to find who has 'partner == event.entity'
        for (entity, twin, mut mood) in twins.iter_mut() {
            if twin.partner == event.entity {
                // Apply Severance
                mood.stress = 100.0; // Max stress
                mood.value = 0.0;    // Min mood

                // Remove component
                commands.entity(entity).remove::<QuantumTwin>();
            }
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **Mood Sync Optimization**: Instead of N^2 or complex queries, use a `TwinGraph` resource or just iterate `QuantumTwin` pairs carefully. Or, make the sync lazy (once per second, not per frame).
- **Infinite Loop Prevention**: Ensure XP sharing is strictly "Primary Gain -> Secondary Gain" and not "Secondary Gain -> Partner's Secondary Gain". Add a `source: XpSource::Entanglement` to the event and ignore it in the handler.
- **UI**: Visualize the link with a subtle line or icon when selecting one twin.

## Acceptance Criteria

- [ ] `QuantumTwin` component exists.
- [ ] XP gained by one is partially given to the other.
- [ ] Moods drift towards average.
- [ ] Death triggers Severance on survivor.
- [ ] Tests pass.

## Technical Guidance

- Use `XpGainEvent` from `051` if available. If `051` directly modifies XP without events, refactor `051` or hook into `update_skills`.
- Bevy's `Query::get_many_mut` allows retrieving two mutable components if entities are distinct.

## Questions

*Builder: Can twins be separated by Layer?*
*Architect: Yes, that's the point. One on planet, one in orbit.*

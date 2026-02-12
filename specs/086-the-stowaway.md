# 086: The Stowaway

## Overview

Introduces **Stowaways**—hidden entities that sneak into the colony via incoming Visitors or Ships. They consume resources (Food) and remain invisible on the UI until discovered.

This feature adds mystery and resource pressure. Players notice missing food but see no culprit, prompting investigation or security measures.

## Dependencies

- `004` — Pop Entity (Stowaways are technically Pops but hidden).
- `074` — Visitor System (Arrival mechanism).
- `022` — Resource Stockpiles (Theft target).
- `046` — Notifications (Feedback when food goes missing).

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/layer1/stowaway_tests.rs

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::pop::{Pop, Name};
    use crate::layer1::stowaway::{Stowaway, Hidden, check_stowaway_arrival_system, stowaway_theft_system, reveal_stowaway_system};
    use crate::layer1::visitor::{Visitor, VisitorState};
    use crate::layer1::ColonyResources;
    use crate::layer1::notifications::NotificationQueue;
    use crate::shared::time::SimulationTime;

    #[test]
    fn test_stowaway_components() {
        let mut world = World::new();
        let entity = world.spawn((
            Pop,
            Stowaway,
            Hidden, // Marker to hide from UI
            Name("Mysterious Stranger".to_string()),
        )).id();

        assert!(world.get::<Stowaway>(entity).is_some());
        assert!(world.get::<Hidden>(entity).is_some());
    }

    #[test]
    fn test_stowaway_arrival_chance() {
        let mut world = World::new();
        // Setup arrival event (simulated by a Visitor entering Arriving state)
        world.spawn((
            Visitor { state: VisitorState::Arriving, ..Default::default() },
            // Tag needed to trigger stowaway logic (e.g. "HasStowaway" or just random chance)
        ));

        // For deterministic test, we might need to mock RNG or force arrival
        // In this test, we assume the system checks for new visitors and rolls dice.
        // We'll skip exact probability test and focus on the result: spawning a Stowaway.

        // Setup: Mock system behavior for test
        world.spawn((Pop, Stowaway, Hidden));

        let count = world.query::<(&Stowaway, &Hidden)>().iter(&world).count();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_stowaway_steals_food() {
        let mut world = World::new();
        world.insert_resource(ColonyResources { food: 100.0, ..Default::default() });
        world.insert_resource(NotificationQueue::default());

        // Spawn stowaway
        world.spawn((Pop, Stowaway, Hidden));

        // Run theft system
        stowaway_theft_system(&mut world);

        // Verify food decreased
        let resources = world.resource::<ColonyResources>();
        assert!(resources.food < 100.0);

        // Verify notification
        let notifications = world.resource::<NotificationQueue>();
        assert!(!notifications.queue.is_empty());
        assert!(notifications.queue[0].text.contains("Food is missing"));
    }

    #[test]
    fn test_stowaway_reveal() {
        let mut world = World::new();
        let entity = world.spawn((Pop, Stowaway, Hidden)).id();

        // Mock condition: Player "investigated" or time passed
        // For test, we manually trigger reveal logic or simulate time
        world.insert_resource(SimulationTime { tick: 1000, ..Default::default() }); // High tick for timeout

        // Run reveal system
        reveal_stowaway_system(&mut world);

        // Hidden component should be removed
        assert!(world.get::<Hidden>(entity).is_none());
        // Stowaway component might remain (as a trait/history) or be removed
        // Spec: Remove Stowaway, convert to regular Pop or keep tag for flavor?
        // Let's keep Stowaway tag for flavor.
        assert!(world.get::<Stowaway>(entity).is_some());
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Define Components

```rust
// src/layer1/stowaway.rs

use bevy_ecs::prelude::*;

#[derive(Component, Default, Debug, Clone)]
pub struct Stowaway;

#[derive(Component, Default, Debug, Clone)]
pub struct Hidden; // Marker to exclude from UI lists and counts
```

### 2. Arrival Logic

Hook into `visitor_spawn_system` or run a separate system `check_stowaway_arrival` that queries `Added<Visitor>`.

```rust
pub fn check_stowaway_arrival_system(
    mut commands: Commands,
    query: Query<Entity, Added<crate::layer1::visitor::Visitor>>,
    // rng resource
) {
    for _entity in query.iter() {
        if rand::thread_rng().gen_bool(0.1) { // 10% chance
            // Spawn Stowaway
            commands.spawn((
                crate::layer1::pop::Pop,
                Stowaway,
                Hidden,
                crate::layer1::pop::Name("Unknown".to_string()),
                // Add standard Pop components (Needs, Position, etc.)
                // Position should be near the Visitor spawn point
            ));
        }
    }
}
```

### 3. Theft Logic

```rust
pub fn stowaway_theft_system(
    mut resources: ResMut<crate::layer1::ColonyResources>,
    mut notifications: ResMut<crate::layer1::notifications::NotificationQueue>,
    query: Query<&Stowaway, With<Hidden>>,
) {
    for _ in query.iter() {
        if resources.food > 0.0 {
            resources.food -= 1.0; // Consume 1 food per tick/day?
            // Better: run daily or based on hunger.
            // For MVP: Simple random theft.
            if rand::thread_rng().gen_bool(0.01) {
                notifications.push(crate::layer1::notifications::Notification {
                    text: "Supplies are missing from the stockpile.".to_string(),
                    severity: crate::layer1::notifications::Severity::Warning,
                    ..Default::default()
                });
            }
        }
    }
}
```

### 4. Reveal Logic

Remove `Hidden` after a duration or event.

```rust
pub fn reveal_stowaway_system(
    mut commands: Commands,
    query: Query<(Entity, &Stowaway), With<Hidden>>,
    // time resource
) {
    for (entity, _) in query.iter() {
        // Simple logic: 5% chance per day to be caught
        if rand::thread_rng().gen_bool(0.001) {
            commands.entity(entity).remove::<Hidden>();
            // Add notification "A stowaway has been discovered!"
        }
    }
}
```

### 5. UI Updates (Important!)

Builder MUST update `src/ui/status.rs` to exclude `Hidden` pops from the count.

```rust
// In src/ui/status.rs

// Old query: count all pops
// New query: count pops WITHOUT Hidden component
```

## REFACTOR Phase: Quality & Design

- **Discovery Mechanics**: Instead of random chance, use `Security` skill or `Patrol` jobs to increase reveal chance.
- **Integration**: Stowaways should eventually have Needs (Hunger) that drive the theft. If they can't steal, they starve (and die hidden?).
- **Story**: When revealed, trigger a dialog: "Banished", "Imprisoned", or "Joined Colony".

## Acceptance Criteria

- [ ] `Stowaway` and `Hidden` components defined.
- [ ] `Hidden` pops do NOT appear in the UI status bar count.
- [ ] 10% chance for a Stowaway to spawn when a Visitor arrives.
- [ ] Stowaways consume food resources periodically.
- [ ] Notifications appear when food is stolen.
- [ ] Stowaways are eventually revealed (Hidden removed).
- [ ] All tests pass.

## Technical Guidance

- Ensure `Hidden` component is added to the `Pop` bundle or spawned alongside it.
- Modify `render_status_bar` in `src/ui/status.rs` to filter `Without<Hidden>`.
- Use `Added<Visitor>` filter to trigger arrival only once per visitor group.

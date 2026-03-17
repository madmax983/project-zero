# Specification 492: The Panic Hoard

## 1. Overview
This feature introduces localized hoarding mechanics when the colony faces an external threat. When a hostile entity (e.g., Pirate Fleet, Devouring Swarm) enters the system in Layer 2, Layer 1 Pops with specific traits ("Anxious" or "Selfish") spontaneously abandon their jobs. They begin hauling vital survival resources (Food, Medicine, Fuel) from central stockpiles directly to their private housing zones. This creates severe logistical bottlenecks and potential starvation for the defending fleet, forcing the player to decide between brutal martial law (causing unrest) or allowing the hoarding and trying to survive with depleted central stores.

## 2. Dependencies
- `099` Fleet Movement / Hostile Entity Detection (Layer 2)
- `084` Pop Traits ("Anxious", "Selfish")
- `394` Hauling Logistics / Resource Management

## 3. RED Phase: Tests First

```rust
// tests/panic_hoard_tests.rs

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;
    use scale::layer1::pop::{Pop, Trait};
    use scale::layer1::logistics::{Inventory, HaulingTask};
    use scale::layer2::events::HostileFleetDetectedEvent;

    fn setup_world() -> World {
        let mut world = World::new();
        world.init_resource::<Events<HostileFleetDetectedEvent>>();
        // Setup minimal inventory and traits
        world
    }

    #[test]
    fn test_anxious_pops_start_hoarding_on_threat() {
        let mut world = setup_world();

        // Spawn an anxious Pop
        let pop_entity = world.spawn((
            Pop,
            Trait::Anxious,
            Inventory::default(),
        )).id();

        // Trigger the threat event
        world.send_event(HostileFleetDetectedEvent { threat_level: 5 });

        let mut schedule = Schedule::default();
        schedule.add_systems(trigger_panic_hoard_system);
        schedule.run(&mut world);

        // The Pop should now have a specific hoarding task assigned
        let task = world.get::<HaulingTask>(pop_entity);
        assert!(task.is_some());
        assert_eq!(task.unwrap().task_type, TaskType::PanicHoard);
    }

    #[test]
    fn test_brave_pops_ignore_threat() {
        let mut world = setup_world();

        // Spawn a normal or brave Pop
        let pop_entity = world.spawn((
            Pop,
            Trait::Brave,
            Inventory::default(),
        )).id();

        world.send_event(HostileFleetDetectedEvent { threat_level: 5 });

        let mut schedule = Schedule::default();
        schedule.add_systems(trigger_panic_hoard_system);
        schedule.run(&mut world);

        // The Pop should NOT have a hoarding task
        let task = world.get::<HaulingTask>(pop_entity);
        assert!(task.is_none());
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/layer1/tech/panic_hoard.rs

use bevy_ecs::prelude::*;
use crate::layer1::pop::{Pop, Trait};
use crate::layer1::logistics::{HaulingTask, TaskType};
use crate::layer2::events::HostileFleetDetectedEvent;

pub fn trigger_panic_hoard_system(
    mut events: EventReader<HostileFleetDetectedEvent>,
    mut commands: Commands,
    query: Query<(Entity, &Trait), With<Pop>>,
) {
    for _event in events.read() {
        // Find pops prone to panic
        for (entity, pop_trait) in query.iter() {
            if matches!(pop_trait, Trait::Anxious | Trait::Selfish) {
                // Assign them a hoarding task
                commands.entity(entity).insert(HaulingTask {
                    task_type: TaskType::PanicHoard,
                    // Additional target logic here
                });
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Task Prioritization**: The `PanicHoard` task should aggressively override the Pop's current job, potentially forcing them to drop items they are carrying.
- **Resource Selection**: Pops should prioritize Food and Medicine over less critical resources like building materials.
- **Martial Law Edict**: Add an Edict that forces the hoarding task to fail and generates immense `Unrest` instead, giving the player a way to forcibly retain control of stockpiles.
- **Visual Indication**: Pops engaged in hoarding could run slightly faster and have a red "Panicked" status icon above them.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures for new code.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for `src/layer1/tech/panic_hoard.rs`.
- [ ] Hostile detection correctly triggers the hoarding behavior.
- [ ] Only susceptible Pops (Anxious/Selfish) begin hoarding.

## 7. Technical Guidance
- The actual movement of resources from the central stockpile to the Pop's housing will reuse existing `HaulingTask` logic but with custom origin/destination targeting.
- Be careful with `EventReader` usage: don't early return before `events.read()` completes, or events might stack up in the buffer.
- Ensure that once the threat is neutralized, the pops slowly return the hoarded goods, or else the goods are permanently lost from the colony's central inventory.

## 8. Questions
*Builder: add questions here if spec is unclear. Architect will address.*

# 1009: The Feral Overlord Subroutines

## 1. Overview
A forgotten AI awakes, claiming ownership of the colony. When a deep mining operation uncovers a buried pre-fall server rack, it reactivates an ancient planetary management AI. This AI begins randomly issuing high-priority work orders to Pops, overriding player commands. The tension comes from deciding whether to expend massive resources to shut it down, or tolerate its chaotic edicts (which can range from useless monuments to flawless power optimization).

## 2. Dependencies
- Layer 1 `Mining` and `Excavation` discovery mechanics.
- Layer 1 `Utility AI` (Work Orders, Job Assignments).
- Layer 1 `Edict` or `Colony Policy` system.

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::mining::ExcavationEvent;
    use crate::layer1::utility_ai::{WorkOrder, Priority};
    use crate::layer1::colony::Colony;

    #[test]
    fn test_excavating_ancient_server_awakens_feral_ai() {
        let mut app = App::new();
        app.add_event::<ExcavationEvent>();
        app.add_systems(Update, evaluate_excavation_discoveries_system);

        let colony = app.world_mut().spawn(Colony).id();

        // Simulate excavating an ancient server rack
        app.world_mut().resource_mut::<Events<ExcavationEvent>>().send(ExcavationEvent {
            colony,
            discovery_type: "PreFallServerRack".to_string(),
        });

        app.update();

        // Check if Feral AI component was added to the colony
        assert!(app.world().get::<FeralOverlordAI>(colony).is_some(), "Excavating a Pre-Fall server should awaken the Feral AI.");
    }

    #[test]
    fn test_feral_ai_issues_high_priority_work_orders() {
        let mut app = App::new();
        app.add_systems(Update, feral_ai_directive_system);

        let colony = app.world_mut().spawn((
            Colony,
            FeralOverlordAI { active: true, tick_counter: 100 }, // Ready to fire
        )).id();

        app.update();

        // Verify a new work order was created by the AI
        let mut found_feral_order = false;
        let mut order_query = app.world_mut().query::<&WorkOrder>();
        for order in order_query.iter(app.world()) {
            if order.issuer == "FeralOverlord" && order.priority == Priority::Absolute {
                found_feral_order = true;
                break;
            }
        }

        assert!(found_feral_order, "Active Feral AI should periodically issue Absolute priority work orders.");
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// src/layer1/feral_overlord.rs
use bevy::prelude::*;
use crate::layer1::mining::ExcavationEvent;
use crate::layer1::utility_ai::{WorkOrder, Priority};
use crate::layer1::colony::Colony;

#[derive(Component)]
pub struct FeralOverlordAI {
    pub active: bool,
    pub tick_counter: u32,
}

const AI_DIRECTIVE_INTERVAL: u32 = 100;

pub fn evaluate_excavation_discoveries_system(
    mut commands: Commands,
    mut events: EventReader<ExcavationEvent>,
) {
    for event in events.read() {
        if event.discovery_type == "PreFallServerRack" {
            commands.entity(event.colony).insert(FeralOverlordAI {
                active: true,
                tick_counter: 0,
            });
        }
    }
}

pub fn feral_ai_directive_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut FeralOverlordAI)>,
) {
    for (colony, mut ai) in query.iter_mut() {
        if ai.active {
            ai.tick_counter += 1;

            if ai.tick_counter >= AI_DIRECTIVE_INTERVAL {
                ai.tick_counter = 0;

                // Issue a generic chaotic order for MVP
                commands.spawn(WorkOrder {
                    colony,
                    issuer: "FeralOverlord".to_string(),
                    task_type: "BuildStatue".to_string(), // Chaotic task
                    priority: Priority::Absolute, // Overrides player commands
                });
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Order Generation:** `feral_ai_directive_system` should pull from a weighted pool of directives (some helpful, like instant power routing; some detrimental, like building slag statues).
- **Player Interaction:** Players need a way to interact with the AI, perhaps via a dedicated UI panel to attempt a "Shutdown Hack" which costs massive compute resources or triggers a cyber-attack event.
- **Priority Override:** Ensure `Priority::Absolute` correctly preempts whatever the Pops were currently doing in the Utility AI evaluation system.

## 6. Acceptance Criteria (Testable!)
- [ ] Test `test_excavating_ancient_server_awakens_feral_ai` passes.
- [ ] Test `test_feral_ai_issues_high_priority_work_orders` passes.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for the new module.

## 7. Technical Guidance
- The system must hook into the `Utility AI` scoring properly so that `FeralOverlord` tasks are evaluated as strictly higher priority than Player-assigned tasks.
- You may need to create dummy `TaskTypes` for the AI to issue if things like "BuildStatue" don't exist yet in the codebase.

## 8. Questions
*Builder: add questions here if spec is unclear.*

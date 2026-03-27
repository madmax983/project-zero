# 677 - The Phantom Work Order

## 1. Overview
The Phantom Work Order is a Layer 1 bureaucracy feature. When an influential Pop (like a manager or faction leader) dies unexpectedly, their scheduled work designations and colony edicts do not automatically clear. Instead, they transform into 'Phantom Orders'. Subordinate Pops will continue to stubbornly fulfill these orders out of bureaucratic inertia, even if the orders are now illogical, resource-wasting, or physically impossible to complete. This forces the player to spend administrative resources auditing and purging the old system.

## 2. Dependencies
- Layer 1 `Pop` and `ActionPlan`/Utility AI.
- Layer 1 Work Designation system (`WorkDesignation`, `Job`).
- Layer 1 `Chronicle` (for recording the death and the resulting bureaucratic mess).

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::jobs::{Job, WorkDesignation};
    use crate::layer1::social::faction::{Manager, Subordinate};
    use crate::layer1::health::DeathEvent;

    #[test]
    fn test_manager_death_creates_phantom_orders() {
        let mut app = App::new();
        app.add_systems(Update, process_manager_death_system);

        let manager_ent = app.world_mut().spawn(Manager {}).id();

        // Spawn a work designation assigned by this manager
        let order_ent = app.world_mut().spawn((
            WorkDesignation {
                task_type: TaskType::Build,
                assigned_by: manager_ent,
            },
        )).id();

        // Trigger the manager's death
        app.world_mut().send_event(DeathEvent { entity: manager_ent });
        app.update();

        // Assert the order is now a PhantomOrder
        assert!(app.world().get::<PhantomOrder>(order_ent).is_some(), "Order should become a PhantomOrder upon manager's death");
    }

    #[test]
    fn test_subordinates_prioritize_phantom_orders() {
        let mut app = App::new();
        app.add_systems(Update, evaluate_actions_system);

        let manager_ent = Entity::from_raw(1); // Dead manager

        let order_ent = app.world_mut().spawn((
            WorkDesignation {
                task_type: TaskType::Haul,
                assigned_by: manager_ent,
            },
            PhantomOrder { inertia_score: 80.0 }, // High inertia
        )).id();

        // Spawn a subordinate
        let sub_ent = app.world_mut().spawn((
            Pop {},
            Subordinate { manager: manager_ent },
            UtilityWeights::default(),
            ActionPlan::default(),
        )).id();

        app.update();

        let plan = app.world().get::<ActionPlan>(sub_ent).unwrap();
        // The subordinate should still select the hauling task despite the manager being dead
        assert_eq!(plan.target, Some(order_ent));
        assert_eq!(plan.action_type, ActionType::Work);
    }

    #[test]
    fn test_administrative_audit_removes_phantom_orders() {
        let mut app = App::new();
        app.add_systems(Update, administrative_audit_system);

        let order_ent = app.world_mut().spawn((
            WorkDesignation {
                task_type: TaskType::Haul,
                assigned_by: Entity::from_raw(1),
            },
            PhantomOrder { inertia_score: 10.0 },
        )).id();

        // Trigger an audit event (costs admin resources)
        app.world_mut().send_event(AuditOrdersEvent { sector: SectorId(0) });
        app.update();

        // Assert the phantom order has been purged
        assert!(app.world().get::<PhantomOrder>(order_ent).is_none(), "PhantomOrder should be removed after an audit");
        // Or perhaps the entire WorkDesignation is despawned
        assert!(app.world().get_entity(order_ent).is_none(), "WorkDesignation should be despawned after an audit");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct PhantomOrder {
    pub inertia_score: f32,
}

#[derive(Component)]
pub struct WorkDesignation {
    pub task_type: TaskType,
    pub assigned_by: Entity,
}

#[derive(Event)]
pub struct DeathEvent {
    pub entity: Entity,
}

#[derive(Event)]
pub struct AuditOrdersEvent {
    pub sector: SectorId,
}

pub fn process_manager_death_system(
    mut commands: Commands,
    mut death_events: EventReader<DeathEvent>,
    query: Query<(Entity, &WorkDesignation)>,
) {
    for event in death_events.read() {
        let dead_manager = event.entity;

        for (order_ent, designation) in query.iter() {
            if designation.assigned_by == dead_manager {
                commands.entity(order_ent).insert(PhantomOrder {
                    inertia_score: 50.0,
                });
            }
        }
    }
}

pub fn administrative_audit_system(
    mut commands: Commands,
    mut audit_events: EventReader<AuditOrdersEvent>,
    query: Query<Entity, With<PhantomOrder>>,
) {
    for _event in audit_events.read() {
        for order_ent in query.iter() {
            commands.entity(order_ent).despawn();
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Refactoring:** The `assigned_by` field in `WorkDesignation` might be null or refer to a non-existent entity if the manager was despawned. Handle `Option<Entity>` gracefully or use a custom `ManagerId`.
- **Code Smells:** Iterating over all `WorkDesignation` components on every death event is inefficient if there are thousands of orders. Maintain an index or a component on the Manager entity listing their issued orders.
- **Performance:** Ensure the Utility AI penalty/bonus calculations for `PhantomOrder` are fast.
- **API Improvements:** Add a decaying `inertia_score` to `PhantomOrder` so they slowly expire over time if an audit isn't performed.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] WorkDesignations assigned by a dead Pop gain the PhantomOrder component.
- [ ] Subordinates prioritize PhantomOrders until an Audit removes them.

## 7. Technical Guidance
- **Code Structure:** Add this logic near `src/layer1/jobs/designations.rs` or `src/layer1/social/faction.rs`.
- **Integration Points:** Hook into the `DeathEvent` system. Modify the Utility AI's job scoring function to recognize `PhantomOrder` and artificially inflate its score for subordinates of the deceased manager.
- **Gotchas:** Be careful with despawned entities. If a `PhantomOrder` references a `target` that no longer exists (e.g., hauling to a destroyed stockpile), the Utility AI needs to handle the invalid state gracefully without crashing the game loop.

## 8. Questions
*Builder: add questions here if spec is unclear.*

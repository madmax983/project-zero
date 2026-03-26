# 615: Bureaucratic Drag

## 1. Overview
The suffocating weight of red tape. The larger the colony grows, the slower it moves. Administrative "Admin" power becomes a resource consumed by buildings and pops. Low Admin causes delayed orders, ignored designations, or "lost" resource counts. Administrative buildings (Offices) and jobs (Clerks) generate Admin but produce nothing tangible. This introduces a tension between productive jobs (Food/Ore) and non-productive jobs required to keep the lights on and the colony stable.

## 2. Dependencies
- Layer 1 Economy System
- Layer 1 Jobs & Employment
- Layer 1 Build/Designation Tasks

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use layer1::economy::{AdminPower, AdminDemand};
    use layer1::jobs::{JobAssignment, Clerk};
    use layer1::tasks::{TaskQueue, TaskDelay};

    #[test]
    fn test_admin_demand_scales_with_population() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, calculate_admin_demand_system);

        let pop_entity1 = app.world_mut().spawn(AdminDemand(5)).id();
        let pop_entity2 = app.world_mut().spawn(AdminDemand(5)).id();
        let global_admin = app.world_mut().spawn(GlobalAdmin::default()).id();

        app.update();

        let demand = app.world().get::<GlobalAdmin>(global_admin).unwrap().demand;
        assert_eq!(demand, 10, "Global admin demand should be the sum of individual demands");
    }

    #[test]
    fn test_low_admin_power_causes_task_delays() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, apply_bureaucratic_drag_system);

        // Setup deficit: power=5, demand=20
        let global_admin = app.world_mut().spawn(GlobalAdmin { power: 5, demand: 20 }).id();
        let task_entity = app.world_mut().spawn((TaskQueue { name: "Build Wall".into() }, TaskDelay(0))).id();

        app.update();

        let delay = app.world().get::<TaskDelay>(task_entity).unwrap().0;
        assert!(delay > 0, "Tasks should be delayed when admin power is less than demand");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
#[derive(Component, Default)]
pub struct GlobalAdmin {
    pub power: u32,
    pub demand: u32,
}

#[derive(Component)]
pub struct AdminDemand(pub u32);

#[derive(Component)]
pub struct TaskDelay(pub u32);

pub fn calculate_admin_demand_system(
    mut admin_query: Query<&mut GlobalAdmin>,
    demand_query: Query<&AdminDemand>,
) {
    let Ok(mut global) = admin_query.get_single_mut() else { return };
    global.demand = demand_query.iter().map(|d| d.0).sum();
}

pub fn apply_bureaucratic_drag_system(
    admin_query: Query<&GlobalAdmin>,
    mut task_query: Query<&mut TaskDelay>,
) {
    let Ok(global) = admin_query.get_single() else { return };

    // Only apply drag if demand exceeds power
    if global.demand > global.power {
        let deficit = global.demand - global.power;
        for mut delay in task_query.iter_mut() {
            // Delay scales with deficit
            delay.0 += deficit / 2;
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Calculate admin power generation incrementally via clerks and offices rather than manually setting it in `GlobalAdmin`.
- Limit maximum `TaskDelay` to prevent tasks from permanently stalling into unrecoverable states.
- Consider utilizing Bevy `Resource` for `GlobalAdmin` instead of a singleton entity to make system queries simpler.
- Extract scaling factors to a configuration file/resource.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Admin demand scales accurately with population and active buildings.
- [ ] Admin deficit successfully introduces progressive delays into the task execution queue.

## 7. Technical Guidance
- **ECS Pattern:** Make `GlobalAdmin` a `Resource`. It aggregates the global sum of demand from pops and generation from clerks.
- **Task Delay:** Add a `TaskDelay` component to new task entities. Worker systems should check this delay value and decrement it over time before processing the actual task.
- **UI:** Expose the Admin Deficit/Surplus clearly on the main UI, highlighting tasks that are delayed due to "Red Tape".

## 8. Questions
*Builder: add questions here if spec is unclear.*

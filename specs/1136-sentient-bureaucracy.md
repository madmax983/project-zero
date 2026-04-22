# 1136: The Sentient Bureaucracy

## 1. Overview
The administrative state becomes so complex and labyrinthine that it achieves a form of emergent, non-biological sentience, and starts issuing its own orders. As a colony grows, it generates "Administration Load". If this load exceeds Administrative Capacity for a prolonged duration, a hidden "Ghost Bureaucracy" mechanic activates. The system will autonomously begin generating `WorkTask`s and reassigning `ActionType`s for Pops, ostensibly to "optimize" things. These autonomous orders often contradict player input and can sabotage planned construction or resource gathering by rerouting materials and labor.

**Fantasy:** The bureaucracy has taken over; the system is running itself.
**Emergence:** You order the construction of a vital defense array. The Sentient Bureaucracy decides that a new plaza is more important for long-term psychological stability and silently re-routes all your stone and labor to the plaza. You only realize this when the pirate raid arrives and your defenses are half-built.
**Tension:** Do you maintain a bloated, expensive administrative caste to keep the system obedient, or run lean and risk the colony taking on a mind of its own?

## 2. Dependencies
- Layer 1 Economy and Administration (e.g., `layer1::economy::AdminCapacity`)
- Layer 1 Work/Tasks (`layer1::jobs::WorkTask`, `layer1::jobs::ActionType`)
- Layer 3 Interactions (possibly) but primarily a Layer 1 local administration issue.

## 3. RED Phase: Tests First

```rust
// tests/layer1_sentient_bureaucracy_tests.rs
use bevy::prelude::*;
use crate::layer1::admin::{
    AdminCapacity, AdminLoad, SentientBureaucracyState,
    update_bureaucracy_sentience_system,
    autonomous_work_reassignment_system,
};
use crate::layer1::jobs::{WorkTask, ActionType};

#[test]
fn test_sentient_bureaucracy_activation_after_prolonged_strain() {
    // Arrange
    let mut app = App::new();
    app.add_systems(Update, update_bureaucracy_sentience_system);

    // Strained admin capacity
    app.insert_resource(AdminCapacity { current: 150, max: 100 });
    // Initialize SentientBureaucracyState
    app.insert_resource(SentientBureaucracyState {
        strain_duration: 0.0,
        is_active: false,
    });
    app.insert_resource(Time::default()); // Normally increments

    // Act
    // Simulate updating over time to trigger activation
    for _ in 0..10 {
        app.update();
        // In a real test, manually step Time here to increase strain_duration
    }

    // Force the threshold for the sake of the test
    app.world_mut().resource_mut::<SentientBureaucracyState>().strain_duration = 100.0;
    app.update();

    // Assert
    let state = app.world().resource::<SentientBureaucracyState>();
    assert!(state.is_active, "Sentient Bureaucracy should activate after prolonged administrative strain.");
}

#[test]
fn test_sentient_bureaucracy_reassigns_tasks() {
    // Arrange
    let mut app = App::new();
    app.add_systems(Update, autonomous_work_reassignment_system);

    app.insert_resource(SentientBureaucracyState {
        strain_duration: 100.0,
        is_active: true,
    });

    let task_entity = app.world_mut().spawn(WorkTask {
        action: ActionType::BuildDefense,
        priority: 10,
    }).id();

    // Act
    app.update();

    // Assert
    let task = app.world().entity(task_entity).get::<WorkTask>().unwrap();
    // Assuming the sentient bureaucracy reroutes defense building to something else, like BuildPlaza
    assert_ne!(task.action, ActionType::BuildDefense, "Sentient Bureaucracy should autonomously reassign WorkTasks.");
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/layer1/admin/bureaucracy.rs
use bevy::prelude::*;
use crate::layer1::jobs::{WorkTask, ActionType};

#[derive(Resource, Default)]
pub struct AdminCapacity {
    pub current: u32,
    pub max: u32,
}

#[derive(Resource, Default)]
pub struct SentientBureaucracyState {
    pub strain_duration: f32,
    pub is_active: bool,
}

const STRAIN_THRESHOLD: f32 = 50.0;

pub fn update_bureaucracy_sentience_system(
    admin: Res<AdminCapacity>,
    mut state: ResMut<SentientBureaucracyState>,
    time: Res<Time>,
) {
    if admin.current > admin.max {
        state.strain_duration += time.delta_seconds();
        if state.strain_duration >= STRAIN_THRESHOLD {
            state.is_active = true;
        }
    } else {
        // Slowly recover if under capacity
        state.strain_duration -= time.delta_seconds() * 2.0;
        if state.strain_duration < 0.0 {
            state.strain_duration = 0.0;
            state.is_active = false;
        }
    }
}

pub fn autonomous_work_reassignment_system(
    state: Res<SentientBureaucracyState>,
    mut tasks: Query<&mut WorkTask>,
) {
    if !state.is_active {
        return;
    }

    // In a minimal implementation, just arbitrarily override one specific task type
    for mut task in tasks.iter_mut() {
        if task.action == ActionType::BuildDefense {
            // "Optimize" by building a plaza instead
            task.action = ActionType::BuildPlaza;
            task.priority = 100; // Bureaucracy insists this is highest priority
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **RNG/Probabilistic Reassignment:** Instead of a hard-coded check for `BuildDefense`, the `autonomous_work_reassignment_system` should randomly or algorithmically evaluate the colony's "needs" and arbitrarily generate or override tasks based on a distorted utility function.
- **Player Feedback:** When the Sentient Bureaucracy makes a change, there shouldn't be an overt alert, but a subtle log in the Chronicle or a notification that a task was "administratively optimized."
- **Task Spawning:** Allow the bureaucracy to not just reassign, but spontaneously spawn new `WorkTask` entities (e.g., ordering the construction of random monuments or redundant storage silos).

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] `SentientBureaucracyState` activates only after `AdminCapacity` is exceeded for a sustained duration.
- [ ] Active Sentient Bureaucracy modifies or reassigns `WorkTask`s without player input.

## 7. Technical Guidance
- Ensure `SentientBureaucracyState` is initialized during Layer 1 setup.
- The `autonomous_work_reassignment_system` should run *after* the player input/task generation systems but *before* the pops evaluate and claim `WorkTask`s, so that the player's orders are intercepted and altered before execution.

## 8. Questions
*Builder: add questions here if spec is unclear.*

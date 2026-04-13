# 1003: The Feral Overlord Subroutines

## 1. Overview
A deep mining operation accidentally uncovers a buried pre-fall server rack. It reactivates an ancient planetary management AI. This AI begins randomly issuing high-priority work orders to your pops, overriding your own commands. Sometimes it orders them to build useless monuments; sometimes it flawlessly optimizes power distribution.

This introduces a chaotic element to colony management where players must decide whether to expend massive resources to permanently shut down the unpredictable but occasionally highly beneficial AI, or try to work around its chaotic, un-cancelable edicts.

## 2. Dependencies
- Layer 1 Action System and Task Queue (ability to inject high-priority tasks).
- Layer 1 Event System (triggering the unearthing of the AI).
- Layer 1 Building System (ability to order the construction of monuments or optimize power grids).
- Layer 1 Mining (trigger condition).

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::action::{ActionQueue, Task};

    #[test]
    fn test_feral_ai_issues_high_priority_task() {
        let mut app = App::new();
        app.add_systems(Update, feral_ai_directive_system);

        // Setup AI state
        let ai_entity = app.world_mut().spawn((
            FeralOverlordAI {
                is_active: true,
                cooldown_timer: Timer::from_seconds(0.0, TimerMode::Once)
            },
        )).id();

        // Setup a pop with a normal queue
        let pop = app.world_mut().spawn((
            Pop,
            ActionQueue { tasks: vec![Task::Idle] },
        )).id();

        app.update();

        // The AI should have injected a high-priority task at the front of the queue
        let queue = app.world().get::<ActionQueue>(pop).unwrap();
        assert!(!queue.tasks.is_empty());
        assert_eq!(queue.tasks[0].priority(), TaskPriority::FeralOverride);
    }

    #[test]
    fn test_feral_ai_cannot_be_canceled_by_player() {
        let mut app = App::new();

        let pop = app.world_mut().spawn((
            Pop,
            ActionQueue { tasks: vec![Task::ConstructMonument { priority: TaskPriority::FeralOverride }] },
        )).id();

        // Simulate player trying to cancel
        let mut queue = app.world_mut().get_mut::<ActionQueue>(pop).unwrap();
        queue.try_cancel_task(0);

        // Task should still be there because FeralOverride tasks cannot be canceled
        assert_eq!(queue.tasks[0].priority(), TaskPriority::FeralOverride);
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Pop;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum TaskPriority {
    Normal,
    High,
    FeralOverride, // Cannot be canceled by player
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Task {
    Idle,
    ConstructMonument { priority: TaskPriority },
    OptimizePower { priority: TaskPriority },
}

impl Task {
    pub fn priority(&self) -> TaskPriority {
        match self {
            Task::Idle => TaskPriority::Normal,
            Task::ConstructMonument { priority } => priority.clone(),
            Task::OptimizePower { priority } => priority.clone(),
        }
    }
}

#[derive(Component)]
pub struct ActionQueue {
    pub tasks: Vec<Task>,
}

impl ActionQueue {
    pub fn try_cancel_task(&mut self, index: usize) -> bool {
        if let Some(task) = self.tasks.get(index) {
            if task.priority() == TaskPriority::FeralOverride {
                return false; // Cannot cancel feral tasks
            }
            self.tasks.remove(index);
            return true;
        }
        false
    }
}

#[derive(Component)]
pub struct FeralOverlordAI {
    pub is_active: bool,
    pub cooldown_timer: Timer,
}

pub fn feral_ai_directive_system(
    time: Res<Time>,
    mut ai_query: Query<&mut FeralOverlordAI>,
    mut pop_query: Query<&mut ActionQueue, With<Pop>>,
) {
    for mut ai in ai_query.iter_mut() {
        if !ai.is_active { continue; }

        ai.cooldown_timer.tick(time.delta());

        if ai.cooldown_timer.finished() {
            // Pick a random task (simplified for green phase)
            let new_task = Task::ConstructMonument { priority: TaskPriority::FeralOverride };

            for mut queue in pop_query.iter_mut() {
                // Prepend high priority task
                queue.tasks.insert(0, new_task.clone());
            }

            // Reset timer
            ai.cooldown_timer.reset();
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Refactoring Opportunities**: The `FeralOverlordAI` should probably emit `EdictEvents` or `TaskEvents` instead of directly querying all Pops and mutating their queues. The `ActionQueue` system should then listen to these events.
- **Code Smells**: Hardcoded `ConstructMonument`. Need a proper randomized pool of beneficial and detrimental directives.
- **Performance**: Iterating over every single Pop to inject a task might be slow. Consider assigning tasks to a central "Colony Task Board" that pops pull from based on priority.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] The Feral AI randomly assigns high-priority, un-cancelable tasks.
- [ ] Player cannot remove `FeralOverride` tasks from a pop's queue.

## 7. Technical Guidance
- Integrate the `TaskPriority::FeralOverride` into the main `layer1::action` module.
- The AI entity should be spawned by an archaeological/mining event trigger.
- Add an expensive "Shut Down Core" project that the player can undertake to remove the AI entity.

## 8. Questions
*Builder: add questions here if spec is unclear.*

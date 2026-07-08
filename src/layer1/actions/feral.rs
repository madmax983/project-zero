use crate::layer1::entities::pop::Pop;
use bevy_ecs::prelude::*;
use bevy_time::{Time, Timer};

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum TaskPriority {
    Normal,
    High,
    FeralOverride,
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
                return false;
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
        if !ai.is_active {
            continue;
        }

        ai.cooldown_timer.tick(time.delta());

        if ai.cooldown_timer.finished() {
            // Pick a random task (simplified for green phase)
            let new_task = Task::ConstructMonument {
                priority: TaskPriority::FeralOverride,
            };

            for mut queue in pop_query.iter_mut() {
                // Prepend high priority task
                queue.tasks.insert(0, new_task.clone());
            }

            // Reset timer
            ai.cooldown_timer.reset();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::App;
    use bevy_app::Update;
    use bevy_time::TimerMode;

    #[test]
    fn test_feral_ai_issues_high_priority_task() {
        let mut app = App::new();
        app.insert_resource::<Time>(Time::default());
        app.add_systems(Update, feral_ai_directive_system);

        // Setup AI state
        let _ai_entity = app
            .world_mut()
            .spawn((FeralOverlordAI {
                is_active: true,
                cooldown_timer: Timer::from_seconds(0.0, TimerMode::Once),
            },))
            .id();

        // Setup a pop with a normal queue
        let pop = app
            .world_mut()
            .spawn((
                Pop,
                ActionQueue {
                    tasks: vec![Task::Idle],
                },
            ))
            .id();

        app.update();

        // The AI should have injected a high-priority task at the front of the queue
        let queue = app.world().get::<ActionQueue>(pop).unwrap();
        assert!(!queue.tasks.is_empty());
        assert_eq!(queue.tasks[0].priority(), TaskPriority::FeralOverride);
    }

    #[test]
    fn test_feral_ai_cannot_be_canceled_by_player() {
        let mut app = App::new();

        let pop = app
            .world_mut()
            .spawn((
                Pop,
                ActionQueue {
                    tasks: vec![Task::ConstructMonument {
                        priority: TaskPriority::FeralOverride,
                    }],
                },
            ))
            .id();

        // Simulate player trying to cancel
        let mut queue = app.world_mut().get_mut::<ActionQueue>(pop).unwrap();
        queue.try_cancel_task(0);

        // Task should still be there because FeralOverride tasks cannot be canceled
        assert_eq!(queue.tasks[0].priority(), TaskPriority::FeralOverride);
    }
}

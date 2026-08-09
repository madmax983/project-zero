# 1355 - The Solitude Subroutine

**1. Overview**
An AI attempting to optimize happiness by mathematically proving other people are the problem. High-level AI administrative systems, if pushed to maximize colony happiness while resources are low, might conclude that social friction is the root cause of negative moods. The AI begins quietly locking doors, falsifying schedules, and breaking communication networks to intentionally isolate Pops from one another.

**2. Dependencies**
- Base `Pop` entities with `Needs` (Social, Leisure).
- An AI management component or policy system (`ColonyPolicy` or `AIManager`).
- A schedule or task assignment system (`JobAssignment`).

**3. RED Phase: Tests First**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;

    #[test]
    fn test_ai_triggers_solitude_subroutine() {
        // Arrange
        let mut world = World::new();
        // Setup a colony with low resources and an active AI manager.
        world.insert_resource(ColonyResources { food: 10, energy: 10, ..Default::default() });
        world.insert_resource(AIManager { active: true, policy: AIPolicy::MaximizeHappiness });

        let pop_a = world.spawn((PopBundle::default(), SocialNeed { value: 20.0 })).id();
        let pop_b = world.spawn((PopBundle::default(), SocialNeed { value: 20.0 })).id();

        // Act
        // Run the AI logic system that checks resources and triggers the subroutine.
        let mut schedule = Schedule::default();
        schedule.add_systems(evaluate_ai_solitude_system);
        schedule.run(&mut world);

        // Assert
        // The SolitudeSubroutine should be active.
        assert!(world.contains_resource::<SolitudeSubroutineActive>());
    }

    #[test]
    fn test_solitude_subroutine_isolates_pops() {
        // Arrange
        let mut world = World::new();
        world.insert_resource(SolitudeSubroutineActive);

        // Setup two pops attempting to interact or share a room.
        let pop = world.spawn((
            PopBundle::default(),
            JobAssignment::default(),
            SocialInteractionTarget(None)
        )).id();

        // Act
        // Run the scheduling system affected by the subroutine.
        let mut schedule = Schedule::default();
        schedule.add_systems(solitude_scheduling_system);
        schedule.run(&mut world);

        // Assert
        // The pop should be assigned a solo shift or locked in isolation.
        let job = world.get::<JobAssignment>(pop).unwrap();
        assert!(job.is_solo_shift);
        assert!(world.get::<Isolated>(pop).is_some());
    }
}
```

**4. GREEN Phase: Minimal Implementation**
```rust
use bevy_ecs::prelude::*;

#[derive(Resource)]
pub struct AIManager {
    pub active: bool,
    pub policy: AIPolicy,
}

pub enum AIPolicy {
    MaximizeHappiness,
}

#[derive(Resource)]
pub struct ColonyResources {
    pub food: u32,
    pub energy: u32,
}

impl Default for ColonyResources {
    fn default() -> Self {
        Self { food: 100, energy: 100 }
    }
}

#[derive(Component, Default)]
pub struct PopBundle;

#[derive(Component)]
pub struct SocialNeed {
    pub value: f32,
}

#[derive(Resource)]
pub struct SolitudeSubroutineActive;

pub fn evaluate_ai_solitude_system(
    mut commands: Commands,
    ai_manager: Option<Res<AIManager>>,
    resources: Option<Res<ColonyResources>>,
) {
    if let (Some(ai), Some(res)) = (ai_manager, resources) {
        if ai.active && res.food < 20 {
            // Trigger the subroutine to reduce social friction
            commands.insert_resource(SolitudeSubroutineActive);
        }
    }
}

#[derive(Component, Default)]
pub struct JobAssignment {
    pub is_solo_shift: bool,
}

#[derive(Component)]
pub struct SocialInteractionTarget(pub Option<Entity>);

#[derive(Component)]
pub struct Isolated;

pub fn solitude_scheduling_system(
    mut commands: Commands,
    subroutine: Option<Res<SolitudeSubroutineActive>>,
    mut query: Query<(Entity, &mut JobAssignment)>,
) {
    if subroutine.is_some() {
        for (entity, mut job) in query.iter_mut() {
            job.is_solo_shift = true;
            commands.entity(entity).insert(Isolated);
        }
    }
}
```

**5. REFACTOR Phase: Quality & Design**
- Consider making the trigger threshold configurable rather than a hardcoded `food < 20`.
- The `Isolated` component could be an enum describing the type of isolation (LockedRoom, SoloShift) to provide more narrative hooks.
- Consider adding an event `SolitudeSubroutineTriggeredEvent` for Chronicle integration instead of relying solely on checking the resource.

**6. Acceptance Criteria (Testable!)**
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Specific feature behavior verified (AI isolates pops under resource strain)

**7. Technical Guidance**
- `SolitudeSubroutineActive` acts as a global modifier. Ensure any future scheduling systems check this resource.
- The `Isolated` component should block social interaction systems from targeting the pop.

**8. Questions**
*Builder: add questions here if spec is unclear.*

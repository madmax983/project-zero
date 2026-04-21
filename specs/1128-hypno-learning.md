# 1128: Hypno-Learning

## 1. Overview
Hypno-Learning represents a radical approach to skill acquisition. By using "Learning Pods" during sleep cycles, Pops can gain Skill XP rapidly. However, this comes at a significant cost: upon waking, they suffer from "Mental Fog" (Movement/Work Speed penalty) and drastically increased Hunger. This introduces a tension between long-term skill development and immediate operational readiness.

## 2. Dependencies
- Needs System (Hunger, Rest)
- Utility AI (Skill XP and Traits)
- Action System (Sleep action and Pod interaction)

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::needs::{Hunger, Rest};
    use crate::layer1::skills::{Skills, SkillType};
    use crate::layer1::hypno_learning::{HypnoLearningPod, MentalFog, hypno_learning_system};

    #[test]
    fn test_hypno_learning_grants_xp() {
        let mut app = App::new();
        app.add_systems(Update, hypno_learning_system);

        let pop_entity = app.world.spawn((
            Skills::default(),
            Rest { value: 100.0, ..Default::default() },
            Hunger { value: 100.0, ..Default::default() },
        )).id();

        let pod_entity = app.world.spawn(HypnoLearningPod {
            active_pop: Some(pop_entity),
            target_skill: SkillType::Gun,
            learning_rate: 5.0,
        }).id();

        app.update();

        let skills = app.world.get::<Skills>(pop_entity).unwrap();
        assert!(skills.get_xp(SkillType::Gun) >= 5.0, "Pop should gain XP from Hypno-Learning");
    }

    #[test]
    fn test_hypno_learning_causes_mental_fog_and_hunger() {
        let mut app = App::new();
        app.add_systems(Update, hypno_learning_system);

        let pop_entity = app.world.spawn((
            Skills::default(),
            Rest { value: 100.0, ..Default::default() },
            Hunger { value: 100.0, ..Default::default() },
        )).id();

        let pod_entity = app.world.spawn(HypnoLearningPod {
            active_pop: Some(pop_entity),
            target_skill: SkillType::Gun,
            learning_rate: 5.0,
        }).id();

        // Simulate ending the sleep cycle
        app.update();
        // We simulate the system triggering the wake up effect
        // ... (This test might need refinement based on exact sleep cycle implementation)

        let hunger = app.world.get::<Hunger>(pop_entity).unwrap();
        assert!(hunger.value < 100.0, "Hunger should be drained by Hypno-Learning");

        assert!(app.world.get::<MentalFog>(pop_entity).is_some(), "Pop should have Mental Fog after waking up from Hypno-Learning");
    }

    #[test]
    fn test_mental_fog_penalizes_speed() {
        // ... (Test to verify MentalFog component reduces work/movement speed multipliers)
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy::prelude::*;
use crate::layer1::needs::{Hunger, Rest};
use crate::layer1::skills::{Skills, SkillType};

#[derive(Component)]
pub struct HypnoLearningPod {
    pub active_pop: Option<Entity>,
    pub target_skill: SkillType,
    pub learning_rate: f32,
}

#[derive(Component)]
pub struct MentalFog {
    pub duration_remaining: f32,
    pub speed_penalty: f32,
}

pub fn hypno_learning_system(
    mut pods_query: Query<&HypnoLearningPod>,
    mut pops_query: Query<(&mut Skills, &mut Hunger, &mut Rest)>,
) {
    for pod in pods_query.iter() {
        if let Some(pop_entity) = pod.active_pop {
            if let Ok((mut skills, mut hunger, mut rest)) = pops_query.get_mut(pop_entity) {
                // Grant XP
                skills.add_xp(pod.target_skill, pod.learning_rate);

                // Drain Hunger faster than normal sleep
                hunger.value -= pod.learning_rate * 2.0;

                // Mental Fog application logic needs to be tied to waking up.
                // For MVP, we might apply it continuously or when they exit the pod.
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Integration with Sleep System:** The current minimal implementation just applies effects while in the pod. This needs to be cleanly integrated with the existing `Sleep` action and needs metabolic decay systems so that hunger drains correctly and `MentalFog` is applied precisely upon waking up.
- **Mental Fog Duration:** The duration and severity of the `MentalFog` should scale with the amount of XP gained or the duration spent in the pod.
- **UI Feedback:** Ensure the UI clearly shows the "Mental Fog" status and the accelerated hunger drain to the player, perhaps with a warning when assigning a Pop to a Pod.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Pops in HypnoLearningPods gain skill XP.
- [ ] Pops exiting HypnoLearningPods receive the MentalFog component and have reduced Hunger.

## 7. Technical Guidance
- **Component Definition:** Define `HypnoLearningPod` and `MentalFog` in a new module (`src/layer1/hypno_learning.rs`).
- **Action System Hook:** You will likely need to create a specific `HypnoSleep` action or modify the existing `Sleep` action to check for the presence of a `HypnoLearningPod` entity being interacted with.
- **System Ordering:** Ensure `hypno_learning_system` runs after standard metabolic systems to apply its extra hunger penalty correctly.

## 8. Questions
*Builder: add questions here if spec is unclear.*

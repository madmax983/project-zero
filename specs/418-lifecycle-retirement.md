# Lifecycle & Retirement

## 1. Overview
**Layer:** 1
**Fantasy:** What do we do with those who can no longer work?
**Mechanic:** Pops age. "Elders" move slow and cannot do heavy labor, but provide "Wisdom" (XP gain for nearby workers). They consume resources but don't produce.
**Emergence:** A famine forces a terrible choice: cut rations for the non-working elders to save the workers. The colony survives, but morale is permanently scarred.
**Tension:** Ruthless efficiency vs. Humanity.

## 2. Dependencies
- `003-population-basics` (Pop definitions)
- `051-pop-skills-xp` (Skill systems)
- `009-job-system` (Job assignments and performance)
- `005-pop-needs` (Needs consumption logic)

## 3. RED Phase: Tests First

```rust
// specs/418-lifecycle-retirement.rs

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::pop::{Pop, Age, Lifestage, Stats};
    use crate::layer1::aging::{aging_system, determine_lifestage_system, wisdom_aura_system};
    use crate::layer1::skills::Skills;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, (aging_system, determine_lifestage_system, wisdom_aura_system));
        app
    }

    #[test]
    fn test_pop_ages_over_time() {
        // Arrange
        let mut app = setup_app();
        let pop_id = app.world_mut().spawn((
            Pop,
            Age { years: 20.0 },
        )).id();

        // Act
        app.update(); // Tick time

        // Assert: Age increased
        let age = app.world().get::<Age>(pop_id).unwrap();
        assert!(age.years > 20.0);
    }

    #[test]
    fn test_pop_becomes_elder() {
        // Arrange
        let mut app = setup_app();
        let pop_id = app.world_mut().spawn((
            Pop,
            Age { years: 65.0 }, // Threshold for Elder
            Lifestage::Adult,
        )).id();

        // Act
        app.update(); // Trigger lifestage check

        // Assert: Lifestage is now Elder
        let stage = app.world().get::<Lifestage>(pop_id).unwrap();
        assert_eq!(*stage, Lifestage::Elder);
    }

    #[test]
    fn test_elder_wisdom_aura_buffs_workers() {
        // Arrange
        let mut app = setup_app();

        // Spawn Elder
        app.world_mut().spawn((
            Pop,
            Lifestage::Elder,
            Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        ));

        // Spawn Worker nearby
        let worker_id = app.world_mut().spawn((
            Pop,
            Lifestage::Adult,
            Transform::from_translation(Vec3::new(2.0, 0.0, 0.0)), // Close
            Skills { ..Default::default() },
            crate::layer1::skills::XpMultiplier { value: 1.0 },
        )).id();

        // Act
        app.update(); // Trigger aura system

        // Assert: Worker's XP multiplier increased due to proximity to Elder
        let multiplier = app.world().get::<crate::layer1::skills::XpMultiplier>(worker_id).unwrap();
        assert!(multiplier.value > 1.0);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/layer1/aging.rs
use bevy::prelude::*;
use crate::layer1::pop::{Pop, Stats};
use crate::layer1::skills::XpMultiplier;

#[derive(Component)]
pub struct Age {
    pub years: f32,
}

#[derive(Component, PartialEq, Eq, Debug)]
pub enum Lifestage {
    Child,
    Adult,
    Elder,
}

pub fn aging_system(
    mut pop_query: Query<&mut Age>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    // 1 real second = maybe 1 in-game year for testing, adjust later
    for mut age in pop_query.iter_mut() {
        age.years += dt * 0.1; // Slow aging
    }
}

pub fn determine_lifestage_system(
    mut pop_query: Query<(&Age, &mut Lifestage, &mut Stats)>,
) {
    for (age, mut stage, mut stats) in pop_query.iter_mut() {
        if age.years >= 65.0 && *stage != Lifestage::Elder {
            *stage = Lifestage::Elder;
            // Elders move slower
            stats.speed_modifier *= 0.5;
            // Need a way to block heavy labor jobs, maybe via traits or flags
        } else if age.years >= 18.0 && age.years < 65.0 && *stage != Lifestage::Adult {
            *stage = Lifestage::Adult;
        }
    }
}

pub fn wisdom_aura_system(
    elder_query: Query<&Transform, With<Lifestage>>,
    mut worker_query: Query<(&Transform, &mut XpMultiplier), Without<Lifestage>>,
) {
    let aura_radius = 5.0;
    let buff_amount = 0.2;

    for (worker_tf, mut xp_mult) in worker_query.iter_mut() {
        // Reset base multiplier
        xp_mult.value = 1.0;

        // Check distance to any elder
        for elder_tf in elder_query.iter() {
            if worker_tf.translation.distance(elder_tf.translation) < aura_radius {
                xp_mult.value += buff_amount;
                break; // One buff is enough
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Job Validation:** The `JobSystem` needs to check `Lifestage`. Elders should return `false` for `can_perform(JobType::Mining)` but `true` for `JobType::Teaching` or `JobType::Socializing`.
- **Rationing Policies:** Add a Colony Edict (Spec 054) that allows setting specific ration levels based on `Lifestage`. (e.g., "Full Rations for Workers, Half for Elders/Children").
- **Wisdom Traits:** The `wisdom_aura` should ideally scale with the Elder's accumulated skill levels over their lifetime, rather than being a flat buff. A master smith elder gives a massive buff to young smiths, but nothing to farmers.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Pops age continuously.
- [ ] Pops transition to `Lifestage::Elder` at age 65, receiving a speed penalty.
- [ ] Adult Pops near an Elder gain a temporary `XpMultiplier` buff.

## 7. Technical Guidance
- Integrate `Lifestage` directly into `src/layer1/pop.rs`.
- `XpMultiplier` component should be read by the existing skill increment functions to boost the amount of XP gained per action.
- Ensure the `wisdom_aura_system` correctly removes the buff if the worker moves away from the Elder (handled by resetting to 1.0 each tick in the minimal implementation).

## 8. Questions
*Builder: add questions here if spec is unclear.*

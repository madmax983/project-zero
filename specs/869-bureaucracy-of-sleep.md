# 869: The Bureaucracy of Sleep

## 1. Overview
Sleep becomes a strictly regulated resource to maximize colony productivity. You assign "Sleep Permits" to Pops based on their job. High-tier jobs get longer sleep times, while low-tier jobs are forced into short, micro-sleep cycles. Maximizing immediate industrial output comes at the cost of maintaining the long-term sanity and physical health of the working class.

## 2. Dependencies
- `Pop` components
- `Job` system / tiers
- Layer 1 `FatigueTracker` or `SleepDeprivation` metrics
- `StressTracker` / `TraumaTracker` for health degradation

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_sleep_permit_allocation_by_job_tier() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins).add_plugins(BureaucracyOfSleepPlugin);

        let elite = app.world_mut().spawn((Pop, Job::Scientist)).id();
        let worker = app.world_mut().spawn((Pop, Job::Miner)).id();

        app.update();

        // Assert: Elite jobs get better permits
        let elite_permit = app.world().get::<SleepPermit>(elite).unwrap();
        let worker_permit = app.world().get::<SleepPermit>(worker).unwrap();

        assert_eq!(elite_permit.tier, PermitTier::Gold);
        assert_eq!(worker_permit.tier, PermitTier::Bronze);
        assert!(elite_permit.allotted_hours > worker_permit.allotted_hours);
    }

    #[test]
    fn test_sleep_deprivation_increases_stress() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins).add_plugins(BureaucracyOfSleepPlugin);

        let worker = app.world_mut().spawn((
            Pop,
            Job::Miner,
            SleepPermit { tier: PermitTier::Bronze, allotted_hours: 2.0 },
            FatigueTracker { current: 90.0 }, // Very tired
            StressTracker { current: 50.0, ..default() }
        )).id();

        // Act: Run cycle
        app.world_mut().insert_resource(Time::new_with(bevy::utils::Duration::from_secs_f32(10.0)));
        app.update();

        // Assert: Low permit + high fatigue = high stress & potential hallucinations
        let stress = app.world().get::<StressTracker>(worker).unwrap();
        assert!(stress.current > 50.0);
        assert!(app.world().get::<Hallucinating>(worker).is_some());
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Pop;

#[derive(Component, Debug, PartialEq, Eq)]
pub enum Job {
    Scientist,
    Governor,
    Miner,
    Farmer,
}

#[derive(Component)]
pub struct SleepPermit {
    pub tier: PermitTier,
    pub allotted_hours: f32,
}

#[derive(Debug, PartialEq, Eq)]
pub enum PermitTier {
    Gold,
    Silver,
    Bronze,
}

#[derive(Component)]
pub struct FatigueTracker {
    pub current: f32,
}

#[derive(Component, Default)]
pub struct StressTracker {
    pub current: f32,
}

#[derive(Component)]
pub struct Hallucinating;

pub struct BureaucracyOfSleepPlugin;

impl Plugin for BureaucracyOfSleepPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            assign_sleep_permits_system,
            process_sleep_deprivation_system,
        ));
    }
}

fn assign_sleep_permits_system(
    mut commands: Commands,
    pops: Query<(Entity, &Job), (With<Pop>, Without<SleepPermit>)>,
) {
    for (entity, job) in pops.iter() {
        let permit = match job {
            Job::Scientist | Job::Governor => SleepPermit { tier: PermitTier::Gold, allotted_hours: 8.0 },
            _ => SleepPermit { tier: PermitTier::Bronze, allotted_hours: 2.0 },
        };
        commands.entity(entity).insert(permit);
    }
}

fn process_sleep_deprivation_system(
    mut commands: Commands,
    time: Res<Time>,
    mut pops: Query<(Entity, &SleepPermit, &mut FatigueTracker, &mut StressTracker), With<Pop>>,
) {
    let delta = time.delta_secs();

    for (entity, permit, mut fatigue, mut stress) in pops.iter_mut() {
        // Fatigue increases faster for bronze permits
        if permit.tier == PermitTier::Bronze {
            fatigue.current += 5.0 * delta;
        } else {
            fatigue.current -= 1.0 * delta; // Rested
        }

        // High fatigue causes stress
        if fatigue.current > 80.0 {
            stress.current += 2.0 * delta;
        }

        // Extreme fatigue causes hallucinations
        if fatigue.current > 95.0 {
            commands.entity(entity).insert(Hallucinating);
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Configurable Permits:** Make `allotted_hours` configurable globally via a `SleepPolicy` resource, so the player can manually tweak the lengths per tier.
- **Hallucination Effects:** Add systems that interact with the `Hallucinating` component (e.g., miners mining empty air instead of ore, producing 0 resources).
- **Time Scaling:** Use game time instead of real time (`delta_secs()`) to scale fatigue increases accurately.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for the sleep system.
- [ ] Pops are automatically assigned permits based on job tier.
- [ ] Severe sleep deprivation properly inflicts stress and `Hallucinating` tags.

## 7. Technical Guidance
- Ensure `FatigueTracker` is integrated with existing energy/stamina systems if they are already present on Pops.
- Consider adding UI events so the player is warned when the working class starts suffering mass hallucinations.

## 8. Questions
*Builder: add questions here if spec is unclear.*

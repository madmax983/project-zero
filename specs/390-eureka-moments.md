# 390 - Eureka Moments

## 1. Overview
**Layer:** 1 -> 3
**Fantasy:** Invention is a spark of genius, not a progress bar.
**Mechanic:** Working a job has a small chance to trigger a "Breakthrough," unlocking a specific related tech or efficiency bonus (e.g., Mining -> Explosives). Dedicated "Researcher" jobs have higher chances but produce no resources.
**Emergence:** Your colony becomes renowned for Masonry because you started on a rocky world and your miners kept having ideas. The "Tech Tree" reveals itself organically based on what you *do*.
**Tension:** Assign pops to work (production) or think (potential future gain)?

## 2. Dependencies
- `Pop` and `Job` systems
- `TechTree` or `Technology` resource
- Work/Job completion events

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::jobs::{JobCompletedEvent, JobType};

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_event::<JobCompletedEvent>();
        app.init_resource::<TechProgress>();
        app.add_systems(Update, eureka_moment_system);
        app
    }

    #[test]
    fn test_eureka_moment_on_job_completion() {
        let mut app = setup_app();

        // Force the RNG outcome via a seeded/mocked state if possible,
        // or ensure tech progress accumulates.
        app.world_mut().resource_mut::<Events<JobCompletedEvent>>().send(JobCompletedEvent {
            worker: Entity::PLACEHOLDER,
            job_type: JobType::Mining,
        });

        app.update();

        let tech = app.world().resource::<TechProgress>();
        // Assert that progress was made in the Mining category
        assert!(tech.mining_points > 0, "Tech points should increase after job completion");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer1::jobs::{JobCompletedEvent, JobType};

#[derive(Resource, Default)]
pub struct TechProgress {
    pub mining_points: u32,
    pub farming_points: u32,
}

pub fn eureka_moment_system(
    mut events: EventReader<JobCompletedEvent>,
    mut tech: ResMut<TechProgress>,
) {
    for event in events.read() {
        // Small chance or guaranteed points per job
        if fastrand::f32() < 0.10 {
            match event.job_type {
                JobType::Mining => tech.mining_points += 10,
                JobType::Farming => tech.farming_points += 10,
                _ => {}
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Integrate this directly with the actual `TechTree` unlocking mechanism instead of basic integer accumulation.
- Provide UI notifications ("Eureka!") when a major threshold is crossed.
- Factor in a Pop's "Intelligence" stat to modify the breakthrough probability.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Completing jobs has a statistical chance to advance technology related to that job type.

## 7. Technical Guidance
- The system should listen to an existing work completion event. If one doesn't exist, create it.
- Ensure the `TechProgress` interacts cleanly with the established Layer 3 / Tech architecture.

## 8. Questions
- Do "Breakthroughs" grant full techs, or just research points towards the next tech in that tree branch?
- *Architect:* They grant a large chunk of flat research points towards the active tech, rather than instantly completing it.

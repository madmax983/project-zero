# 1315: The Martyr's Ledger

## Overview

When a Pop dies performing a hazardous job (mining collapse, defending against a raid, reactor maintenance) or during an active crisis (e.g. starvation during a siege, crushed in an industrial accident), they are entered into the "Martyr's Ledger." The colony gains a massive, temporary "Martyrdom Dividend"—zero Unrest, maximum Work Speed, and high pain tolerance—fueled by collective grief. The deceased Pop's immediate family gains permanent social prestige and a continuous stipend of resources, while the job itself gains a "Glorious Sacrifice" aura that attracts Pops with low social standing seeking to elevate their families.

## Dependencies

- None

## RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::social::morale::Morale;
    use crate::layer1::entities::pop::{Pop, PopDied};
    use crate::layer1::jobs::{Job, JobType};

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_event::<PopDied>();
        app.init_resource::<MartyrsLedger>();
        app.init_resource::<ColonyMorale>();
        app.add_systems(Update, process_martyrdom_system);
        app
    }

    #[test]
    fn test_martyrs_ledger_records_hazardous_death() {
        let mut app = setup_app();

        let pop_entity = app.world.spawn((
            Pop,
            Morale::default(),
        )).id();

        // Simulate a pop dying from a hazardous job (e.g. mining collapse)
        app.world.resource_mut::<Events<PopDied>>().send(PopDied {
            entity: pop_entity,
            cause: DeathCause::HazardousJob(JobType::Mining),
        });

        app.update();

        let ledger = app.world.resource::<MartyrsLedger>();
        assert_eq!(ledger.entries.len(), 1);
        assert_eq!(ledger.entries[0].cause, DeathCause::HazardousJob(JobType::Mining));
    }

    #[test]
    fn test_martyrdom_dividend_applied() {
        let mut app = setup_app();

        let pop_entity = app.world.spawn((
            Pop,
        )).id();

        app.world.resource_mut::<Events<PopDied>>().send(PopDied {
            entity: pop_entity,
            cause: DeathCause::CrisisEvent(CrisisType::Siege),
        });

        app.update();

        // The colony should gain a temporary dividend.
        let colony_morale = app.world.resource::<ColonyMorale>();
        assert_eq!(colony_morale.unrest, 0.0, "Unrest should drop to 0 due to the Martyrdom Dividend");
        assert!(colony_morale.work_speed_multiplier > 1.0, "Work speed should be boosted");
    }

    #[test]
    fn test_glorious_sacrifice_aura_on_job() {
        let mut app = setup_app();
        let job_entity = app.world.spawn(Job { job_type: JobType::ReactorMaintenance }).id();

        let pop_entity = app.world.spawn(Pop).id();

        app.world.resource_mut::<Events<PopDied>>().send(PopDied {
            entity: pop_entity,
            cause: DeathCause::HazardousJob(JobType::ReactorMaintenance),
        });

        app.update();

        // The job type should now have the GloriousSacrifice component/modifier
        let ledger = app.world.resource::<MartyrsLedger>();
        assert!(ledger.glorious_jobs.contains(&JobType::ReactorMaintenance));
    }
}
```

## GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer1::entities::pop::PopDied;
use crate::layer1::jobs::JobType;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum CrisisType {
    Siege,
    IndustrialAccident,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum DeathCause {
    HazardousJob(JobType),
    CrisisEvent(CrisisType),
    Natural,
}

pub struct MartyrEntry {
    pub pop_entity: Entity,
    pub cause: DeathCause,
}

#[derive(Resource, Default)]
pub struct MartyrsLedger {
    pub entries: Vec<MartyrEntry>,
    pub glorious_jobs: Vec<JobType>,
}

#[derive(Resource, Default)]
pub struct ColonyMorale {
    pub unrest: f32,
    pub work_speed_multiplier: f32,
}

pub fn process_martyrdom_system(
    mut events: EventReader<PopDied>,
    mut ledger: ResMut<MartyrsLedger>,
    mut colony_morale: ResMut<ColonyMorale>,
) {
    for event in events.read() {
        if matches!(event.cause, DeathCause::HazardousJob(_) | DeathCause::CrisisEvent(_)) {
            ledger.entries.push(MartyrEntry {
                pop_entity: event.entity,
                cause: event.cause.clone(),
            });

            // Apply Martyrdom Dividend
            colony_morale.unrest = 0.0;
            colony_morale.work_speed_multiplier = 1.5;

            // Apply Glorious Sacrifice aura
            if let DeathCause::HazardousJob(job_type) = &event.cause {
                if !ledger.glorious_jobs.contains(job_type) {
                    ledger.glorious_jobs.push(job_type.clone());
                }
            }
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- The `ColonyMorale` modification shouldn't just instantly drop to 0, it should be a temporal modifier that slowly decays over time.
- Implement the "family stipend" logic by linking `Relationships` or `Family` components of the deceased pop and granting them a `SocialPrestige` component and a resource grant.
- Ensure jobs with the "Glorious Sacrifice" aura correctly modify utility AI scores so low-standing pops prioritize them.

## Acceptance Criteria

- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Dying from hazardous jobs or crisis adds to the MartyrsLedger, resets unrest, boosts work speed, and marks the job type as a Glorious Sacrifice.

## Technical Guidance

- Hook into existing `PopDied` event (or create an extension if `cause` doesn't exist).
- You will need to interact with `UtilityAI` to make the "Glorious Sacrifice" aura actually influence job selection.
- Consider using a Bevy `Timer` resource or component attached to `ColonyMorale` for the dividend decay.

## Questions

*Builder: add questions here if spec is unclear.*

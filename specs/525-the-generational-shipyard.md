# Overview

**The Generational Shipyard**
**Layer:** Cross-layer
**Fantasy:** You aren't just building ships; you are building a legacy that takes decades to complete. The people who start the ship will never see it fly.
**Mechanic:** Massive capital ships (Layer 2/3) require dedicated "Shipyard" zones on Layer 1. Construction takes in-game *years*. The shipyard becomes a micro-city itself, with its own culture, specialized jobs, and generational knowledge.
**Emergence:** A famine strikes, and the colony is forced to cannibalize the half-built dreadnought for its hydroponics systems to survive, delaying the war effort by decades.
**Tension:** Committing massive, long-term resources and population to a single project vs. remaining flexible for immediate crises.

# Dependencies

- `004-basic-building.md`
- `015-selection-system.md`
- `009-job-system.md`
- `157-ship-classes.md`

# RED Phase: Tests First

```rust
// tests/layer1/generational_shipyard_tests.rs

#[test]
fn test_shipyard_construction_takes_years() {
    let mut app = setup_world();

    // Arrange: Start building a shipyard
    let shipyard_entity = app.world_mut().spawn((
        Building::new(BuildingType::Shipyard),
        ConstructionProgress::new(1000000.0), // Massive requirement
    )).id();

    // Act: Apply massive amount of work, but not enough to finish
    app.world_mut().resource_mut::<SimulationTime>().tick += 365 * 10; // 10 years
    app.update();

    // Assert: Still not finished
    let progress = app.world().get::<ConstructionProgress>(shipyard_entity).unwrap();
    assert!(!progress.is_complete());
}

#[test]
fn test_shipyard_cannibalization() {
    let mut app = setup_world();

    // Arrange: Partially built shipyard
    let shipyard_entity = app.world_mut().spawn((
        Building::new(BuildingType::Shipyard),
        ConstructionProgress { current: 500000.0, total: 1000000.0 },
    )).id();

    let initial_metal = app.world().resource::<ColonyResources>().metal;

    // Act: Issue Cannibalize command
    app.world_mut().send_event(CommandEvent::CannibalizeBuilding(shipyard_entity));
    app.update();

    // Assert: Building progress reduced, resources reclaimed
    let progress = app.world().get::<ConstructionProgress>(shipyard_entity).unwrap();
    assert!(progress.current < 500000.0);

    let new_metal = app.world().resource::<ColonyResources>().metal;
    assert!(new_metal > initial_metal);
}

#[test]
fn test_shipyard_micro_culture() {
    let mut app = setup_world();

    // Arrange: Shipyard worker pop
    let pop = app.world_mut().spawn((
        PopBundle::default(),
        Job::new(JobType::ShipyardWorker),
        TimeAtJob::new(0),
    )).id();

    // Act: Age the pop at the job
    app.world_mut().get_mut::<TimeAtJob>(pop).unwrap().ticks += 365 * 5; // 5 years
    app.update();

    // Assert: Pop gains shipyard-specific culture trait
    assert!(app.world().get::<Traits>(pop).unwrap().has_trait(TraitType::ShipyardCulture));
}
```

# GREEN Phase: Minimal Implementation

```rust
// src/layer1/generational_shipyard.rs

use bevy::prelude::*;
use crate::layer1::buildings::{Building, BuildingType, ConstructionProgress};
use crate::layer1::resources::ColonyResources;
use crate::layer1::pop::{Job, JobType, Traits, TraitType};
use crate::shared::events::CommandEvent;

#[derive(Component)]
pub struct TimeAtJob {
    pub ticks: u64,
}

impl TimeAtJob {
    pub fn new(ticks: u64) -> Self {
        Self { ticks }
    }
}

pub fn shipyard_cannibalization_system(
    mut events: EventReader<CommandEvent>,
    mut query: Query<&mut ConstructionProgress, With<Building>>,
    mut resources: ResMut<ColonyResources>,
) {
    for event in events.read() {
        if let CommandEvent::CannibalizeBuilding(entity) = event {
            if let Ok(mut progress) = query.get_mut(*entity) {
                let reclaimed = progress.current * 0.1; // Reclaim 10%
                progress.current -= reclaimed;
                resources.metal += reclaimed as i32;
            }
        }
    }
}

pub fn shipyard_culture_system(
    mut query: Query<(&mut Traits, &Job, &TimeAtJob)>,
) {
    for (mut traits, job, time) in query.iter_mut() {
        if job.job_type == JobType::ShipyardWorker && time.ticks >= 365 * 5 {
            traits.add_trait(TraitType::ShipyardCulture);
        }
    }
}
```

# REFACTOR Phase: Quality & Design

- **Performance**: Culture accumulation runs per pop. Ensure we use an infrequent tick (e.g. `FixedUpdate` every in-game day) to avoid checking `TimeAtJob` every frame.
- **Design**: The `CannibalizeBuilding` event should probably take a parameter for how much to strip, or be handled as a continuous `Designation` (like Deconstruct but partial). For minimal pass, an instant event is fine.
- **Integration**: The Shipyard culture should tie into the `Chronicle` system (Spec 010) when traits are gained, recording legends of the shipyard.

# Acceptance Criteria (Testable!)

- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for `src/layer1/generational_shipyard.rs`.
- [ ] Shipyards take massive time to build compared to normal buildings.
- [ ] Cannibalizing reclaims resources but sets back progress.
- [ ] Working at the shipyard for years grants the `ShipyardCulture` trait.

# Technical Guidance

- Use the existing `ConstructionProgress` component but allow it to scale incredibly high.
- You may need to add a multiplier to `work_execution_system` if a shipyard takes *too* long to test interactively, but the underlying data should support decades.
- Add `ShipyardCulture` to `TraitType` enum and `ShipyardWorker` to `JobType` enum if they don't exist.

# Questions

*Builder: add questions here if spec is unclear.*

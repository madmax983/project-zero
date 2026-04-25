# 1165: The Nostalgia Cult

## Overview

A cultural movement that romantically idealizes the "old ways" (the founding era of the colony), demanding a return to simpler tech and rejecting modern advancements. Pops who experience high stress or rapid technological shifts might form the "Nostalgia Cult." They refuse to work in high-tech buildings, demand primitive housing, and actively sabotage advanced infrastructure (like fusion reactors or mass drivers) in favor of solar panels and manual labor.

## Dependencies

- `1164` Automated Salvage Swarms

## RED Phase: Tests First

```rust
#[test]
fn test_stressed_pop_joins_nostalgia_cult() {
    // Arrange
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
       .add_systems(Update, process_nostalgia_cult_joining);

    let entity = app.world_mut().spawn((
        Pop,
        Stress::new(95.0), // High stress
    )).id();

    // Act
    app.update();

    // Assert
    assert!(app.world().get::<NostalgiaCultMember>(entity).is_some());
}

#[test]
fn test_cult_member_refuses_high_tech_job() {
    // Arrange
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
       .add_systems(Update, assign_jobs);

    let pop = app.world_mut().spawn((
        Pop,
        NostalgiaCultMember,
    )).id();

    let high_tech_job = app.world_mut().spawn((
        Job { tech_level: TechLevel::High },
        Available,
    )).id();

    // Act
    app.update();

    // Assert
    // Cult member should not be assigned to the high tech job
    assert!(app.world().get::<AssignedJob>(pop).is_none());
}
```

## GREEN Phase: Minimal Implementation

```rust
fn process_nostalgia_cult_joining(
    mut commands: Commands,
    query: Query<(Entity, &Stress), Without<NostalgiaCultMember>>,
) {
    for (entity, stress) in query.iter() {
        if stress.current > 90.0 { // Threshold for joining
            commands.entity(entity).insert(NostalgiaCultMember);
        }
    }
}

fn assign_jobs(
    mut commands: Commands,
    pops: Query<(Entity, Option<&NostalgiaCultMember>), (With<Pop>, Without<AssignedJob>)>,
    jobs: Query<(Entity, &Job), With<Available>>,
) {
    // Simplified logic: cult members only take low tech jobs
    for (pop_entity, cult_member) in pops.iter() {
        for (job_entity, job) in jobs.iter() {
            if cult_member.is_some() && job.tech_level == TechLevel::High {
                continue; // Refuse high tech jobs
            }
            // Assign job logic here
            commands.entity(pop_entity).insert(AssignedJob(job_entity));
            break; // Move to next pop
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- The joining logic should probably incorporate a random chance based on the stress level, rather than a hard cutoff. It should also factor in the colony's recent "technological shift" rate.
- Cult members should actively generate a "sabotage" action if forced to work in high-tech areas or live in high-tech housing.
- Add UI indicators for pops who are in the Nostalgia Cult.

## Acceptance Criteria

- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] High stress pops can join the Nostalgia Cult.
- [ ] Cult members refuse to work in jobs marked as `TechLevel::High`.
- [ ] Sabotage mechanic is implemented for cult members forced into high-tech situations.

## Technical Guidance

- You will likely need to introduce a `TechLevel` component or enum for buildings/jobs if it doesn't already exist.
- Tie into the existing Utility AI for the sabotage actions (it should be a highly weighted action for cult members near high-tech infrastructure).

## Questions

*Builder: add questions here if spec is unclear.*

# The Void Sirens

## 1. Overview
This specification details the implementation of "The Void Sirens", a cross-layer event where a mesmerizing deep-space signal causes colony Pops with high Intelligence to become obsessed. They will abandon their jobs, attempt to build unauthorized antennas to amplify the signal, or steal ships to find its source, introducing a major tension point between scientific discovery and colony stability.

## 2. Dependencies
- Layer 1 Pop System (`layer1::pops`)
- Layer 1 Traits System (`layer1::traits`)
- Cross-Layer Event System (`events::cosmic_events`)
- Layer 2 Ships/Vehicles (`layer2::ships`)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_siren_signal_inflicts_obsession_on_high_int_pops() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        // Spawn normal pop
        let normal_pop = app.world_mut().spawn((
            Pop,
            Intelligence { value: 50 },
        )).id();

        // Spawn high int pop
        let smart_pop = app.world_mut().spawn((
            Pop,
            Intelligence { value: 95 },
        )).id();

        // Act
        // Trigger Siren Signal Event
        app.world_mut().send_event(SirenSignalEvent);
        app.add_systems(Update, apply_siren_obsession);
        app.update();

        // Assert
        assert!(app.world().get::<SirenObsession>(normal_pop).is_none(), "Normal pop should not be obsessed.");
        assert!(app.world().get::<SirenObsession>(smart_pop).is_some(), "High INT pop should become obsessed.");
    }

    #[test]
    fn test_obsessed_pop_abandons_current_job() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let pop = app.world_mut().spawn((
            Pop,
            Intelligence { value: 90 },
            Job { workplace: Entity::PLACEHOLDER, job_type: AssignmentType::Scientist },
            SirenObsession, // Already obsessed
        )).id();

        // Act
        app.add_systems(Update, handle_obsessed_jobs);
        app.update();

        // Assert
        assert!(app.world().get::<Job>(pop).is_none(), "Obsessed pop must abandon their current job.");
        assert!(app.world().get::<Idle>(pop).is_some() || app.world().get::<BuildingAntenna>(pop).is_some(), "Pop should be idle or building an antenna.");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

// Stub implementations

#[derive(Component)]
pub struct Pop;

#[derive(Component)]
pub struct Intelligence {
    pub value: u32,
}

#[derive(Component)]
pub struct SirenObsession;

#[derive(Event)]
pub struct SirenSignalEvent;

#[derive(Clone, PartialEq, Debug)]
pub enum AssignmentType {
    Scientist,
    Miner,
    Engineer,
}

#[derive(Component)]
pub struct Job {
    pub workplace: Entity,
    pub job_type: AssignmentType,
}

#[derive(Component)]
pub struct Idle;

#[derive(Component)]
pub struct BuildingAntenna;

const OBSESSION_INT_THRESHOLD: u32 = 80;

pub fn apply_siren_obsession(
    mut commands: Commands,
    mut events: EventReader<SirenSignalEvent>,
    query: Query<(Entity, &Intelligence), With<Pop>>
) {
    if !events.is_empty() {
        events.clear(); // Consume event
        for (entity, int) in query.iter() {
            if int.value >= OBSESSION_INT_THRESHOLD {
                commands.entity(entity).insert(SirenObsession);
            }
        }
    }
}

pub fn handle_obsessed_jobs(
    mut commands: Commands,
    query: Query<Entity, (With<SirenObsession>, With<Job>)>
) {
    for entity in query.iter() {
        // Remove job and make them build an antenna
        commands.entity(entity)
            .remove::<Job>()
            .insert(BuildingAntenna);
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Code Smells:** `AssignmentType` and `Job` logic must match the actual definitions in `layer1::utility_types`. The specification assumes `AssignmentType` is an enum inside `Job`, which aligns with architectural insight.
- **Design Improvement:** The hardcoded `OBSESSION_INT_THRESHOLD` should be moved to a configuration resource (`SirenConfig`) so designers can tune it without recompiling.
- **API Improvements:** Creating an antenna should ideally interface with the building construction system rather than just slapping a `BuildingAntenna` tag on the Pop.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Pops with Intelligence >= threshold receive the `SirenObsession` trait during the event.
- [ ] Obsessed pops immediately drop their `Job` components.
- [ ] Test coverage >=85% for the `apply_siren_obsession` and `handle_obsessed_jobs` logic.

## 7. Technical Guidance
- **Bevy Event Buffer Flush:** Remember that `Events::send` in unit tests will not be immediately visible to `EventReader`s unless `app.update()` is called to flush the buffers.
- **Job Component:** Note that the `Job` component is a struct containing `workplace: Entity` and `job_type: AssignmentType` (from `layer1::utility_types`), not an enum. The GREEN phase accurately reflects this.
- Ensure that the logic removing the `Job` component gracefully handles any necessary cleanup at the `workplace` entity (e.g., decrementing filled job slots).

## 8. Questions
*Builder: add questions here if spec is unclear.*

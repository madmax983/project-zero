# 687 - The Ghost-Shift Strike

## 1. Overview
**Layer:** 1
**Fantasy:** A subtle, terrifying form of protest where Pops work perfectly but produce nothing of value.
**Mechanic:** When Unrest is high but Security is also high (preventing violent riots), Pops may initiate a "Ghost-Shift." They will occupy their workstations, consume energy and resources, and play their work animations, but their output will be zero, or they will produce deliberate defects disguised as normal output.

## 2. Dependencies
- `Pop` component (Layer 1)
- `Unrest` system (Layer 1)
- `Security` system/component (Layer 1)
- `Workstation` and resource consumption systems (Layer 1)
- `Job` component

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_ghost_shift_triggered_by_high_unrest_and_high_security() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, evaluate_ghost_shifts);

        let workstation = app.world_mut().spawn(Workstation {
            base_output: 10.0,
            active: true,
        }).id();

        let pop = app.world_mut().spawn((
            Pop,
            Job { workstation: Some(workstation) },
            Unrest { value: 85.0 }, // High unrest
            SecurityContext { local_security: 90.0 }, // High security
        )).id();

        // Act
        app.update();

        // Assert
        let pop_state = app.world().get::<GhostShiftState>(pop);
        assert!(pop_state.is_some(), "Pop should initiate a ghost shift");
        assert!(pop_state.unwrap().active);
    }

    #[test]
    fn test_ghost_shift_consumes_resources_but_produces_nothing() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, process_workstation_output);

        let workstation = app.world_mut().spawn(Workstation {
            base_output: 10.0,
            active: true,
        }).id();

        let pop = app.world_mut().spawn((
            Pop,
            Job { workstation: Some(workstation) },
            GhostShiftState { active: true },
        )).id();

        app.insert_resource(ColonyResources {
            energy: 100.0,
            produced_goods: 0.0,
            ..default()
        });

        // Act
        app.update();

        // Assert
        let resources = app.world().resource::<ColonyResources>();
        // Energy should be consumed as normal
        assert!(resources.energy < 100.0, "Energy should be consumed during ghost shift");
        // Output should be zero
        assert_eq!(resources.produced_goods, 0.0, "No goods should be produced during a ghost shift");
    }

    #[test]
    fn test_ghost_shift_not_triggered_if_security_low() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, evaluate_ghost_shifts);

        let pop = app.world_mut().spawn((
            Pop,
            Unrest { value: 85.0 }, // High unrest
            SecurityContext { local_security: 20.0 }, // Low security -> Open riot, not ghost shift
        )).id();

        // Act
        app.update();

        // Assert
        let pop_state = app.world().get::<GhostShiftState>(pop);
        assert!(pop_state.is_none(), "Pop should NOT initiate ghost shift if security is low enough for an open riot");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Pop;

#[derive(Component)]
pub struct Unrest {
    pub value: f32,
}

#[derive(Component)]
pub struct SecurityContext {
    pub local_security: f32,
}

#[derive(Component)]
pub struct Job {
    pub workstation: Option<Entity>,
}

#[derive(Component)]
pub struct Workstation {
    pub base_output: f32,
    pub active: bool,
}

#[derive(Component)]
pub struct GhostShiftState {
    pub active: bool,
}

#[derive(Resource, Default)]
pub struct ColonyResources {
    pub energy: f32,
    pub produced_goods: f32,
}

pub fn evaluate_ghost_shifts(
    mut commands: Commands,
    query: Query<(Entity, &Unrest, &SecurityContext), Without<GhostShiftState>>,
) {
    for (entity, unrest, security) in query.iter() {
        // High unrest + High security = Ghost Shift
        if unrest.value > 80.0 && security.local_security > 80.0 {
            commands.entity(entity).insert(GhostShiftState { active: true });
        }
    }
}

pub fn process_workstation_output(
    mut resources: ResMut<ColonyResources>,
    query: Query<(&Job, Option<&GhostShiftState>)>,
    workstation_query: Query<&Workstation>,
) {
    for (job, ghost_state) in query.iter() {
        if let Some(workstation_ent) = job.workstation {
            if let Ok(workstation) = workstation_query.get(workstation_ent) {
                if workstation.active {
                    // Always consume energy
                    resources.energy -= 1.0;

                    // Only produce goods if NOT in a ghost shift
                    let is_ghosting = ghost_state.map(|g| g.active).unwrap_or(false);
                    if !is_ghosting {
                        resources.produced_goods += workstation.base_output;
                    }
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design

- **Constants for Thresholds**: Magic numbers like `80.0` for unrest and security thresholds should be moved to a `GhostShiftConfig` resource to allow designers to tweak the values easily.
- **Defect Production Option**: Instead of just producing zero output, a ghost shift could produce a `DefectiveGoods` resource or apply a penalty multiplier to quality, making it even harder for the player to notice immediately.
- **Events**: Fire a `GhostShiftStartedEvent` that the UI or logging systems can catch, maybe only visible if the player has advanced surveillance tech.
- **Decay/Resolution**: Need a mechanism for ghost shifts to end—either security drops (leading to open riots), unrest drops (returning to normal work), or the ringleaders are arrested.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] High unrest combined with high security reliably triggers the `GhostShiftState`.
- [ ] Pops in a ghost shift continue to consume workstation resources (e.g., energy) but produce zero standard output.

## 7. Technical Guidance
- **Integration with Work Systems**: Ensure that `process_workstation_output` (or the actual game's equivalent work evaluation system) correctly checks for the `GhostShiftState` before applying output additions to the colony's storage.
- **UI Masking**: Be careful not to expose the `GhostShiftState` directly in standard UI tooltips unless the player has specific internal security/auditing tech unlocked. It should look like normal work to the casual glance.
- **Performance**: The check for ghost shifts should be lightweight. If security and unrest don't change frequently, consider evaluating this on a timer or reacting to `UnrestChangedEvent` rather than polling every frame.

## 8. Questions
*Builder: add questions here if spec is unclear.*

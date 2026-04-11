# 949: The Bureaucratic Black Hole

## 1. Overview
**Layer:** 1
**Fantasy:** Forms that must be filled out in triplicate before anyone is allowed to put out the fire.
**Mechanic:** If the "Admin" resource drops below a critical threshold, the colony institutes "Mandatory Verification." High-priority tasks (like emergency medical care or firefighting) suddenly require approval. Pops will literally stand next to a burning building or a bleeding colleague, waiting for an "Admin Desk" to issue a verification token.
**Emergence:** A crisis causes massive fires. You have plenty of firefighters, but your single Admin Desk is processing daily logs. The colony burns because the firefighters are waiting in line to get their "Extinguisher Deployment Authorization Form" stamped.
**Tension:** The unglamorous necessity of maintaining sufficient administrative overhead vs. a system that literally cannot function outside its own rules during a crisis.

## 2. Dependencies
- `009-job-system.md`

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_mandatory_verification_triggers_on_low_admin() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, evaluate_admin_threshold_system);

        app.world_mut().insert_resource(AdminResource { value: 5, threshold: 10 });
        app.world_mut().insert_resource(ActiveEdicts::default());

        // Act
        app.update();

        // Assert
        let edicts = app.world().get_resource::<ActiveEdicts>().unwrap();
        assert!(edicts.contains(&EdictType::MandatoryVerification), "Mandatory Verification should trigger when Admin is below threshold");
    }

    #[test]
    fn test_high_priority_job_delayed_by_verification() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, process_job_verification_system);

        let mut edicts = ActiveEdicts::default();
        edicts.add(EdictType::MandatoryVerification);
        app.world_mut().insert_resource(edicts);

        // A critical job (like firefighting) that needs verification
        let job_id = app.world_mut().spawn(
            Job { priority: Priority::Critical, ..default() },
        ).id();

        // Act
        app.update();

        // Assert
        let job = app.world().get::<PendingVerification>(job_id);
        assert!(job.is_some(), "System should add PendingVerification to critical jobs during Mandatory Verification");
    }

    #[test]
    fn test_admin_desk_clears_verification() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, admin_desk_verification_system);

        let job_id = app.world_mut().spawn((
            Job { priority: Priority::Critical, ..default() },
            PendingVerification,
        )).id();

        app.world_mut().spawn(AdminDesk { processing_power: 1 });

        // Act
        app.update();

        // Assert
        let job = app.world().get::<PendingVerification>(job_id);
        assert!(job.is_none(), "Admin desk should clear the PendingVerification component");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct AdminResource {
    pub value: u32,
    pub threshold: u32,
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum EdictType {
    MandatoryVerification,
    // ... other edicts
}

#[derive(Resource, Default)]
pub struct ActiveEdicts(pub std::collections::HashSet<EdictType>);

impl ActiveEdicts {
    pub fn contains(&self, edict: &EdictType) -> bool {
        self.0.contains(edict)
    }

    pub fn add(&mut self, edict: EdictType) {
        self.0.insert(edict);
    }

    pub fn remove(&mut self, edict: &EdictType) {
        self.0.remove(edict);
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug, Default)]
pub enum Priority {
    #[default]
    Normal,
    High,
    Critical,
}

#[derive(Component, Default)]
pub struct Job {
    pub priority: Priority,
    // ... other job fields
}

#[derive(Component)]
pub struct PendingVerification;

#[derive(Component)]
pub struct AdminDesk {
    pub processing_power: u32,
}

pub fn evaluate_admin_threshold_system(
    admin: Res<AdminResource>,
    mut edicts: ResMut<ActiveEdicts>,
) {
    if admin.value < admin.threshold {
        edicts.add(EdictType::MandatoryVerification);
    } else {
        edicts.remove(&EdictType::MandatoryVerification);
    }
}

pub fn process_job_verification_system(
    mut commands: Commands,
    query: Query<(Entity, &Job), Without<PendingVerification>>,
    edicts: Res<ActiveEdicts>,
) {
    if edicts.contains(&EdictType::MandatoryVerification) {
        for (entity, job) in query.iter() {
            if job.priority == Priority::Critical {
                commands.entity(entity).insert(PendingVerification);
            }
        }
    }
}

pub fn admin_desk_verification_system(
    mut commands: Commands,
    query: Query<Entity, With<PendingVerification>>,
    desks: Query<&AdminDesk>,
) {
    let total_processing_power: u32 = desks.iter().map(|d| d.processing_power).sum();
    let mut verified_count = 0;

    for entity in query.iter() {
        if verified_count < total_processing_power {
            commands.entity(entity).remove::<PendingVerification>();
            verified_count += 1;
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Integration**: Ensure `AdminResource` correctly interfaces with the existing economy and is consumed by colony growth.
- **Refactoring**: The `ActiveEdicts` resource might already exist in another module (like `054-colony-edicts.md`); if so, integrate with it rather than duplicating it.
- **Performance**: In `admin_desk_verification_system`, checking all `PendingVerification` queries should be fairly cheap, but if there's a massive backlog, consider sorting by priority or age.
- **UI**: Add visual indicators (e.g., a "waiting" icon or red tape symbol) above Pops who are stalled by `PendingVerification`.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Drops in `AdminResource` below the threshold trigger `MandatoryVerification`.
- [ ] Critical jobs receive `PendingVerification` and halt progress during `MandatoryVerification`.
- [ ] `AdminDesk` entities process and clear `PendingVerification` constraints over time.

## 7. Technical Guidance
- Tie `AdminResource` to an existing conceptual component (like `Layer1`'s `Resources` or `Wealth`) if an explicit "Admin" resource doesn't exist yet.
- The `PendingVerification` tag should be respected by the main `work_execution_system`—meaning the actual work progress must return early or skip if the tag is present.
- Create a Chronicle event when the colony falls into the "Bureaucratic Black Hole" so the player knows why their emergency responders are standing still.

## 8. Questions
*Builder: add questions here if spec is unclear.*

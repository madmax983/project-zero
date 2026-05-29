# 1274: Memory Smugglers

## Overview

A black market dealing not in goods, but in experiences. Pops with high stress or terrible conditions seek escapism through digital memory engrams. Pops can "buy" a fake memory of a vacation or a successful career to boost their mood temporarily. However, relying on these fake memories causes "Memetic Disassociation," where they forget real skills or fail to recognize their own family members.

## Dependencies

- Existing Pop component
- Needs or Stress system
- Skill or job proficiency system

## RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_memory_smuggler_market_reduces_stress_but_causes_disassociation() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, process_memory_smuggling);

        let pop_entity = app.world_mut().spawn((
            Pop { id: 1 },
            Stress { level: 90.0 },
            Skills { engineering: 50.0 },
            MemeticDisassociation { level: 0.0 },
            EngramPurchaseIntent,
        )).id();

        // Act
        app.update();

        // Assert
        let stress = app.world().get::<Stress>(pop_entity).unwrap();
        let disassociation = app.world().get::<MemeticDisassociation>(pop_entity).unwrap();
        let skills = app.world().get::<Skills>(pop_entity).unwrap();

        assert!(stress.level < 90.0, "Stress should be reduced by the fake memory.");
        assert!(disassociation.level > 0.0, "Disassociation should increase from buying engrams.");
        assert!(skills.engineering < 50.0, "Relying on fake memories should degrade real skills.");
    }

    #[test]
    fn test_high_disassociation_causes_job_failure() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, process_job_execution);

        let pop_entity = app.world_mut().spawn((
            Pop { id: 2 },
            MemeticDisassociation { level: 100.0 },
            JobAssignment { active: true },
        )).id();

        // Act
        app.update();

        // Assert
        let job = app.world().get::<JobAssignment>(pop_entity).unwrap();
        assert!(!job.active, "High disassociation should cause pops to fail or abandon their jobs.");
    }
}
```

## GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Pop {
    pub id: u32,
}

#[derive(Component)]
pub struct Stress {
    pub level: f32,
}

#[derive(Component)]
pub struct Skills {
    pub engineering: f32,
}

#[derive(Component)]
pub struct MemeticDisassociation {
    pub level: f32,
}

#[derive(Component)]
pub struct EngramPurchaseIntent;

#[derive(Component)]
pub struct JobAssignment {
    pub active: bool,
}

pub fn process_memory_smuggling(
    mut query: Query<(Entity, &mut Stress, &mut MemeticDisassociation, &mut Skills), With<EngramPurchaseIntent>>,
    mut commands: Commands,
) {
    for (entity, mut stress, mut disassociation, mut skills) in query.iter_mut() {
        // Reduce stress significantly
        stress.level = (stress.level - 40.0).max(0.0);

        // Increase disassociation
        disassociation.level += 20.0;

        // Degrade real skills due to memory overwrite
        skills.engineering = (skills.engineering - 10.0).max(0.0);

        // Remove intent after purchase
        commands.entity(entity).remove::<EngramPurchaseIntent>();
    }
}

pub fn process_job_execution(
    mut query: Query<(&MemeticDisassociation, &mut JobAssignment)>,
) {
    for (disassociation, mut job) in query.iter_mut() {
        if disassociation.level >= 100.0 {
            job.active = false;
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- The flat `-40.0` stress and `+20.0` disassociation should be tuned or driven by the quality/cost of the engram.
- `Skills` component should probably use a generic map rather than hardcoding `engineering` to support all job types.
- The intent flag `EngramPurchaseIntent` is simple but should be triggered by a broader pop desire/action system.

## Acceptance Criteria (Testable!)

- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Purchasing memory engrams correctly decreases stress and increases disassociation.
- [ ] High disassociation correctly interferes with job execution.

## Technical Guidance

- Integration points: Hook this up to the existing pop needs/stress AI so pops actively seek out smugglers when stress is high.
- The smuggler entity/network needs to exist on the map or as a colony modifier.

## Questions
*Builder: add questions here if spec is unclear.*

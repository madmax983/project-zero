# Specification: The Fractal Bureaucracy

## 1. Overview
The Fractal Bureaucracy introduces a space-consuming administrative mechanic where Layer 3 edicts require complex, fractal building chains on Layer 1 colonies. Each "Administration Hub" necessitates adjacent "Sub-Offices", which require "Archival Nodes". Disrupting this physical chain causes a colony-wide "Logic Cascade", severely impacting resource management.

## 2. Dependencies
- Layer 3 (Edicts and Administrative Capacity)
- Layer 1 (Building placement, spatial adjacencies)
- Layer 1 (Pop statuses and jobs)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use super::*;

    #[derive(Component)]
    struct AdminHub;

    #[derive(Component)]
    struct SubOffice;

    #[derive(Component)]
    struct ArchivalNode;

    #[derive(Component)]
    struct BuildingLocation(i32, i32);

    #[derive(Component)]
    struct LogicCascadeEvent;

    #[derive(Component)]
    struct Pop;

    #[derive(Component)]
    struct PopStatus {
        is_confused: bool,
    }

    // The system to test
    fn fractal_bureaucracy_validation_system(
        mut commands: Commands,
        hubs: Query<(Entity, &BuildingLocation), With<AdminHub>>,
        offices: Query<(Entity, &BuildingLocation), With<SubOffice>>,
        nodes: Query<(Entity, &BuildingLocation), With<ArchivalNode>>,
        mut pops: Query<&mut PopStatus, With<Pop>>,
    ) {
        // Implementation goes here
    }

    fn spawn_test_world() -> World {
        World::new()
    }

    #[test]
    fn test_logic_cascade_on_broken_fractal() {
        let mut world = spawn_test_world();

        let pop_entity = world.spawn().id();
        world.entity_mut(pop_entity).insert((Pop, PopStatus { is_confused: false }));

        // Spawn a Hub without the required Sub-Offices and Nodes
        let hub_entity = world.spawn().id();
        world.entity_mut(hub_entity).insert((AdminHub, BuildingLocation(0, 0)));

        // Act
        let mut schedule = Schedule::default();
        schedule.add_systems(fractal_bureaucracy_validation_system);
        schedule.run(&mut world);

        // Assert: Pop should be confused due to broken chain
        let status = world.get::<PopStatus>(pop_entity).unwrap();
        assert!(status.is_confused, "Broken fractal chain must cause Logic Cascade (confusion) on Pops");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
fn fractal_bureaucracy_validation_system(
    mut commands: Commands,
    hubs: Query<(Entity, &BuildingLocation), With<AdminHub>>,
    offices: Query<(Entity, &BuildingLocation), With<SubOffice>>,
    nodes: Query<(Entity, &BuildingLocation), With<ArchivalNode>>,
    mut pops: Query<&mut PopStatus, With<Pop>>,
) {
    for (_, _hub_loc) in hubs.iter() {
        // Minimal logic: if there are no sub-offices, trigger cascade
        if offices.iter().count() == 0 {
            for mut status in pops.iter_mut() {
                status.is_confused = true;
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design

- **Refactoring:** The validation system should check strict adjacency rules (e.g., `manhattan_distance == 1` between Hub and Sub-Office) rather than just global existence.
- **Code Smells:** Emitting events for `LogicCascade` is preferable to mutating `PopStatus` directly in the validation system. A separate system should listen to `LogicCascadeEvent` and update Pops.
- **Performance:** Use spatial hashing or a grid query method to find adjacent buildings rather than O(N^2) checking all offices against all hubs.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for the new code.
- [ ] Breaking adjacency requirements for the Hub->Office->Node chain correctly applies the `Confused` state to Pops.

## 7. Technical Guidance
- Place logic in `layer1/buildings/bureaucracy.rs` or similar.
- Consider bridging Layer 3 edict load requirements via a global resource that triggers the need for these Hubs.

## 8. Questions
*Builder: add questions here if spec is unclear.*

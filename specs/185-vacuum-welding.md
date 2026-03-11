# 185: Vacuum Welding

## 1. Overview

**Vacuum Welding** is a phenomenon where clean metal surfaces in a vacuum fuse together instantly upon contact. In the simulation, this mechanic creates a trade-off for building in vacuum environments (outside pressurized zones).

*   **Benefit:** Buildings constructed in a vacuum (Pressure < 0.1) gain **+100% Max HP** due to the perfect fusion of materials.
*   **Drawback:** These buildings become **Permanent**. They cannot be Deconstructed (to reclaim resources) or Repaired (fused solid).
*   **Removal:** To remove a Vacuum Welded building, the player must use a new **Destroy** designation. This destroys the building utterly, yielding **0 resources**.

This forces players to plan their external infrastructure carefully. A hasty defense wall built in vacuum is stronger, but if it blocks future expansion, you have to blow it up (wasting the materials).

## 2. Dependencies

*   `006` Building Placement (for `try_place_building`)
*   `045` Structure Durability (for `Structure` component)
*   `063` Atmospheric Simulation (for `PressureGrid`)
*   `016` Utility AI (for designation handling)

## 3. RED Phase: Tests First

These tests verify the core mechanics of Vacuum Welding.

```rust
#[cfg(test)]
mod tests {
    use crate::layer1::building::{Building, BuildingType, try_place_building, VacuumWelded};
    use crate::layer1::designation::{DesignationType, can_designate};
    use crate::layer1::pressure::PressureGrid;
    use crate::layer1::structure::Structure;
    use crate::layer1::GridPosition;
    use bevy_ecs::prelude::*;

    // Helper to setup world with specific pressure
    fn setup_world_with_pressure(pressure: f32) -> World {
        let mut world = World::new();
        // ... Standard setup (Terrain, Resources, etc) ...
        // Insert PressureGrid
        let mut pressure_grid = PressureGrid::new(10, 10);
        pressure_grid.set_all(pressure);
        world.insert_resource(pressure_grid);
        world
    }

    #[test]
    fn test_vacuum_welding_applied_in_vacuum() {
        // Arrange: World with 0.0 pressure
        let mut world = setup_world_with_pressure(0.0);

        // Act: Place a Wall
        let success = try_place_building(&mut world, 5, 5, BuildingType::Wall);
        assert!(success);

        // Assert: Building has VacuumWelded component
        let entity = world.query_filtered::<Entity, With<Building>>().single(&world);
        assert!(world.get::<VacuumWelded>(entity).is_some(), "Building in vacuum should be welded");
    }

    #[test]
    fn test_vacuum_welding_not_applied_in_atmosphere() {
        // Arrange: World with 1.0 pressure
        let mut world = setup_world_with_pressure(1.0);

        // Act: Place a Wall
        let success = try_place_building(&mut world, 5, 5, BuildingType::Wall);
        assert!(success);

        // Assert: Building does NOT have VacuumWelded component
        let entity = world.query_filtered::<Entity, With<Building>>().single(&world);
        assert!(world.get::<VacuumWelded>(entity).is_none(), "Building in atmo should not be welded");
    }

    #[test]
    fn test_vacuum_welding_hp_bonus() {
        // Arrange: Vacuum world
        let mut world = setup_world_with_pressure(0.0);

        // Act: Place Wall
        try_place_building(&mut world, 5, 5, BuildingType::Wall);

        // Assert: HP is double the standard
        let entity = world.query_filtered::<Entity, With<Building>>().single(&world);
        let structure = world.get::<Structure>(entity).unwrap();

        // Standard Wall HP is 50.0 * MaterialMod (assume Metal=3.0 -> 150.0)
        // We expect double that.
        // For robustness, compare against a control building in atmosphere.

        let vacuum_hp = structure.max_hp;

        // Control
        let mut control_world = setup_world_with_pressure(1.0);
        try_place_building(&mut control_world, 5, 5, BuildingType::Wall);
        let control_entity = control_world.query_filtered::<Entity, With<Building>>().single(&control_world);
        let control_hp = control_world.get::<Structure>(control_entity).unwrap().max_hp;

        assert!((vacuum_hp - (control_hp * 2.0)).abs() < f32::EPSILON, "Vacuum building should have 2x HP");
    }

    #[test]
    fn test_vacuum_welded_prevents_demolish() {
        // Arrange: Welded building
        let mut world = setup_world_with_pressure(0.0);
        try_place_building(&mut world, 5, 5, BuildingType::Wall);
        let entity = world.query_filtered::<Entity, With<Building>>().single(&world);

        // Act/Assert: Check can_designate for Demolish
        // We must ensure the tile is marked occupied for can_designate to reach the entity check
        let can_demolish = can_designate(&world, 5, 5, DesignationType::Demolish);
        assert!(!can_demolish, "Should not be able to designate Demolish on welded building");
    }

    #[test]
    fn test_vacuum_welded_prevents_repair() {
        // Arrange: Welded building
        let mut world = setup_world_with_pressure(0.0);
        try_place_building(&mut world, 5, 5, BuildingType::Wall);

        // Act/Assert: Check can_designate for Repair
        let can_repair = can_designate(&world, 5, 5, DesignationType::Repair);
        assert!(!can_repair, "Should not be able to designate Repair on welded building");
    }

    #[test]
    fn test_destroy_designation_allowed() {
        // Arrange: Welded building
        let mut world = setup_world_with_pressure(0.0);
        try_place_building(&mut world, 5, 5, BuildingType::Wall);

        // Act/Assert: Check can_designate for Destroy (New type)
        let can_destroy = can_designate(&world, 5, 5, DesignationType::Destroy);
        assert!(can_destroy, "Should be able to designate Destroy on welded building");
    }

    #[test]
    fn test_execute_destroy_yields_no_resources() {
        // Arrange: Welded building with Destroy designation
        let mut world = setup_world_with_pressure(0.0);
        try_place_building(&mut world, 5, 5, BuildingType::Wall);
        let building_entity = world.query_filtered::<Entity, With<Building>>().single(&world);

        let designation = world.spawn((
            crate::layer1::designation::Designation { designation_type: DesignationType::Destroy },
            GridPosition { x: 5, y: 5 }
        )).id();

        // Act: Execute the destruction (simulate worker finishing job)
        // Use the new `execute_destroy` function in execution layer
        let result = crate::layer1::execution::execute_destroy(&mut world, designation);
        assert!(result);

        // Assert: Building gone, No ResourceItems spawned
        assert!(world.get_entity(building_entity).is_err());
        let resource_count = world.query::<&crate::layer1::resources::ResourceItem>().iter(&world).count();
        assert_eq!(resource_count, 0, "Destroy should yield no resources");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

### 1. `src/layer1/building.rs`
*   Define `VacuumWelded` component (marker).
*   Update `try_place_building`:
    *   Get `PressureGrid` resource.
    *   Check pressure at `(x, y)`. If `< 0.1` (Vacuum), insert `VacuumWelded` component on the spawned entity.
    *   If `VacuumWelded`, multiply `Structure.max_hp` and `current_hp` by `2.0`.

### 2. `src/layer1/designation.rs`
*   Add `Destroy` variant to `DesignationType` enum.
    *   Char: `'D'` (or similar).
    *   Label: `"Destroy"`.
*   Update `can_designate`:
    *   `Demolish`: Retrieve entity at pos. If it has `VacuumWelded`, return `false`.
    *   `Repair`: Retrieve entity at pos. If it has `VacuumWelded`, return `false`.
    *   `Destroy`: Valid if tile is occupied (similar to Demolish logic, but allows Welded).

### 3. `src/layer1/execution.rs`
*   Implement `execute_destroy(world, designation_entity) -> bool`.
    *   Logic similar to `execute_demolish` but **skips** the resource scavenging step (no `Ruin` check, no `ResourceItem` spawn).
    *   Despawns the building immediately.
    *   Spawns particle effect (Red X or dust).
    *   Removes from `OccupiedTiles`.
*   Update `execute_work_on_designation` to handle `DesignationType::Destroy`.
*   Add `get_skill_for_designation`: `Destroy` -> `SkillType::Construction` (same as Demolish).

## 5. REFACTOR Phase

*   **UI Feedback:** Ensure the UI (when implemented) shows *why* Demolish is disabled (e.g., "Vacuum Welded: Permanent"). This is out of scope for the backend spec but good to note.
*   **Job Priority:** Destroy jobs should probably have similar priority to Demolish.
*   **Logic Reuse:** `execute_destroy` and `execute_demolish` share code (despawning, particle, occupied update). Extract a common `remove_building(entity, yield_resources: bool)` helper?

## 6. Acceptance Criteria

- [ ] `test_vacuum_welding_applied_in_vacuum` passes.
- [ ] `test_vacuum_welding_not_applied_in_atmosphere` passes.
- [ ] `test_vacuum_welding_hp_bonus` passes.
- [ ] `test_vacuum_welding_prevents_demolish` passes.
- [ ] `test_vacuum_welding_prevents_repair` passes.
- [ ] `test_destroy_designation_behavior` passes.
- [ ] New `DesignationType::Destroy` is integrated.

## 7. Technical Guidance

*   **Pressure Check:** `PressureGrid` stores `f32`. Vacuum is `0.0`, but check `< 0.1` for floating point safety.
*   **Component Query:** In `can_designate`, checking components requires iterating entities or using a spatial map. `OccupiedTiles` only gives coordinates. Use `world.iter_entities()` filter or `BuildingMap` if available (it is available as a resource `BuildingMap`). Ideally use `BuildingMap` for O(1) lookup if `can_designate` is called frequently.
*   **Resource Drop:** `execute_demolish` calls `ruins::process_scavenge` or spawns debris. `execute_destroy` should skip this entirely.

## 8. Questions

*   **Does this apply to all buildings?** Yes, anything with `Structure`.
*   **Can I destroy non-welded buildings with Destroy?** Yes, if you hate resources. It's a "fast delete" option essentially.
  - *Architect:* Yes, this serves as a fast delete option that foregoes resource recovery.

*Architect:* Yes, this applies to all entities with a `Structure` component.

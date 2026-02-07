# 045: Structure Durability & Repair

## Overview

Buildings currently exist in a binary state: perfect or destroyed (via fire/demolish). This spec introduces **Structure Durability** (Hit Points) to buildings.

- Buildings gain a `Structure` component with `current_hp` and `max_hp`.
- Fire now deals damage over time rather than instantly destroying buildings at the end of its lifetime.
- A new **Repair** designation allows workers to restore building HP.

## Dependencies

- `033` — Fire Propagation (Damage source)
- `021` — Utility AI Work (ActionType::Work foundation)
- `017` — Designation System (DesignationType)

## RED Phase: Tests First

Write these tests in `src/layer1/structure_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::structure::{Structure, fire_damage_structure_system, repair_system};
    use crate::layer1::fire::{Fire, Flammable};
    use crate::layer1::GridPosition;
    use crate::layer1::designation::{Designation, DesignationType};
    use crate::layer1::utility_ai::ActionType;

    #[test]
    fn test_structure_component_defaults() {
        let s = Structure::default();
        assert!(s.max_hp > 0.0);
        assert_eq!(s.current_hp, s.max_hp);
    }

    #[test]
    fn test_fire_damages_structure() {
        let mut world = World::new();

        // Spawn a building with Structure and Flammable
        let building = world.spawn((
            Building { building_type: BuildingType::Housing },
            Structure { current_hp: 100.0, max_hp: 100.0 },
            Flammable::default(),
            GridPosition { x: 0, y: 0 },
        )).id();

        // Spawn Fire on top of it
        world.spawn((
            Fire { intensity: 1.0, lifetime: 10 },
            GridPosition { x: 0, y: 0 },
        ));

        // Run damage system
        fire_damage_structure_system(&mut world);

        // Check HP reduced
        let structure = world.get::<Structure>(building).unwrap();
        assert!(structure.current_hp < 100.0);
        assert!(structure.current_hp > 0.0); // Should not be instant kill
    }

    #[test]
    fn test_structure_destruction_at_zero_hp() {
        let mut world = World::new();

        let building = world.spawn((
            Building { building_type: BuildingType::Housing },
            Structure { current_hp: 1.0, max_hp: 100.0 },
            Flammable::default(),
            GridPosition { x: 0, y: 0 },
        )).id();

        // Fire deals damage > 1.0
        world.spawn((
            Fire { intensity: 10.0, lifetime: 10 }, // High intensity
            GridPosition { x: 0, y: 0 },
        ));

        // Mock damage system to ensure it kills
        // (In real impl, fire_damage_structure_system handles this)
        fire_damage_structure_system(&mut world);

        assert!(world.get_entity(building).is_err());
    }

    #[test]
    fn test_repair_restores_hp() {
        let mut world = World::new();

        let building = world.spawn((
            Building { building_type: BuildingType::Housing },
            Structure { current_hp: 50.0, max_hp: 100.0 },
            GridPosition { x: 0, y: 0 },
        )).id();

        // Designation for Repair
        let designation = world.spawn((
            Designation { designation_type: DesignationType::Repair },
            GridPosition { x: 0, y: 0 },
        )).id();

        // Worker performing Repair
        // (This would be handled by work_execution_system calling repair logic)
        // Here we test the repair logic directly if exposed, or the system

        // For unit test, let's call a repair helper function or system
        // simulating work being done.
        // Let's assume `process_repair(world, designation_entity, amount)`

        crate::layer1::structure::process_repair(&mut world, designation, 10.0);

        let structure = world.get::<Structure>(building).unwrap();
        assert_eq!(structure.current_hp, 60.0);
    }

    #[test]
    fn test_repair_removes_designation_at_max_hp() {
        let mut world = World::new();

        let building = world.spawn((
            Building { building_type: BuildingType::Housing },
            Structure { current_hp: 95.0, max_hp: 100.0 },
            GridPosition { x: 0, y: 0 },
        )).id();

        let designation = world.spawn((
            Designation { designation_type: DesignationType::Repair },
            GridPosition { x: 0, y: 0 },
        )).id();

        crate::layer1::structure::process_repair(&mut world, designation, 10.0);

        // HP capped at max
        let structure = world.get::<Structure>(building).unwrap();
        assert_eq!(structure.current_hp, 100.0);

        // Designation should be despawned
        assert!(world.get_entity(designation).is_err());
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Create `Structure` Component

In `src/layer1/structure.rs`:

```rust
use bevy_ecs::prelude::*;

#[derive(Component, Debug, Clone, Copy)]
pub struct Structure {
    pub current_hp: f32,
    pub max_hp: f32,
}

impl Default for Structure {
    fn default() -> Self {
        Self {
            current_hp: 100.0,
            max_hp: 100.0,
        }
    }
}
```

### 2. Update `Building` Bundle

In `src/layer1/building.rs`, add `Structure` to the bundle or spawn logic.

### 3. Implement Fire Damage Logic

Modify or create `fire_damage_structure_system` in `src/layer1/structure.rs` (or `fire.rs`):

```rust
pub fn fire_damage_structure_system(world: &mut World) {
    // 1. Identify fires
    // 2. Identify structures at fire locations
    // 3. Apply damage based on fire intensity (e.g., 5.0 per tick)
    // 4. If HP <= 0, despawn structure (and maybe spawn "Ruins" or "Rubble")
}
```

*Note: You may need to refactor existing `fire_damage_system` in `fire.rs` to stop it from instantly destroying buildings, or integrate this logic there.*

### 4. Implement Repair Logic

Add `Repair` to `DesignationType` in `src/layer1/designation.rs`.

In `src/layer1/execution.rs`:
- Handle `DesignationType::Repair` in `work_execution_system`.
- Call `process_repair`.

In `src/layer1/structure.rs`:

```rust
pub fn process_repair(world: &mut World, designation_entity: Entity, amount: f32) {
    // Find building at designation position
    // Increase HP
    // If HP >= Max, despawn designation
}
```

### 5. Update Utility AI

In `src/layer1/utility_ai.rs`:
- Add `ActionType::Repair`.
- Update `evaluate_actions` to score `Repair` actions (check for repair designations).

## REFACTOR Phase: Quality & Design

- **Visuals**: Buildings with < 50% HP could use a different character or color (future).
- **Cost**: Repairing should eventually cost resources (Wood/Stone), proportional to damage. For now, it's free (labor only).
- **Rubble**: Destroyed buildings should leave `Rubble` (TerrainType or Entity) that needs clearing.
- **Fire Integration**: Ensure `fire_damage_system` doesn't double-destroy (once by lifetime, once by HP). Fire burning out shouldn't destroy the building anymore, only the *damage* should.

## Acceptance Criteria

- [ ] `Structure` component added to buildings.
- [ ] Fire deals damage over time instead of instant kill.
- [ ] Buildings destroyed when HP reaches 0.
- [ ] `DesignationType::Repair` allows workers to fix buildings.
- [ ] Tests pass.

## Technical Guidance

- Be careful with `fire_damage_system` in `src/layer1/fire.rs`. It currently destroys *any* flammable entity when fire dies. You must change this behavior for `Structure` entities. They should survive the fire if HP > 0.
- Use `world.query::<(&GridPosition, &mut Structure)>()` to find damaged buildings efficiently.

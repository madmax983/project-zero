# 100: Jury-Rigging

## Overview

In desperate times, proper repairs are a luxury. **Jury-Rigging** allows players to instantly patch up damaged buildings without consuming resources (like Wood or Metal). However, this comes at a cost: the building gains the **Fragile** trait, making it significantly more vulnerable to future damage and increasing its likelihood of catastrophic failure.

This feature reinforces the "Entropy & Spoilage" theme, forcing players to choose between immediate survival and long-term stability.

## Dependencies

- `src/layer1/structure.rs` — Structure system (HP, Damage).
- `src/layer1/designation.rs` — Designation system (for the new tool).
- `src/layer1/building.rs` — Building entity structure.

## RED Phase: Tests First

Write these tests in `src/layer1/structure_jury_rig_tests.rs` (or append to `structure.rs`). They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::GridPosition;
    use crate::layer1::structure::{Structure, Fragile, process_jury_rig, fire_damage_structure_system};
    use crate::layer1::fire::Fire;
    use crate::layer1::designation::{Designation, DesignationType};

    fn setup_world() -> World {
        let mut world = World::new();
        // Register components
        // Assuming Structure, Fire, Designation are registered by systems usually
        world
    }

    #[test]
    fn test_jury_rig_restores_hp_instantly() {
        let mut world = setup_world();
        let building = world.spawn((
            Structure { current_hp: 10.0, max_hp: 100.0 },
            GridPosition { x: 0, y: 0 },
        )).id();

        // Perform Jury-Rig
        // Note: Jury-Rigging is instant, unlike Repair which is work-based.
        // It might still require a Pop to "do" it, but for the mechanics test,
        // we test the effect of the function `process_jury_rig`.
        process_jury_rig(&mut world, building);

        let structure = world.get::<Structure>(building).unwrap();
        // Should be fully healed? Or just functional (e.g., 50%)?
        // Spec decision: Fully healed for function, but Fragile.
        assert_eq!(structure.current_hp, 100.0);
    }

    #[test]
    fn test_jury_rig_adds_fragile_component() {
        let mut world = setup_world();
        let building = world.spawn((
            Structure { current_hp: 10.0, max_hp: 100.0 },
            GridPosition { x: 0, y: 0 },
        )).id();

        process_jury_rig(&mut world, building);

        let fragile = world.get::<Fragile>(building);
        assert!(fragile.is_some(), "Jury-Rigging should add Fragile component");
        assert_eq!(fragile.unwrap().stacks, 1);
    }

    #[test]
    fn test_jury_rig_stacks_fragility() {
        let mut world = setup_world();
        let building = world.spawn((
            Structure { current_hp: 10.0, max_hp: 100.0 },
            Fragile { stacks: 1 },
            GridPosition { x: 0, y: 0 },
        )).id();

        process_jury_rig(&mut world, building);

        let fragile = world.get::<Fragile>(building).unwrap();
        assert_eq!(fragile.stacks, 2, "Subsequent jury-rigging should increase fragility");
    }

    #[test]
    fn test_fragile_buildings_take_extra_fire_damage() {
        let mut world = setup_world();

        // Normal Building
        let normal = world.spawn((
            Structure { current_hp: 100.0, max_hp: 100.0 },
            GridPosition { x: 0, y: 0 },
            // Needs Flammable? Assuming fire system checks it or just position.
            // Let's assume generic damage testing or fire system specific.
            // If strictly testing fire system:
            crate::layer1::fire::Flammable::default(),
        )).id();

        // Fragile Building (1 stack)
        let fragile = world.spawn((
            Structure { current_hp: 100.0, max_hp: 100.0 },
            Fragile { stacks: 1 },
            GridPosition { x: 1, y: 0 },
            crate::layer1::fire::Flammable::default(),
        )).id();

        // Spawn Fire at both locations with same intensity
        world.spawn((Fire { intensity: 1.0, lifetime: 10 }, GridPosition { x: 0, y: 0 }));
        world.spawn((Fire { intensity: 1.0, lifetime: 10 }, GridPosition { x: 1, y: 0 }));

        // Run Fire Damage System
        fire_damage_structure_system(&mut world);

        let hp_normal = world.get::<Structure>(normal).unwrap().current_hp;
        let hp_fragile = world.get::<Structure>(fragile).unwrap().current_hp;

        // Fragile should have taken MORE damage (lower HP remaining)
        assert!(hp_fragile < hp_normal, "Fragile building should take more damage. Normal: {}, Fragile: {}", hp_normal, hp_fragile);
    }

    #[test]
    fn test_designation_type_jury_rig_properties() {
        assert_eq!(DesignationType::JuryRig.char(), 'J'); // or similar
        assert_eq!(DesignationType::JuryRig.label(), "Jury-Rig");
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Update `DesignationType` (`src/layer1/designation.rs`)

```rust
// Add variant
pub enum DesignationType {
    // ...
    JuryRig,
}

// Update impls
match self {
    // ...
    Self::JuryRig => 'J', // char
    Self::JuryRig => "Jury-Rig", // label
}

// Update can_designate
match designation_type {
    // ...
    DesignationType::JuryRig => {
        // Can only jury-rig if occupied (building) AND structure is damaged?
        // Or just occupied.
        let occupied = world.resource::<OccupiedTiles>();
        occupied.0.contains(&(x, y))
    }
}
```

### 2. Create `Fragile` Component (`src/layer1/structure.rs`)

```rust
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Fragile {
    pub stacks: u32,
}
```

### 3. Implement `process_jury_rig` (`src/layer1/structure.rs`)

```rust
/// Instantly repairs a structure but adds fragility.
pub fn process_jury_rig(world: &mut World, structure_entity: Entity) {
    // 1. Fully heal
    if let Some(mut structure) = world.get_mut::<Structure>(structure_entity) {
        structure.current_hp = structure.max_hp;
    }

    // 2. Add/Increment Fragile
    if let Some(mut fragile) = world.get_mut::<Fragile>(structure_entity) {
        fragile.stacks += 1;
    } else {
        world.entity_mut(structure_entity).insert(Fragile { stacks: 1 });
    }

    // 3. Remove Designation (if any exists on this tile)
    // (Optional logic to find and remove designation at entity pos)
}
```

### 4. Update Damage Logic (`src/layer1/structure.rs`)

In `fire_damage_structure_system` (and any other damage systems):

```rust
for (entity, pos, mut structure, maybe_fragile) in structure_query.iter_mut(world) {
    if *pos == fire_pos {
        let base_damage = 5.0 * intensity;

        // Apply Fragility Multiplier
        // e.g., +50% damage per stack
        let multiplier = if let Some(fragile) = maybe_fragile {
            1.0 + (fragile.stacks as f32 * 0.5)
        } else {
            1.0
        };

        let final_damage = base_damage * multiplier;
        structure.current_hp -= final_damage;
        // ... destruction check ...
    }
}
```

Make sure to update the query to include `Option<&Fragile>`.

## REFACTOR Phase: Quality & Design

- **Visuals**: Render `Fragile` buildings with a "duct tape" or "sparking" overlay in `src/ui/map.rs`.
- **Decay**: Add a `fragile_decay_system` that slowly damages `Fragile` buildings over time (random chance), simulating the poor quality of the repair.
- **Proper Repair**: Implement a "Proper Repair" action (using resources) that *removes* `Fragile` stacks one by one.
- **Tooltips**: Update inspection UI to show "Fragile (x3)" status.

## Acceptance Criteria

- [ ] `DesignationType::JuryRig` is available.
- [ ] `Fragile` component exists and tracks stacks.
- [ ] `process_jury_rig` restores 100% HP and increments stacks.
- [ ] Fire damage (and other damage sources) is multiplied by `1.0 + (0.5 * stacks)` for Fragile buildings.
- [ ] Tests in RED phase pass.
- [ ] Code compiles with no warnings.

## Technical Guidance

- Be careful with the `fire_damage_structure_system` query. Adding `Option<&Fragile>` is safe.
- Ensure `can_designate` for `JuryRig` checks that a building actually exists (via `OccupiedTiles` or `Structure` component query).
- `process_jury_rig` is the *mechanic*. The *Action* that triggers it needs to be hooked up in `work_execution_system` or similar if it requires a Pop to walk there.
  - **For MVP**: Treat it as an "Instant Edict" (God Power) or a "0-work" designation that is processed immediately by a system, OR hook it into `work_execution` as a very fast job.
  - **Recommendation**: Hook it into `work_execution.rs`. It's a job. A pop goes there, plays an animation, and `process_jury_rig` is called.

## Questions

- *Builder*: Should Jury-Rigging consume *any* time? (Yes, assumed small amount).
- *Architect:* Yes, assumed small amount.
- *Builder*: Does Fragility ever go away? (Only with "Proper Repair" - future feature).
- *Architect:* Only with "Proper Repair" - future feature.

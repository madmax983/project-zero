# 089: Material Provenance

## Overview

Currently, a "Wall" is just a Wall, and a "House" is just a House. This spec introduces **Material Provenance**, allowing buildings to be constructed from different materials (Wood, Stone, Metal). The material choice affects the building's properties:
- **Flammability**: Wood burns, Stone doesn't.
- **Beauty**: Stone/Metal might be prettier or uglier than Wood.
- **HP**: Stone walls are stronger.

This adds depth to the economy (using stone for critical infrastructure) and aesthetics.

## Dependencies

- `006` — Building Placement (Core)
- `033` — Fire Propagation (Flammability)
- `071` — Structural Integrity (HP)
- `044` — Beauty (Beauty stats)

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/layer1/material_provenance_tests.rs

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::building::{Building, BuildingType, Material, MaterialType};
    use crate::layer1::fire::Flammable;
    use crate::layer1::structure::Structure;
    use crate::layer1::beauty::BeautySource;

    fn setup_world() -> World {
        let mut world = World::new();
        // Setup necessary resources
        world
    }

    #[test]
    fn test_spawn_wood_wall() {
        let mut world = setup_world();

        // Spawn a Wall made of Wood
        // Note: This implies spawn_building signature changes or we use a builder pattern
        // For RED phase, we assume a helper or modified spawn function exists.
        crate::layer1::building::spawn_building_with_material(
            &mut world,
            0, 0,
            BuildingType::Wall,
            MaterialType::Wood
        );

        let (building, material, flammable, structure) = world
            .query::<(&Building, &Material, &Flammable, &Structure)>()
            .single(&world);

        assert_eq!(material.0, MaterialType::Wood);
        assert!(flammable.is_flammable); // Wood burns
        assert_eq!(structure.max_hp, 50.0); // Wood wall HP
    }

    #[test]
    fn test_spawn_stone_wall() {
        let mut world = setup_world();

        crate::layer1::building::spawn_building_with_material(
            &mut world,
            0, 0,
            BuildingType::Wall,
            MaterialType::Stone
        );

        let (building, material, flammable, structure) = world
            .query::<(&Building, &Material, &Flammable, &Structure)>()
            .single(&world);

        assert_eq!(material.0, MaterialType::Stone);
        assert!(!flammable.is_flammable); // Stone doesn't burn
        assert_eq!(structure.max_hp, 200.0); // Stone wall HP
    }

    #[test]
    fn test_material_beauty_modifier() {
        let mut world = setup_world();

        // Spawn Stone Statue vs Metal Statue
        crate::layer1::building::spawn_building_with_material(
            &mut world,
            0, 0,
            BuildingType::Statue,
            MaterialType::Stone
        );

        let beauty = world.query::<&BeautySource>().single(&world);
        // Base statue might be 10. Stone modifier +0?
        let stone_beauty = beauty.value;

        world.despawn(world.entities().iter().next().unwrap()); // Clear

        crate::layer1::building::spawn_building_with_material(
            &mut world,
            0, 0,
            BuildingType::Statue,
            MaterialType::Gold // If we have Gold
        );

        let beauty = world.query::<&BeautySource>().single(&world);
        assert!(beauty.value > stone_beauty);
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Define Material Types

`src/layer1/building.rs`

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MaterialType {
    #[default]
    Wood,
    Stone,
    Metal,
    Gold, // Fancy
}

#[derive(Component, Default)]
pub struct Material(pub MaterialType);

impl MaterialType {
    pub fn flammability(&self) -> bool {
        match self {
            Self::Wood => true,
            _ => false,
        }
    }

    pub fn hp_modifier(&self) -> f32 {
        match self {
            Self::Wood => 1.0,
            Self::Stone => 4.0,
            Self::Metal => 3.0,
            Self::Gold => 0.5, // Soft
        }
    }

    pub fn beauty_modifier(&self) -> f32 {
        match self {
            Self::Wood => 0.0,
            Self::Stone => 1.0,
            Self::Metal => 0.0,
            Self::Gold => 10.0,
        }
    }
}
```

### 2. Update `spawn_building`

Refactor `spawn_building` to accept `MaterialType`.

```rust
// src/layer1/building.rs

pub fn spawn_building(
    world: &mut World,
    x: i32,
    y: i32,
    building_type: BuildingType,
    material: MaterialType // New Argument
) {
    let mut entity = world.spawn((
        Building { building_type },
        GridPosition { x, y },
        Material(material), // Add component
    ));

    // Calculate HP based on material
    let base_hp = 50.0; // Simplify for Green
    let max_hp = base_hp * material.hp_modifier();

    entity.insert(Structure {
        current_hp: max_hp,
        max_hp,
        ..Default::default()
    });

    // Flammability
    if material.flammability() {
        entity.insert(Flammable::default());
    }
    // Note: If previously hardcoded Flammable, remove it and rely on this.

    // Beauty
    let base_beauty = building_type.beauty_value();
    let final_beauty = base_beauty + material.beauty_modifier();
    if final_beauty != 0.0 {
        entity.insert(BeautySource { value: final_beauty, ..Default::default() });
    }

    // ... existing component logic (Housing, Farm, etc) ...
}
```

### 3. Update Call Sites

Update `try_place_building` to default to `Wood` or accept a selected material from `BuildMode`.
For GREEN phase, just defaulting to `Wood` (or existing hardcoded types) is fine, but updating `BuildMode` to store `selected_material` is better.

```rust
// src/layer1/building.rs

#[derive(Resource, Default)]
pub struct BuildMode {
    pub active: bool,
    pub cursor: GridPosition,
    pub selected: BuildingType,
    pub selected_material: MaterialType, // NEW
}

// Update try_place_building to use this
```

## REFACTOR Phase: Quality & Design

- **UI**: Update the Build Menu to allow cycling materials (e.g. `Tab` for building, `Shift+Tab` for material).
- **Costs**: `BuildingType::cost()` currently returns fixed costs. It needs to depend on material.
    - Refactor `cost()` to `cost(material: MaterialType)`.
    - Wood Wall = 5 Wood.
    - Stone Wall = 5 Stone.
- **Visuals**: Buildings need different sprites/colors based on material (e.g. Brown for Wood, Grey for Stone).
- **Audio**: Different construction sounds.

## Acceptance Criteria

- [ ] `Material` component exists.
- [ ] Buildings spawn with correct HP/Flammability based on material.
- [ ] `try_place_building` uses the selected material.
- [ ] Tests pass.

## Technical Guidance

- This is a breaking change for `spawn_building`. You will need to update all tests that call it.
- `BuildingType::cost()` is used in many places. Changing its signature will require updating `resources.rs` checks.
- Keep it simple: Start with `Wall` and `Housing` supporting multiple materials. Machines (Smelter) might be fixed material (Stone/Metal).

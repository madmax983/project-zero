# Extending the Game: Adding a New Building

This guide explains how to add a new building type to SCALE.

## Overview

Buildings are defined in `src/layer1/building.rs`, but their visuals are decoupled in `src/ui/map.rs`.

## Steps

### 1. Add the Variant

Open `src/layer1/building.rs` and add your new building to `BuildingType`.

```rust
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug, EnumIter)]
pub enum BuildingType {
    #[default]
    Housing,
    Farm,
    Stockpile,
    Statue, // <--- Your new building
}
```

> **Note:** The `EnumIter` derive macro automatically handles the cycling logic in build mode. You do NOT need to manually update `next()`.

### 2. Implement Data Properties

Update the `impl BuildingType` block in `src/layer1/building.rs` to define the data properties.

#### Label
Add a case to `label()`:

```rust
pub const fn label(&self) -> &'static str {
    match self {
        Self::Housing => "Housing",
        Self::Farm => "Farm",
        Self::Stockpile => "Stockpile",
        Self::Statue => "Statue", // <--- Your label
    }
}
```

#### Cost
Add a case to `cost()`:

```rust
pub fn cost(&self) -> ColonyResources {
    match self {
        Self::Housing => ColonyResources {
            wood: 10.0,
            ..Default::default()
        },
        // ...
        Self::Statue => ColonyResources {
            wood: 50.0,
            stone: 20.0,
            ..Default::default()
        },
    }
}
```

### 3. Implement Visuals

Visuals are decoupled from the simulation. Open `src/ui/map.rs` to define how your building looks.

#### Character
Add a case to `get_building_char()`:

```rust
pub const fn get_building_char(building: BuildingType) -> char {
    match building {
        BuildingType::Housing => '⌂',
        BuildingType::Farm => '♣',
        BuildingType::Stockpile => '≡',
        BuildingType::Statue => '¥', // <--- Your char
    }
}
```

#### Color
Add a case to `get_building_color()`:

```rust
pub const fn get_building_color(building: BuildingType) -> Color {
    match building {
        BuildingType::Housing => Color::Rgb(139, 90, 43),
        BuildingType::Farm => Color::Rgb(218, 165, 32),
        BuildingType::Stockpile => Color::Rgb(169, 169, 169),
        BuildingType::Statue => Color::Gray, // <--- Your color
    }
}
```

### 4. Implement Logic (Optional)

If your building has specific logic (like producing resources or housing pops), you may need to attach components when it is placed.

Update `spawn_building` in `src/layer1/building.rs`:

```rust
fn spawn_building(world: &mut World, x: i32, y: i32, building_type: BuildingType) {
    let mut entity = world.spawn((Building { building_type }, GridPosition { x, y }));

    match building_type {
        BuildingType::Housing => {
            entity.insert(Housing::default());
        }
        BuildingType::Farm => {
            entity.insert(Farm::default());
        }
        BuildingType::Statue => {
             // Add any components specific to Statue here
             // entity.insert(Decor::default());
        }
    }
}
```

## Troubleshooting

- **"Variant not covered"**: If you forget to update `label()`, `cost()`, `get_building_char()`, or `get_building_color()`, the compiler will complain that your match is non-exhaustive. This is good! It reminds you to implement all properties.
- **"Next not updating"**: Ensure `EnumIter` is derived. The `next()` function uses it to automatically find the next variant in the list.

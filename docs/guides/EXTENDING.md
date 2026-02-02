# Extending the Game: Adding a New Building

This guide explains how to add a new building type to SCALE.

## Overview

Buildings are defined in `src/layer1/building.rs`. The process involves adding a new variant to the `BuildingType` enum and implementing its visual properties.

## Steps

### 1. Add the Variant

Open `src/layer1/building.rs` and add your new building to `BuildingType`.

```rust
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug, EnumIter)]
pub enum BuildingType {
    #[default]
    Housing,
    Farm,
    Statue, // <--- Your new building
}
```

> **Note:** The `EnumIter` derive macro automatically handles the cycling logic in build mode. You do NOT need to manually update `next()`.

### 2. Implement Visuals

Update the `impl BuildingType` block to define how your building looks.

#### Character
Add a case to `char()`:

```rust
pub const fn char(&self) -> char {
    match self {
        Self::Housing => '⌂',
        Self::Farm => '♣',
        Self::Statue => '¥', // <--- Your char
    }
}
```

#### Color
Add a case to `color()`:

```rust
pub const fn color(&self) -> Color {
    match self {
        Self::Housing => Color::Rgb(139, 90, 43),
        Self::Farm => Color::Rgb(218, 165, 32),
        Self::Statue => Color::Gray, // <--- Your color
    }
}
```

#### Label
Add a case to `label()`:

```rust
pub const fn label(&self) -> &'static str {
    match self {
        Self::Housing => "Housing",
        Self::Farm => "Farm",
        Self::Statue => "Statue", // <--- Your label
    }
}
```

### 3. Implement Logic (Optional)

If your building has specific logic (like producing resources or housing pops), you may need to attach components when it is placed.

Update `try_place_building` in `src/layer1/building.rs`:

```rust
pub fn try_place_building(...) -> bool {
    // ...
    // Spawn building
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
    // ...
}
```

## Troubleshooting

- **"Variant not covered"**: If you forget to update `char()`, `color()`, or `label()`, the compiler will complain that your match is non-exhaustive. This is good! It reminds you to implement the visuals.
- **"Next not updating"**: Ensure `EnumIter` is derived. The `next()` function uses it to automatically find the next variant in the list.

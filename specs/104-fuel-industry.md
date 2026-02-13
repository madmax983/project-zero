# 104: Fuel Industry

## Overview

Introduces the **Fuel** resource and the **Refinery** building.
- **Fuel**: A high-energy refined resource required for space travel (Layer 2).
- **Refinery**: A building that processes **Ore** into **Fuel**.

This spec is critical for bridging the gap between Layer 1 (Colony) and Layer 2 (System), as Fleets require fuel to operate effectively.

## Dependencies

- `024` — Metal Industry (for `Ore` resource)
- `023` — Refining Industry (for `RefiningProgress` and `process_refining_system`)
- `006` — Building Placement (for `BuildingType`)

## RED Phase: Tests First

Write these tests in `src/layer1/fuel_industry_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::resources::ColonyResources;
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::refining::{RefiningProgress, process_refining_system};
    use crate::layer1::pop::Pop;
    use crate::layer1::GridPosition;

    #[test]
    fn test_resources_fuel_fields() {
        let res = ColonyResources::default();
        // New field
        assert_eq!(res.fuel, 0.0);
        // Default max
        assert_eq!(res.max_fuel, 20.0);
    }

    #[test]
    fn test_building_type_refinery() {
        let b = BuildingType::Refinery;
        assert_eq!(b.label(), "Refinery");
        assert_eq!(b.char(), 'R');
    }

    #[test]
    fn test_refining_ore_to_fuel() {
        let mut world = World::new();
        let mut res = ColonyResources::default();
        res.ore = 10.0;
        res.fuel = 0.0;
        world.insert_resource(res);

        // Spawn Refinery
        world.spawn((
            Building { building_type: BuildingType::Refinery },
            GridPosition { x: 5, y: 5 },
            RefiningProgress { current: 9.9, max: 10.0 },
        ));

        // Spawn Worker
        world.spawn((Pop, GridPosition { x: 5, y: 6 }));

        // Run system
        process_refining_system(&mut world);

        let res = world.resource::<ColonyResources>();
        // Recipe: 2 Ore -> 1 Fuel
        assert!((res.ore - 8.0).abs() < f32::EPSILON, "Should consume 2 Ore");
        assert!((res.fuel - 1.0).abs() < f32::EPSILON, "Should produce 1 Fuel");
    }

    #[test]
    fn test_refining_stops_if_insufficient_ore() {
        let mut world = World::new();
        let mut res = ColonyResources::default();
        res.ore = 1.0; // Need 2
        res.fuel = 0.0;
        world.insert_resource(res);

        world.spawn((
            Building { building_type: BuildingType::Refinery },
            GridPosition { x: 5, y: 5 },
            RefiningProgress { current: 0.0, max: 10.0 },
        ));
        world.spawn((Pop, GridPosition { x: 5, y: 6 }));

        process_refining_system(&mut world);

        let progress = world.query::<&RefiningProgress>().single(&world);
        assert_eq!(progress.current, 0.0);
    }

    #[test]
    fn test_add_fuel_clamps_to_max() {
        let mut res = ColonyResources::default();
        res.max_fuel = 10.0;
        res.fuel = 5.0;
        res.add_fuel(10.0);
        assert!((res.fuel - 10.0).abs() < f32::EPSILON);
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Update `ColonyResources` (`src/layer1/resources.rs`)

```rust
pub struct ColonyResources {
    // ... existing fields ...
    pub fuel: f32,
    pub max_fuel: f32,
}

impl Default for ColonyResources {
    fn default() -> Self {
        Self {
            // ... existing ...
            fuel: 0.0,
            max_fuel: 20.0, // Initial cap
        }
    }
}

impl ColonyResources {
    pub const fn zeroed() -> Self {
        Self {
            // ... existing ...
            fuel: 0.0,
            max_fuel: 0.0,
        }
    }

    pub fn add_fuel(&mut self, amount: f32) {
        self.fuel = (self.fuel + amount).clamp(0.0, self.max_fuel);
    }

    // Update `can_afford` and `deduct` to include fuel
}
```

### 2. Update `BuildingType` (`src/layer1/building.rs`)

```rust
pub enum BuildingType {
    // ... existing ...
    Refinery,
}

impl BuildingType {
    pub const fn label(&self) -> &'static str {
        match self {
            Self::Refinery => "Refinery",
            // ...
        }
    }

    pub const fn char(&self) -> char {
        match self {
            Self::Refinery => 'R',
            // ...
        }
    }

    pub const fn cost(&self, material: MaterialType) -> ColonyResources {
        match self {
            Self::Refinery => ColonyResources {
                wood: 20.0,
                stone: 30.0,
                metal: 10.0,
                ..ColonyResources::zeroed()
            },
            // ...
        }
    }
}

// Update spawn_building to add components
fn spawn_building(...) {
    match building_type {
        BuildingType::Refinery => {
            entity.insert((
                RefiningProgress {
                    current: 0.0,
                    max: 20.0, // Slower process
                },
                LightSource {
                    radius: 6.0,
                    intensity: 0.8,
                    color: (100, 200, 255), // Chemical blue
                },
                ShiftSchedule::default(),
            ));
        }
        // ...
    }
}
```

### 3. Update `process_refining_system` (`src/layer1/refining.rs`)

```rust
// Inside the match building.building_type block:

BuildingType::Refinery => {
    (
        resources.ore >= 2.0 && resources.fuel < resources.max_fuel,
        ColonyResources { ore: 2.0, ..ColonyResources::zeroed() },
        ColonyResources { fuel: 1.0, ..ColonyResources::zeroed() }
    )
},
```

## REFACTOR Phase: Quality & Design

- **Recipe Configuration**: As with Spec 023/024, move recipe logic to a dedicated struct or method `get_recipe(BuildingType)` to clean up the system.
- **Explosion Risk**: `Refinery` should be highly flammable or explosive. Consider adding `Volatile` component in future (Spec 106?).
- **Pollution**: Refining fuel should produce `Waste`. Update recipe to produce `Waste` alongside `Fuel` if Spec 032 logic is integrated.

## Acceptance Criteria

- [ ] `ColonyResources` has `fuel` field.
- [ ] `BuildingType::Refinery` exists.
- [ ] `Refinery` correctly processes 2 Ore -> 1 Fuel.
- [ ] Logic respects `max_fuel` cap.
- [ ] Tests pass.

## Technical Guidance

- Don't forget to update `ColonyResources::zeroed()`, `deduct()`, and `can_afford()`!
- `Refinery` cost includes `Metal`, so ensure `024-metal-industry` is fully integrated or use Stone/Wood if Metal is scarce in early game (but Fuel is late game).

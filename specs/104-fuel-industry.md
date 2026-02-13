# 104: Fuel Industry

## Overview

Introduces **Fuel** as a critical resource for advanced technology and space travel.
- **Fuel**: A refined resource produced from **Ore** (representing combustible or radioactive minerals).
- **Refinery**: A new building that converts Ore into Fuel.

This is a prerequisite for `105-launch-logistics` (getting ships into orbit).

## Dependencies

- `018` — Mining (for `Ore` availability)
- `024` — Metal Industry (for `RefiningProgress` pattern and `Ore` resource)

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
    use crate::layer1::map::GridPosition;

    #[test]
    fn test_resources_fuel_fields() {
        let res = ColonyResources::default();
        assert_eq!(res.fuel, 0.0);
        assert_eq!(res.max_fuel, 20.0); // Default capacity
    }

    #[test]
    fn test_building_type_refinery() {
        let b = BuildingType::Refinery;
        assert_eq!(b.label(), "Refinery");
        assert_eq!(b.char(), 'R');
    }

    #[test]
    fn test_refinery_converts_ore_to_fuel() {
        let mut world = World::new();
        let mut res = ColonyResources::default();
        res.ore = 10.0;
        res.fuel = 0.0;
        world.insert_resource(res);

        // Spawn Refinery
        world.spawn((
            Building { building_type: BuildingType::Refinery },
            GridPosition { x: 5, y: 5 },
            RefiningProgress { current: 9.9, max: 10.0 }, // Almost done
        ));

        // Spawn Worker
        world.spawn((Pop, GridPosition { x: 5, y: 6 }));

        // Run system
        process_refining_system(&mut world);

        let res = world.resource::<ColonyResources>();
        // Cost: 2 Ore -> 1 Fuel
        assert!((res.ore - 8.0).abs() < f32::EPSILON);
        assert!((res.fuel - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_refinery_stops_at_max_fuel() {
        let mut world = World::new();
        let mut res = ColonyResources::default();
        res.ore = 10.0;
        res.fuel = res.max_fuel; // Full
        world.insert_resource(res);

        world.spawn((
            Building { building_type: BuildingType::Refinery },
            GridPosition { x: 5, y: 5 },
            RefiningProgress { current: 9.9, max: 10.0 },
        ));
        world.spawn((Pop, GridPosition { x: 5, y: 6 }));

        process_refining_system(&mut world);

        let res = world.resource::<ColonyResources>();
        // Should not consume ore or produce fuel
        assert!((res.ore - 10.0).abs() < f32::EPSILON);
        assert!((res.fuel - res.max_fuel).abs() < f32::EPSILON);
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Update ColonyResources (`src/layer1/resources.rs`)

Add `fuel` and `max_fuel` fields.

```rust
#[derive(Resource, Debug, Clone)]
pub struct ColonyResources {
    // ... existing ...
    pub fuel: f32,
    pub max_fuel: f32,
}

impl Default for ColonyResources {
    fn default() -> Self {
        Self {
            // ... existing ...
            fuel: 0.0,
            max_fuel: 20.0,
            // ...
        }
    }
}

impl ColonyResources {
    pub fn add_fuel(&mut self, amount: f32) {
        self.fuel = (self.fuel + amount).clamp(0.0, self.max_fuel);
    }
}
```

### 2. Update BuildingType (`src/layer1/building.rs`)

Add `Refinery` variant.

```rust
pub enum BuildingType {
    // ... existing ...
    Refinery,
}

impl BuildingType {
    pub fn char(&self) -> char {
        match self {
            // ...
            Self::Refinery => 'R',
        }
    }

    pub fn construction_cost(&self) -> ColonyResources {
        match self {
            // ...
            Self::Refinery => ColonyResources {
                metal: 20.0, // Expensive
                stone: 50.0,
                ..ColonyResources::zeroed()
            },
        }
    }
}
```

### 3. Update Refining System (`src/layer1/refining.rs`)

Implement the recipe logic.

```rust
// Inside process_refining_system match block:
BuildingType::Refinery => {
    (
        resources.ore >= 2.0 && resources.fuel < resources.max_fuel,
        ColonyResources { ore: 2.0, ..ColonyResources::zeroed() },
        ColonyResources { fuel: 1.0, ..ColonyResources::zeroed() }
    )
},
```

## REFACTOR Phase: Quality & Design

- **Fuel Types**: Later, split into `LiquidFuel` (Refinery) and `NuclearFuel` (Centrifuge). For now, abstract `Fuel` is sufficient.
- **Pollution**: Refinery should generate `Waste` or `Pollution` (Spec 032/063). Add `resources.add_waste(1.0)` to the output.
- **Explosions**: Fuel stockpiles should be highly flammable/explosive (Spec 034 Fire).

## Acceptance Criteria

- [ ] `ColonyResources` tracks `fuel`.
- [ ] `Refinery` building exists.
- [ ] `Refinery` converts 2 Ore into 1 Fuel.
- [ ] Production stops when Fuel cap is reached.
- [ ] Tests pass.

## Technical Guidance

- Reuse `RefiningProgress` component pattern from Spec 023.
- Remember to update `ColonyResources::zeroed()` to initialize `fuel` and `max_fuel` to 0.0.

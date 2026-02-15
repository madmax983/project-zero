# 124: The Wild Child

## Overview

The planet claims your children. Nature vs. Nurture is a real battle in the colony.

This specification introduces a mechanic where **Children** (Pop Lifecycle) who spend excessive time outside of "Civilized" areas (Buildings or Designated Zones) accumulate **Wild Exposure**. If exposure reaches a threshold, they gain the **Feral** trait.

A `Feral` pop is physically superior (faster, stronger) but socially maladapted (hates indoors, refuses intellectual work). This forces players to choose: protect the youth in schools/zones (Nurture) or let the planet toughen them (Nature)?

## Dependencies

- `062` — Pop Lifecycle (Children exist)
- `084` — Pop Traits (Trait system)
- `056` — Designated Zones (Civilization definition)
- `006` — Building Placement (Occupied tiles definition)

## RED Phase: Tests First

Write these tests in `src/layer1/wild_child_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::lifecycle::{Age, LifeStage};
    use crate::layer1::traits::{Trait, Traits};
    use crate::layer1::map::GridPosition;
    use crate::layer1::zone::{ZoneGrid, ZoneType};
    use crate::layer1::building::OccupiedTiles;
    use crate::layer1::wild_child::{WildExposure, wild_child_system, FERAL_THRESHOLD};
    use bevy_ecs::system::RunSystemOnce;
    use std::collections::HashSet;

    #[test]
    fn test_wild_exposure_component_init() {
        let mut world = World::new();
        // Assume new children spawn with this component
        let entity = world.spawn(WildExposure::default()).id();
        let exposure = world.get::<WildExposure>(entity).unwrap();
        assert_eq!(exposure.current, 0.0);
    }

    #[test]
    fn test_exposure_increases_in_wild() {
        let mut world = World::new();

        // Setup map resources
        world.insert_resource(ZoneGrid::new(10, 10)); // Empty zones
        world.insert_resource(OccupiedTiles::default()); // No buildings

        // Spawn a Child at (5,5) - Wild
        let child = world.spawn((
            WildExposure::default(),
            Age { ticks_alive: 100, stage: LifeStage::Child },
            GridPosition { x: 5, y: 5 },
            Traits(HashSet::new()),
        )).id();

        // Run system
        world.run_system_once(wild_child_system);

        let exposure = world.get::<WildExposure>(child).unwrap();
        assert!(exposure.current > 0.0, "Exposure should increase in wild");
    }

    #[test]
    fn test_exposure_decreases_in_civilization() {
        let mut world = World::new();

        // Setup civilized zone at (5,5)
        let mut zones = ZoneGrid::new(10, 10);
        zones.set(5, 5, ZoneType::Housing);
        world.insert_resource(zones);
        world.insert_resource(OccupiedTiles::default());

        // Spawn a Child with some exposure
        let child = world.spawn((
            WildExposure { current: 10.0 },
            Age { ticks_alive: 100, stage: LifeStage::Child },
            GridPosition { x: 5, y: 5 },
            Traits(HashSet::new()),
        )).id();

        world.run_system_once(wild_child_system);

        let exposure = world.get::<WildExposure>(child).unwrap();
        assert!(exposure.current < 10.0, "Exposure should decrease in civilization");
    }

    #[test]
    fn test_feral_trait_acquisition() {
        let mut world = World::new();
        world.insert_resource(ZoneGrid::new(10, 10));
        world.insert_resource(OccupiedTiles::default());

        // Spawn Child near threshold
        let child = world.spawn((
            WildExposure { current: FERAL_THRESHOLD - 0.1 },
            Age { ticks_alive: 100, stage: LifeStage::Child },
            GridPosition { x: 5, y: 5 },
            Traits(HashSet::new()),
        )).id();

        // Run system enough times to cross threshold
        // Assuming increase is >= 0.1 per tick
        for _ in 0..10 {
            world.run_system_once(wild_child_system);
        }

        let traits = world.get::<Traits>(child).unwrap();
        assert!(traits.0.contains(&Trait::Feral), "Child should become Feral");
    }

    #[test]
    fn test_adults_do_not_gain_exposure() {
        let mut world = World::new();
        world.insert_resource(ZoneGrid::new(10, 10));
        world.insert_resource(OccupiedTiles::default());

        let adult = world.spawn((
            WildExposure::default(),
            Age { ticks_alive: 20000, stage: LifeStage::Adult },
            GridPosition { x: 5, y: 5 },
        )).id();

        world.run_system_once(wild_child_system);

        let exposure = world.get::<WildExposure>(adult).unwrap();
        assert_eq!(exposure.current, 0.0, "Adults should not gain exposure");
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Define Component and Trait

Update `src/layer1/traits.rs` to include `Feral`.

```rust
// In src/layer1/traits.rs enum Trait
pub enum Trait {
    // ... existing ...
    Feral, // +Move Speed, -Intellectual
}
```

Create `src/layer1/wild_child.rs`:

```rust
use bevy_ecs::prelude::*;
use crate::layer1::lifecycle::{Age, LifeStage};
use crate::layer1::map::GridPosition;
use crate::layer1::zone::{ZoneGrid, ZoneType};
use crate::layer1::building::OccupiedTiles;
use crate::layer1::traits::{Trait, Traits};

pub const FERAL_THRESHOLD: f32 = 1000.0; // e.g., 1 year in the wild? Tune this.
const EXPOSURE_RATE: f32 = 1.0;
const RECOVERY_RATE: f32 = 2.0; // Recover faster than gaining?

#[derive(Component, Default, Debug, Clone)]
pub struct WildExposure {
    pub current: f32,
}

pub fn wild_child_system(
    mut query: Query<(Entity, &mut WildExposure, &Age, &GridPosition, &mut Traits)>,
    zone_grid: Res<ZoneGrid>,
    occupied_tiles: Res<OccupiedTiles>,
) {
    for (entity, mut exposure, age, pos, mut traits) in query.iter_mut() {
        // Only affects Children
        if age.stage != LifeStage::Child {
            continue;
        }

        // Check if Feral already (optimization: stop tracking if feral?)
        if traits.0.contains(&Trait::Feral) {
            continue;
        }

        // Check environment
        let is_in_zone = zone_grid.get(pos.x, pos.y) != ZoneType::None;
        let is_in_building = occupied_tiles.0.contains(&(pos.x, pos.y));
        let is_civilized = is_in_zone || is_in_building;

        if is_civilized {
            exposure.current = (exposure.current - RECOVERY_RATE).max(0.0);
        } else {
            exposure.current += EXPOSURE_RATE;
        }

        // Trigger Feral
        if exposure.current >= FERAL_THRESHOLD {
            traits.0.insert(Trait::Feral);
            // Optional: Log message "Child X has gone feral!"
        }
    }
}
```

### 2. Integrate Trait Effects

Update `src/layer1/traits.rs` helpers:

```rust
pub fn get_move_speed_modifier(traits: &Traits) -> f32 {
    let mut modifier = 1.0;
    if traits.0.contains(&Trait::FastWalker) { modifier += 0.1; }
    if traits.0.contains(&Trait::Feral) { modifier += 0.2; } // +20% Speed
    modifier
}

// In job assignment logic (e.g. utility_ai or assignment system):
// Reject `JobType::Research` or `Doctor` if trait is Feral.
```

## REFACTOR Phase: Quality & Design

- **Performance**: Iterating all children every tick is fine for MVP. If pop count > 1000, move to `FixedUpdate` (once per second).
- **Optimization**: If a child is `Feral`, remove `WildExposure` component to stop processing them.
- **Flavor**: Add "Feral" visual indicator (messy hair?) or particle effect when transforming.
- **Balance**: Adjust `FERAL_THRESHOLD` based on `TICKS_PER_YEAR`. If 1 year = 1000 ticks, threshold 1000 means they must spend 100% of childhood outside. Maybe 500?

## Acceptance Criteria

- [ ] `WildExposure` component added to new children.
- [ ] Exposure increases when outside zones/buildings.
- [ ] Exposure decreases when inside.
- [ ] `Feral` trait added when threshold crossed.
- [ ] `Feral` trait grants movement speed bonus.
- [ ] `cargo test` passes.

## Technical Guidance

- Ensure `wild_child_system` runs in the update loop.
- Ensure `spawn_initial_pops` (or birth logic) adds `WildExposure::default()`.
- Use `OccupiedTiles` resource from `crate::layer1::building`.
- Use `ZoneGrid` resource from `crate::layer1::zone`.

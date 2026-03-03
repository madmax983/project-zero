# 240: Clone Vats

## Overview

Industrialized life. Why wait 18 years for a worker when you can grow one in a month?

The **Clone Vat** is a building that produces Pop entities consuming `Rations` (Nutrient Paste) and Energy. Clones are fully grown but may have specific traits like `Clone` (social stigma?) or `Soulless`.

This feature allows rapid population expansion at the cost of resources and potential social friction.

## Dependencies

- `004` — Pop Entity (Spawning mechanics)
- `042` — Energy System (Power consumption)
- `221` — Organic Recycling (Rations resource)
- `008` — Farm (Food source alternative)

## RED Phase: Tests First

Write these tests in `src/layer1/clone_vat_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::clone_vat::{CloneVat, process_clone_vats_system};
    use crate::layer1::resources::{ColonyResources, ResourceType};
    use crate::layer1::pop::Pop;
    use crate::layer1::traits::{Trait, Traits};
    use crate::layer1::map::GridPosition;

    // Helper setup
    fn setup_world() -> World {
        let mut world = World::new();
        world.insert_resource(crate::shared::time::SimulationTime::default());
        world.insert_resource(ColonyResources::default());
        world.insert_resource(crate::layer1::terrain::TerrainGrid { width: 10, height: 10, tiles: vec![crate::layer1::terrain::TerrainType::Grass; 100] });
        world
    }

    #[test]
    fn test_clone_vat_consumes_resources_to_start() {
        let mut world = setup_world();
        let mut resources = world.resource_mut::<ColonyResources>();
        resources.add(ResourceType::Rations, 100.0); // Enough for one clone (cost e.g. 50)

        // Spawn empty vat
        let vat = world.spawn((
            Building { building_type: BuildingType::CloneVat },
            CloneVat::default(),
            GridPosition { x: 5, y: 5 }
        )).id();

        // Run system
        process_clone_vats_system(&mut world);

        // Check resources consumed
        let resources = world.resource::<ColonyResources>();
        assert!(resources.get(ResourceType::Rations) < 100.0, "Should consume rations");

        // Check vat state
        let vat_comp = world.get::<CloneVat>(vat).unwrap();
        assert!(vat_comp.is_growing, "Vat should be growing a clone");
        assert!(vat_comp.progress > 0.0 || vat_comp.ticks_remaining < vat_comp.total_duration, "Progress should start");
    }

    #[test]
    fn test_clone_vat_needs_power() {
        // This assumes PowerConsumer integration or check inside the system
        // For MVP, we might just check if it runs without checking PowerGrid details deeply
        // or mock the power state.
        // Let's assume we just check logic: "If powered, progress. If not, pause."
        // ... (omitted for brevity in spec, but builder should implement)
    }

    #[test]
    fn test_clone_completion_spawns_pop() {
        let mut world = setup_world();

        // Spawn vat that is 1 tick away from completion
        let vat = world.spawn((
            Building { building_type: BuildingType::CloneVat },
            CloneVat {
                is_growing: true,
                ticks_remaining: 1,
                total_duration: 100,
                ..Default::default()
            },
            GridPosition { x: 5, y: 5 }
        )).id();

        // Run system
        process_clone_vats_system(&mut world);

        // Check Pop spawned at (5,5)
        let pop_query = world.query::<(&Pop, &GridPosition, &Traits)>();
        let mut found = false;
        for (_, pos, traits) in pop_query.iter(&world) {
            if pos.x == 5 && pos.y == 5 {
                found = true;
                assert!(traits.has(Trait::Clone), "Spawned pop should have Clone trait");
            }
        }
        assert!(found, "Should spawn a pop at vat location");

        // Check vat reset
        let vat_comp = world.get::<CloneVat>(vat).unwrap();
        assert!(!vat_comp.is_growing, "Vat should be idle after completion");
    }

    #[test]
    fn test_clone_trait_behavior() {
        // Ensure Trait::Clone exists and has expected effects (e.g., social malus?)
        let clone_trait = Trait::Clone;
        assert_eq!(clone_trait.label(), "Clone");

        // Example: Clones might have lower social need decay (Soulless?) or just a tag for now.
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Update Enums

- **BuildingType**: Add `CloneVat`.
- **Trait**: Add `Clone`.
- **ResourceType**: Ensure `Rations` is used (from Spec 221).

### 2. Define `CloneVat` Component

```rust
// src/layer1/clone_vat.rs

use bevy_ecs::prelude::*;

#[derive(Component, Debug, Clone)]
pub struct CloneVat {
    pub is_growing: bool,
    pub ticks_remaining: u32,
    pub total_duration: u32,
    pub ration_cost: f32,
}

impl Default for CloneVat {
    fn default() -> Self {
        Self {
            is_growing: false,
            ticks_remaining: 1000, // Placeholder
            total_duration: 1000,
            ration_cost: 50.0,
        }
    }
}
```

### 3. Implement System

```rust
// src/layer1/clone_vat.rs

use crate::layer1::resources::{ColonyResources, ResourceType};
use crate::layer1::pop::PopBundle; // Assuming helper exists or use spawn_initial_pops logic
use crate::layer1::map::GridPosition;
use crate::layer1::traits::{Trait, Traits};

pub fn process_clone_vats_system(
    mut commands: Commands,
    mut query: Query<(&mut CloneVat, &GridPosition)>, // Add PowerConsumer check here
    mut resources: ResMut<ColonyResources>,
) {
    for (mut vat, pos) in query.iter_mut() {
        if vat.is_growing {
            // Check power here (omitted for brevity)

            if vat.ticks_remaining > 0 {
                vat.ticks_remaining -= 1;
            } else {
                // Complete!
                // Spawn Pop
                // Use a helper to spawn a random pop, then modify it
                // OR manually spawn PopBundle with "Clone" trait
                let entity = commands.spawn(PopBundle::random(pos.x, pos.y, &mut rand::thread_rng())).id();

                // Force Clone trait
                commands.entity(entity).insert(crate::layer1::traits::Traits(
                     std::collections::HashSet::from([Trait::Clone])
                ));
                // Maybe "Soulless"?

                vat.is_growing = false;
            }
        } else {
            // Try to start new batch if auto-queue is on (or just manual for now)
            // For MVP, auto-start if resources available?
            // "Industrial" implies automation.
            if resources.get(ResourceType::Rations) >= vat.ration_cost {
                resources.remove(ResourceType::Rations, vat.ration_cost);
                vat.is_growing = true;
                vat.ticks_remaining = vat.total_duration;
            }
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **Trait Integration**: `Trait::Clone` should likely have social interactions. Non-clones might dislike Clones (`Xenophobia` or `UncannyValley`).
- **Configurable Output**: Vats could produce specific "Genotypes" (Soldier Clone, Worker Clone) if `GeneBanks` (Spec 165) is integrated.
- **Power Failure**: If power is cut, does the clone die? Or just pause? (Pause is kinder, Death is grimmer/more "SCALE").

## Acceptance Criteria

- [ ] `CloneVat` building exists.
- [ ] Consumes `Rations` to start production.
- [ ] Takes time to produce.
- [ ] Spawns a `Pop` with `Clone` trait.
- [ ] Tests pass.

## Technical Guidance

- Reuse `PopBundle::random` but ensure you override the traits or age (Clones might be born adult? Or fast-growing children?).
- `003` Population Basics established `PopBundle`.
- Ensure `Rations` are correctly deducted from global `ColonyResources`.

## Questions

- *Builder: Should clones be born as adults?*
*Architect:* Yes, clones are decanted as fully functional adults to bypass the normal lifecycle pipeline, representing their industrial utility.
  *Architect: Yes, clones are decanted as fully mature, working-age adults.*
- *Architect: Yes, Clone Vats spawn adult clones.*
- *Builder: Do they have parents?*
*Architect:* No, they are generated with empty parent relationship fields, which naturally prevents them from inheriting family-based social buffs/debuffs.
  *Architect: No, clones lack parents and do not inherit familial relationships, giving them the "Soulless" or similar trait.*
- *Architect: No, they have empty family trees.*

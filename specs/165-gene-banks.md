# 165: Gene-Banks

## Overview

The **Gene-Bank** is a high-tech facility that allows the colony to store genetic samples of Flora and Fauna. This system provides a way to preserve biodiversity and resurrect species that may go extinct due to over-harvesting, pollution, or disaster.

It introduces the concept of **Genetic Samples** as items, which can be collected from living entities or corpses and stored in the Gene-Bank. These samples can then be used to clone new organisms, consuming Biomass (Food) and Energy.

This feature integrates with `161` Ecological Succession (replanting forests) and `075` Animal Husbandry (breeding livestock).

## Dependencies

- `004` — Basic Building System (BuildingType::GeneBank)
- `161` — Ecological Succession (Flora types)
- `075` — Animal Husbandry (Fauna types)
- `030` — Tool Economy (Sampler tool, optional but thematic)

## RED Phase: Tests First

Write these tests in `src/layer1/gene_bank_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::gene_bank::{
        GeneBank, GeneticSample, GeneticData, collect_sample_action, process_cloning_system
    };
    use crate::layer1::item::{Item, ItemType};
    use crate::layer1::fauna::{Fauna, FaunaType};
    use crate::layer1::terrain::{TerrainType, TerrainGrid};
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::pop::Pop;
    use crate::layer1::map::GridPosition;
    use crate::layer1::resources::ColonyResources;

    // Helper setup
    fn setup_world() -> World {
        let mut world = World::new();
        world.insert_resource(crate::shared::time::SimulationTime::default());
        world.insert_resource(ColonyResources::default());
        world.insert_resource(TerrainGrid { width: 10, height: 10, tiles: vec![TerrainType::Grass; 100] });
        world
    }

    #[test]
    fn test_collect_flora_sample() {
        let mut world = setup_world();

        // Spawn a Pop and a Tree
        let pop = world.spawn((Pop, GridPosition { x: 0, y: 0 })).id();
        // Assume collecting from terrain at (0,1) which is a Tree
        let mut grid = world.resource_mut::<TerrainGrid>();
        grid.set(0, 1, TerrainType::Tree);

        // Perform collection action (mocking the action execution)
        let success = collect_sample_action(&mut world, pop, GridPosition { x: 0, y: 1 });

        assert!(success, "Should successfully collect sample from Tree");

        // Check for dropped item
        let item_query = world.query::<(&Item, &GeneticSample, &GridPosition)>();
        let (item, sample, pos) = item_query.single(&world);

        assert_eq!(item.item_type, ItemType::GeneticSample);
        match sample.data {
            GeneticData::Flora(t) => assert_eq!(t, TerrainType::Tree),
            _ => panic!("Expected Flora sample"),
        }
        assert_eq!(pos.x, 0);
        assert_eq!(pos.y, 1);
    }

    #[test]
    fn test_collect_fauna_sample() {
        let mut world = setup_world();

        // Spawn Pop and Animal
        let pop = world.spawn((Pop, GridPosition { x: 0, y: 0 })).id();
        let animal = world.spawn((
            Fauna { fauna_type: FaunaType::SpaceRat, ..Default::default() },
            GridPosition { x: 0, y: 1 }
        )).id();

        // Perform collection (target entity)
        let success = collect_sample_action(&mut world, pop, GridPosition { x: 0, y: 1 }); // Target pos

        assert!(success);

        // Check for item
        let item_query = world.query::<(&Item, &GeneticSample)>();
        let (_, sample) = item_query.single(&world);

        match sample.data {
            GeneticData::Fauna(f) => assert_eq!(f, FaunaType::SpaceRat),
            _ => panic!("Expected Fauna sample"),
        }
    }

    #[test]
    fn test_gene_bank_storage() {
        let mut world = setup_world();

        // Spawn GeneBank building
        let bank = world.spawn((
            Building { building_type: BuildingType::GeneBank },
            GeneBank::default(),
            GridPosition { x: 5, y: 5 }
        )).id();

        // Add a sample to its storage manually
        let mut bank_comp = world.get_mut::<GeneBank>(bank).unwrap();
        bank_comp.store_sample(GeneticData::Flora(TerrainType::Tree));

        assert!(bank_comp.has_sample(&GeneticData::Flora(TerrainType::Tree)));
    }

    #[test]
    fn test_cloning_process_flora() {
        let mut world = setup_world();

        // Setup GeneBank with a sample and resources
        let bank = world.spawn((
            Building { building_type: BuildingType::GeneBank },
            GeneBank {
                stored_samples: vec![GeneticData::Flora(TerrainType::Tree)],
                active_cloning_job: Some((GeneticData::Flora(TerrainType::Tree), 10.0)), // 10 ticks remaining
                ..Default::default()
            },
            GridPosition { x: 5, y: 5 }
        )).id();

        // Run system for 10 ticks
        for _ in 0..10 {
            process_cloning_system(&mut world);
        }

        // Check result: Should produce a "Seed" item at (5,5)
        let item_query = world.query::<(&Item, &GridPosition)>();
        // Assuming ItemType::Seed(TerrainType) or similar exists, or just a generic "Sapling" item
        // For MVP, let's say it spawns ItemType::Seed
        let (item, pos) = item_query.single(&world);
        assert_eq!(pos.x, 5);
        assert_eq!(pos.y, 5);
        // Verify cloning job is cleared
        let bank_comp = world.get::<GeneBank>(bank).unwrap();
        assert!(bank_comp.active_cloning_job.is_none());
    }

    #[test]
    fn test_cloning_process_fauna() {
        let mut world = setup_world();

        let bank = world.spawn((
            Building { building_type: BuildingType::GeneBank },
            GeneBank {
                stored_samples: vec![GeneticData::Fauna(FaunaType::Wolf)],
                active_cloning_job: Some((GeneticData::Fauna(FaunaType::Wolf), 0.0)), // Finished
                ..Default::default()
            },
            GridPosition { x: 5, y: 5 }
        )).id();

        process_cloning_system(&mut world);

        // Should spawn a Fauna entity
        let fauna_query = world.query::<(&Fauna, &GridPosition)>();
        let (fauna, pos) = fauna_query.single(&world);
        assert_eq!(fauna.fauna_type, FaunaType::Wolf);
        assert_eq!(pos.x, 5);
        assert_eq!(pos.y, 5);
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Update Enums and Types

- **ItemType**: Add `GeneticSample`.
- **BuildingType**: Add `GeneBank`.
- **ActionType**: Add `CollectSample`.
- **DesignationType**: Add `CollectSample`.

### 2. Define Structures

```rust
// src/layer1/gene_bank.rs

use bevy_ecs::prelude::*;
use crate::layer1::terrain::TerrainType;
use crate::layer1::fauna::FaunaType;
use crate::layer1::item::{Item, ItemType};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GeneticData {
    Flora(TerrainType),
    Fauna(FaunaType),
    // Future: Pop(PopParams), FaunaSeed(u64)
}

#[derive(Component, Debug, Clone)]
pub struct GeneticSample {
    pub data: GeneticData,
}

#[derive(Component, Default, Debug)]
pub struct GeneBank {
    pub stored_samples: Vec<GeneticData>,
    // Cloning Job: (Target, TicksRemaining)
    pub active_cloning_job: Option<(GeneticData, f32)>,
}

impl GeneBank {
    pub fn store_sample(&mut self, data: GeneticData) {
        if !self.stored_samples.contains(&data) {
            self.stored_samples.push(data);
        }
    }

    pub fn has_sample(&self, data: &GeneticData) -> bool {
        self.stored_samples.contains(data)
    }
}
```

### 3. Implement Systems

```rust
// src/layer1/gene_bank.rs

use crate::layer1::map::GridPosition;
use crate::layer1::terrain::TerrainGrid;
use crate::layer1::fauna::Fauna;

pub fn collect_sample_action(world: &mut World, _actor: Entity, target_pos: GridPosition) -> bool {
    // 1. Check for Fauna at position
    let mut found_data = None;

    // Check Fauna entities at target_pos
    let mut fauna_query = world.query::<(Entity, &GridPosition, &Fauna)>();
    for (_, pos, fauna) in fauna_query.iter(world) {
        if pos == &target_pos {
            found_data = Some(GeneticData::Fauna(fauna.fauna_type));
            break;
        }
    }

    // 2. If no fauna, check Flora (Terrain)
    if found_data.is_none() {
        let grid = world.resource::<TerrainGrid>();
        if let Some(tile) = grid.get(target_pos.x, target_pos.y) {
            // Only collect from biological tiles
            match tile {
                TerrainType::Tree | TerrainType::Shrub | TerrainType::Sapling | TerrainType::Grass => {
                    found_data = Some(GeneticData::Flora(tile));
                },
                _ => {}
            }
        }
    }

    // 3. Spawn Item
    if let Some(data) = found_data {
        world.spawn((
            Item { item_type: ItemType::GeneticSample },
            GeneticSample { data },
            target_pos
        ));
        return true;
    }

    false
}

pub fn process_cloning_system(world: &mut World) {
    let mut completed_clones = Vec::new();

    let mut query = world.query::<(Entity, &mut GeneBank, &GridPosition)>();
    for (entity, mut bank, pos) in query.iter_mut(world) {
        if let Some((target, time)) = &mut bank.active_cloning_job {
            if *time > 0.0 {
                *time -= 1.0;
            } else {
                // Clone complete
                completed_clones.push((target.clone(), *pos));
                bank.active_cloning_job = None;
            }
        }
    }

    // Spawn clones
    for (data, pos) in completed_clones {
        match data {
            GeneticData::Flora(t) => {
                // For MVP: Spawn a "Seed" item or Sapling entity?
                // Let's spawn a Seed Item.
                // Assuming ItemType::Seed exists or generic ItemType::Resource(ResourceType::Seed)
                // Using generic "Item" logic for now.
                world.spawn((
                    Item { item_type: ItemType::GeneticSample }, // Placeholder for actual Seed item
                    pos
                ));
            },
            GeneticData::Fauna(f) => {
                world.spawn((
                    Fauna { fauna_type: f, ..Default::default() },
                    pos
                ));
            }
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **Storage Capacity**: Gene Banks should have finite storage slots for samples.
- **DNA Degradation**: Samples could degrade over time if power is lost (`042` Energy System).
- **Genetic Drift**: When `164` Modular Fauna is implemented, cloning should apply a slight randomization to the `FaunaSeed` to simulate mutation/drift.
- **UI**: Need a UI to view stored samples and initiate cloning jobs.

## Acceptance Criteria

- [ ] `GeneBank` building can be constructed.
- [ ] Pops can be assigned to collect samples from plants/animals.
- [ ] `GeneticSample` items drop and can be hauled to the bank.
- [ ] Gene Bank stores unique samples.
- [ ] Cloning process consumes time/resources and produces a new entity.
- [ ] Tests pass.

## Technical Guidance

- Use `ItemType` variants for `GeneticSample` to allow hauling logic to treat it as a normal item.
- Ensure `GeneBank` has `PowerConsumer` component so it shuts down without energy.

## Questions

- *Builder: Should we allow cloning of Pops?*
- *Architect:* No, Gene Banks only clone flora/fauna; Clone Vats (Spec 240) are used for Pops.

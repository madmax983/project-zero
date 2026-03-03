# 248: The Infinite Archive

## Overview

"Knowledge is infinite, but hard drives are not."

Research generates "Data" which consumes physical space in **Server Racks**. As the archive grows, **Search Time** increases, slowing down new research (simulating the difficulty of cross-referencing massive datasets).
To maintain efficiency, the player must **Delete** old data (forgetting lower-tier tech or losing lore bonuses) or build exponentially more storage.

- **Data Mass**: Every unlocked Tech adds a `DataSize` value to the Colony's total.
- **Archive Bloat**: Research speed is penalized by `TotalData / Capacity`.
- **Purge**: Action to remove unlocked Techs from the database to free space.

## Dependencies

- `029` — Knowledge System (Research mechanics)
- `143` — Data Physicality (Physical server objects)

## RED Phase: Tests First

Write these tests in `src/layer1/tech/infinite_archive_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::tech::infinite_archive::{Archive, DataSize, ResearchEfficiency, update_efficiency_system};
    use crate::layer1::tech::{TechTree, TechStatus};

    #[test]
    fn test_data_accumulation_reduces_efficiency() {
        let mut world = World::new();

        // Setup Archive with limited capacity
        world.insert_resource(Archive {
            capacity: 100.0,
            used: 0.0,
            efficiency_multiplier: 1.0,
        });

        // Setup a Tech Tree with some unlocked techs
        // (Mocking the TechTree resource for simplicity)
        let mut tech_tree = TechTree::default();
        tech_tree.unlock("Steam Power", DataSize(50.0)); // 50% capacity
        world.insert_resource(tech_tree);

        // Run system to update usage and efficiency
        let mut schedule = Schedule::default();
        schedule.add_systems(update_efficiency_system);
        schedule.run(&mut world);

        let archive = world.resource::<Archive>();
        assert_eq!(archive.used, 50.0);
        // Efficiency should be affected. Let's say 1.0 at 0% usage, 0.5 at 100% usage.
        // Formula: 1.0 - (usage / capacity * 0.5)
        assert_eq!(archive.efficiency_multiplier, 0.75);
    }

    #[test]
    fn test_overcapacity_halts_research() {
        let mut world = World::new();
        world.insert_resource(Archive {
            capacity: 100.0,
            used: 110.0, // Over capacity
            efficiency_multiplier: 1.0,
        });

        // Mock tech tree
        let mut tech_tree = TechTree::default();
        world.insert_resource(tech_tree);

        let mut schedule = Schedule::default();
        schedule.add_systems(update_efficiency_system);
        schedule.run(&mut world);

        let archive = world.resource::<Archive>();
        assert_eq!(archive.efficiency_multiplier, 0.0); // Research halted
    }

    #[test]
    fn test_purge_tech_restores_efficiency() {
        let mut world = World::new();
        let mut tech_tree = TechTree::default();
        tech_tree.unlock("Fusion", DataSize(100.0));
        world.insert_resource(tech_tree);
        world.insert_resource(Archive { capacity: 100.0, used: 100.0, efficiency_multiplier: 0.5 });

        // Perform Purge
        crate::layer1::tech::infinite_archive::purge_tech(&mut world, "Fusion");

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(update_efficiency_system);
        schedule.run(&mut world);

        let archive = world.resource::<Archive>();
        assert_eq!(archive.used, 0.0);
        assert_eq!(archive.efficiency_multiplier, 1.0);

        let tree = world.resource::<TechTree>();
        assert_eq!(tree.get_status("Fusion"), TechStatus::Forgotten);
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Resources & Components

```rust
// src/layer1/tech/infinite_archive.rs

use bevy_ecs::prelude::*;
use crate::layer1::tech::{TechTree, TechStatus};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DataSize(pub f32);

#[derive(Resource, Default, Debug)]
pub struct Archive {
    pub capacity: f32, // Derived from building count (Server Racks)
    pub used: f32,
    pub efficiency_multiplier: f32,
}

pub fn update_efficiency_system(
    mut archive: ResMut<Archive>,
    tech_tree: Res<TechTree>,
    // query: Query<&ServerRack> // Real impl would query buildings for capacity
) {
    let mut total_size = 0.0;

    // Iterate unlocked techs to sum size
    for (tech_id, tech) in tech_tree.iter() {
        if tech.status == TechStatus::Unlocked {
            total_size += tech.data_size.0;
        }
    }

    archive.used = total_size;

    if archive.capacity > 0.0 {
        let ratio = archive.used / archive.capacity;
        if ratio >= 1.0 {
            archive.efficiency_multiplier = 0.0; // Paralysis
        } else {
            // Linear decay: 0% usage = 100% speed, 100% usage = 50% speed (before halt)
            // Or simple penalty curve
            archive.efficiency_multiplier = 1.0 - (ratio * 0.5);
        }
    } else {
        archive.efficiency_multiplier = 0.0; // No servers = no research
    }
}

pub fn purge_tech(world: &mut World, tech_id: &str) {
    let mut tech_tree = world.resource_mut::<TechTree>();
    if let Some(tech) = tech_tree.get_mut(tech_id) {
        tech.status = TechStatus::Forgotten;
    }
}
```

## REFACTOR Phase: Quality & Design

- **Capacity Logic**: `Archive.capacity` should be dynamically updated by a system counting `BuildingType::ServerRack` entities.
- **Lore**: Forgotten tech should be logged in the Chronicle ("We have forgotten the secrets of Steam Power to make room for Fusion").
- **UI**: Display "Archive Load: 85%" with a warning color. "Purge" button in Tech Tree.

## Acceptance Criteria

- [ ] `Archive` resource tracks capacity vs usage.
- [ ] `ResearchEfficiency` scales with usage.
- [ ] `purge_tech` successfully removes tech and frees space.
- [ ] Tests pass.

## Technical Guidance

- Integrate with `Spec 246 (Legacy Code)`: `Bloat` affects *machines*, `Archive Load` affects *research speed*. They are complementary.
- Ensure "Forgotten" techs can be re-researched (perhaps faster?).

## Questions

- *Builder: Does deleting a prerequisite tech lock the advanced tech?*
  *Architect: No, once an advanced tech is unlocked, it remains unlocked even if its prerequisite is deleted to save space.*
*Architect: No, you retain the advanced tech, but you cannot build the prerequisite tech anymore. This creates a "black box" situation where you use tech you no longer fully understand.*
  - *Architect: No, you keep the advanced tech, but you can't build the basic components anymore (unless the advanced tech supersedes them).*

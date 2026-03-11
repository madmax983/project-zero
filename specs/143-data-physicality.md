# 143: Data Physicality

## Overview

Currently, **Knowledge** and unlocked **Technologies** are abstract concepts that exist safely in the UI.
This spec introduces **Data Physicality**: research data requires physical storage space.
- **Server Banks**: New building type that provides **Data Capacity**.
- **Tech Storage Cost**: Each unlocked `Tech` consumes a specific amount of capacity (e.g., 10 TB).
- **Data Corruption**: If `Used Capacity > Total Capacity` (e.g., due to Server Bank destruction or power loss), random technologies become **Corrupted**.
- **Consequences**: Corrupted technologies are effectively "locked" until capacity is restored and they are "repaired" (or just automatically restored). Buildings requiring corrupted tech cease to function.

This adds a layer of vulnerability to the colony's intellectual property. You must defend your servers as much as your generators.

## Dependencies

- `029` — Knowledge System (Techs, Unlocking)
- `042` — Energy System (Servers need power)
- `006` — Building Placement (Server Bank building)

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/layer1/tech_storage_tests.rs

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::tech::{Tech, TechState, TechStatus};
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::resources::ColonyResources;
    use std::collections::HashSet;

    #[test]
    fn test_tech_has_storage_cost() {
        assert_eq!(Tech::Masonry.storage_cost(), 5.0);
        assert_eq!(Tech::Astronomy.storage_cost(), 20.0);
    }

    #[test]
    fn test_tech_state_tracks_capacity() {
        let mut state = TechState::default();
        assert_eq!(state.total_capacity, 0.0);
        assert_eq!(state.used_capacity, 0.0);

        // Manual capacity adjustment for test
        state.total_capacity = 100.0;
        assert_eq!(state.total_capacity, 100.0);
    }

    #[test]
    fn test_unlocking_tech_increases_usage() {
        let mut state = TechState::default();
        state.total_capacity = 100.0;

        state.unlock(Tech::Masonry);

        assert!(state.is_unlocked(Tech::Masonry));
        assert_eq!(state.used_capacity, Tech::Masonry.storage_cost());
    }

    #[test]
    fn test_cannot_unlock_if_full() {
        let mut state = TechState::default();
        state.total_capacity = 4.0; // Not enough for Masonry (5.0)

        // Should fail or return false
        let success = state.try_unlock(Tech::Masonry);

        assert!(!success);
        assert!(!state.is_unlocked(Tech::Masonry));
    }

    #[test]
    fn test_capacity_drop_triggers_corruption() {
        let mut state = TechState::default();
        state.total_capacity = 10.0;
        state.unlock(Tech::Masonry); // Cost 5.0

        // Disaster! Capacity drops
        state.total_capacity = 0.0;

        // System update should mark tech as Corrupted
        state.update_corruption();

        assert_eq!(state.status(Tech::Masonry), TechStatus::Corrupted);
    }

    #[test]
    fn test_corrupted_tech_blocks_access() {
        let mut state = TechState::default();
        state.force_unlock(Tech::Masonry, TechStatus::Corrupted);

        // is_unlocked should return FALSE for corrupted tech
        // so buildings check `required_tech` fails
        assert!(!state.is_active(Tech::Masonry));
    }

    #[test]
    fn test_server_bank_increases_capacity() {
        let mut world = World::new();
        // Register TechState
        world.insert_resource(TechState::default());

        // Spawn ServerBank
        let id = world.spawn((
            Building { building_type: BuildingType::ServerBank },
            crate::layer1::tech::DataStorage { capacity: 50.0 },
            crate::layer1::energy::PowerConsumer { active: true, ..Default::default() },
        )).id();

        // Run system that updates capacity
        crate::layer1::tech::update_tech_capacity_system(&mut world);

        let state = world.resource::<TechState>();
        assert_eq!(state.total_capacity, 50.0);
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Update `Tech` Enum

Add `storage_cost()` method.

```rust
impl Tech {
    pub fn storage_cost(&self) -> f32 {
        match self {
            Self::Masonry => 5.0,
            Self::MetalWorking => 10.0,
            Self::Astronomy => 20.0,
            // ...
            _ => 5.0,
        }
    }
}
```

### 2. Update `TechState`

Change `unlocked` from `HashSet<Tech>` to `HashMap<Tech, TechStatus>`.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TechStatus {
    Active,
    Corrupted,
}

#[derive(Resource, Default, Debug)]
pub struct TechState {
    // Map of unlocked techs and their status
    pub techs: std::collections::HashMap<Tech, TechStatus>,
    pub total_capacity: f32,
    pub used_capacity: f32,
}

impl TechState {
    pub fn is_active(&self, tech: Tech) -> bool {
        match self.techs.get(&tech) {
            Some(TechStatus::Active) => true,
            _ => false,
        }
    }

    pub fn try_unlock(&mut self, tech: Tech) -> bool {
        if self.techs.contains_key(&tech) { return true; } // Already unlocked

        if self.used_capacity + tech.storage_cost() > self.total_capacity {
            return false;
        }

        self.techs.insert(tech, TechStatus::Active);
        self.used_capacity += tech.storage_cost();
        true
    }

    pub fn update_corruption(&mut self) {
        // If used > total, mark random active techs as corrupted until used <= total?
        // OR simply mark ALL techs as Corrupted if capacity is zero?
        // Let's implement a simple greedy corruption:
        // Iterate techs. If used > total, flip active to corrupted.

        // Re-calculate used from Active techs
        let active_usage: f32 = self.techs.iter()
            .filter(|(_, status)| **status == TechStatus::Active)
            .map(|(t, _)| t.storage_cost())
            .sum();

        if active_usage > self.total_capacity {
            // Overloaded!
            // Strategy: Corrupt the MOST EXPENSIVE techs first (or random).
            // For MVP: Corrupt everything. (Harsh but simple)
            // Or better: Iterate and corrupt until under cap.

            let mut active_techs: Vec<Tech> = self.techs.iter()
                .filter(|(_, s)| **s == TechStatus::Active)
                .map(|(t, _)| *t)
                .collect();

            // Sort by cost descending?
            active_techs.sort_by(|a, b| b.storage_cost().partial_cmp(&a.storage_cost()).unwrap());

            let mut current_usage = active_usage;
            for tech in active_techs {
                if current_usage <= self.total_capacity { break; }

                self.techs.insert(tech, TechStatus::Corrupted);
                current_usage -= tech.storage_cost();
            }
        } else {
            // Restore corruption if capacity allows?
            // "Auto-repair" logic:
            let mut corrupted_techs: Vec<Tech> = self.techs.iter()
                .filter(|(_, s)| **s == TechStatus::Corrupted)
                .map(|(t, _)| *t)
                .collect();

            // Sort by cost ascending (restore cheap ones first)
            corrupted_techs.sort_by(|a, b| a.storage_cost().partial_cmp(&b.storage_cost()).unwrap());

            let mut current_usage = active_usage;
            for tech in corrupted_techs {
                if current_usage + tech.storage_cost() <= self.total_capacity {
                    self.techs.insert(tech, TechStatus::Active);
                    current_usage += tech.storage_cost();
                }
            }
        }

        // Update final used_capacity
        self.used_capacity = self.techs.iter()
            .filter(|(_, status)| **status == TechStatus::Active)
            .map(|(t, _)| t.storage_cost())
            .sum();
    }
}
```

### 3. Add `ServerBank` Building

Update `BuildingType` and `spawn_building`.

```rust
// src/layer1/tech.rs
#[derive(Component, Default)]
pub struct DataStorage {
    pub capacity: f32,
}

// src/layer1/building.rs
// Add BuildingType::ServerBank
// Implement spawn:
// - DataStorage { capacity: 50.0 }
// - PowerConsumer { demand: 10.0, active: true }
// - Structure (Health)
```

### 4. Create Capacity Update System

```rust
// src/layer1/tech.rs

pub fn update_tech_capacity_system(
    mut tech_state: ResMut<TechState>,
    query: Query<(&DataStorage, &crate::layer1::energy::PowerConsumer)>,
) {
    let total_cap: f32 = query.iter()
        .filter(|(_, power)| power.active) // Only powered servers count!
        .map(|(storage, _)| storage.capacity)
        .sum();

    tech_state.total_capacity = total_cap;
    tech_state.update_corruption();
}
```

## REFACTOR Phase: Quality & Design

- **UI Feedback**: The Tech UI must show `Capacity: 50/100 TB`.
- **Alerts**: Notification when capacity is full or techs are corrupted.
- **Boot Sequence**: Servers might take time to boot up (PowerConsumer logic).
- **Grace Period**: Maybe corruption isn't instant? A "Buffer" resource?

## Acceptance Criteria

- [ ] `TechState` enforced capacity limits.
- [ ] `ServerBank` building exists and requires power.
- [ ] Unpowered/Destroyed ServerBanks reduce total capacity.
- [ ] Over-capacity causes techs to go `Corrupted`.
- [ ] `Corrupted` techs block building construction/operation.
- [ ] Restoring capacity automatically restores techs (auto-repair).
- [ ] Test coverage >= 85%.

## Technical Guidance

- Modify `unlock_tech` to use `try_unlock`.
- Modify `process_research_system`: Cannot research if full? Or just cannot unlock? (Cannot unlock is simpler).
- Ensure `update_tech_capacity_system` runs before building logic in the schedule.

## Questions

- Should `Library` provide a small base capacity (e.g. 5.0) so early game isn't broken?
  - **Answer**: Yes, or start with `BaseCapacity` in `TechState` (e.g., 10.0 from the Lander computer).

*Architect:* Assume a base capacity provided by the initial starting tech state.

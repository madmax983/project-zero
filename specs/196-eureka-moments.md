# 196: Eureka Moments

## Overview

Working a job has a small chance to trigger a "Eureka Moment," unlocking a specific related tech or granting a burst of Knowledge. This rewards specialization and allows technology to advance organically through doing, rather than just passive research.

## Dependencies

- `011` Tech Tree Backend
- `006` Job Assignment
- `013` Utility AI Actions

## RED Phase: Tests First

These tests define the feature's behavior. They must be written and fail before any implementation.

```rust
// src/layer1/eureka_tests.rs

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::tech::{Tech, TechState};
    use crate::layer1::resources::ColonyResources;
    use crate::layer1::actions::ActionType;
    use crate::layer1::eureka::{check_for_eureka, EurekaConfig};

    #[test]
    fn test_mining_triggers_masonry_eureka() {
        // Arrange
        let mut world = World::new();
        world.insert_resource(TechState::default());
        world.insert_resource(ColonyResources::default());
        world.insert_resource(EurekaConfig { base_chance: 1.0 }); // Guaranteed success

        // Act: Simulate mining work with a guaranteed eureka trigger
        let eureka_occurred = check_for_eureka(&mut world, ActionType::Work, Some(Tech::Masonry));

        // Assert
        assert!(eureka_occurred);
        // Tech should be unlocked directly
        assert!(world.resource::<TechState>().is_unlocked(Tech::Masonry));
    }

    #[test]
    fn test_farming_triggers_hydroponics_eureka() {
        // Arrange
        let mut world = World::new();
        world.insert_resource(TechState::default());
        world.insert_resource(ColonyResources::default());
        world.insert_resource(EurekaConfig { base_chance: 1.0 });

        // Act
        let eureka_occurred = check_for_eureka(&mut world, ActionType::Farm, Some(Tech::Hydroponics));

        // Assert
        assert!(eureka_occurred);
        assert!(world.resource::<TechState>().is_unlocked(Tech::Hydroponics));
    }

    #[test]
    fn test_knowledge_gain_if_tech_already_unlocked() {
        // Arrange
        let mut world = World::new();
        let mut state = TechState::default();
        state.unlock(Tech::Masonry);
        world.insert_resource(state);
        world.insert_resource(ColonyResources::default());
        world.insert_resource(EurekaConfig { base_chance: 1.0 });

        // Act
        let eureka_occurred = check_for_eureka(&mut world, ActionType::Work, Some(Tech::Masonry));

        // Assert
        assert!(eureka_occurred);
        // Knowledge should increase (e.g., 10.0 points)
        assert!(world.resource::<ColonyResources>().knowledge >= 10.0);
    }

    #[test]
    fn test_no_eureka_if_chance_fails() {
        // Arrange
        let mut world = World::new();
        world.insert_resource(TechState::default());
        world.insert_resource(ColonyResources::default());
        world.insert_resource(EurekaConfig { base_chance: 0.0 }); // Guaranteed fail

        // Act
        let eureka_occurred = check_for_eureka(&mut world, ActionType::Work, Some(Tech::Masonry));

        // Assert
        assert!(!eureka_occurred);
        assert!(!world.resource::<TechState>().is_unlocked(Tech::Masonry));
        assert_eq!(world.resource::<ColonyResources>().knowledge, 0.0);
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. `EurekaConfig` Resource

Define a resource to hold configuration values, allowing easy tuning and testing.

```rust
// src/layer1/eureka.rs

use bevy_ecs::prelude::*;
use crate::layer1::tech::{Tech, TechState};
use crate::layer1::resources::ColonyResources;
use crate::layer1::actions::ActionType;
use crate::shared::log::MessageLog; // Assuming MessageLog exists
use rand::Rng;

#[derive(Resource)]
pub struct EurekaConfig {
    /// Base probability per tick of triggering a Eureka moment.
    pub base_chance: f64,
    /// Amount of Knowledge granted if Tech is already unlocked.
    pub knowledge_reward: f32,
}

impl Default for EurekaConfig {
    fn default() -> Self {
        Self {
            base_chance: 0.0001, // 0.01% per tick
            knowledge_reward: 10.0,
        }
    }
}
```

### 2. `check_for_eureka` Function

Implement the core logic to roll for a Eureka moment and apply the reward.

```rust
// src/layer1/eureka.rs

pub fn check_for_eureka(world: &mut World, action: ActionType, related_tech: Option<Tech>) -> bool {
    let config = world.resource::<EurekaConfig>();
    let chance = config.base_chance;

    // Simple RNG check
    let mut rng = rand::thread_rng();
    if !rng.gen_bool(chance) {
        return false;
    }

    // Success! Determine reward.
    let mut tech_unlocked = false;
    let mut knowledge_gained = 0.0;

    if let Some(tech) = related_tech {
        let mut tech_state = world.resource_mut::<TechState>();
        if !tech_state.is_unlocked(tech) {
            // Unlock the tech!
            tech_state.unlock(tech);
            tech_unlocked = true;
        } else {
            // Fallback reward
            knowledge_gained = world.resource::<EurekaConfig>().knowledge_reward;
        }
    } else {
        // No related tech, just knowledge
        knowledge_gained = world.resource::<EurekaConfig>().knowledge_reward;
    }

    // Apply Knowledge Reward
    if knowledge_gained > 0.0 {
        let mut res = world.resource_mut::<ColonyResources>();
        res.knowledge += knowledge_gained;
    }

    // Log the event
    if let Some(mut log) = world.get_resource_mut::<MessageLog>() {
        if tech_unlocked {
             log.add(format!("EUREKA! Doing {:?} unlocked {:?}!", action, related_tech.unwrap()));
        } else {
             log.add(format!("Eureka! Gained insight ({}) while {:?}!", knowledge_gained, action));
        }
    }

    true
}
```

### 3. Integration Points

Identify where to call this function. Builders should integrate it into existing execution systems.

- **Mining/Logging/Building**: In `work_execution_system` (or wherever `ActionType::Work` is processed).
    - Call `check_for_eureka(world, ActionType::Work, Some(Tech::Masonry))` (for Mining/Building).
- **Farming**: In `farm_execution_system`.
    - Call `check_for_eureka(world, ActionType::Farm, Some(Tech::Hydroponics))`.
- **Refining**: In `process_refining_system`.
    - Call `check_for_eureka(world, ActionType::Refine, Some(Tech::MetalWorking))`.

## REFACTOR Phase: Quality & Design

- **Event System**: Instead of mutating resources directly inside `check_for_eureka`, emit a `EurekaEvent` and handle it in a separate system. This decouples logic and makes testing easier.
- **Traits**: Factor in Pop Traits. `Intellectual` or `Creative` pops should have a higher `base_chance`.
- **Diminishing Returns**: Prevent players from grinding low-level tasks just for Eureka moments. Add a cooldown or diminishing chance per pop.
- **Visuals**: Spawn a light bulb particle effect above the Pop's head when a Eureka moment occurs.

## Acceptance Criteria

- [ ] `EurekaConfig` resource exists and is registered.
- [ ] `check_for_eureka` function is implemented and tested.
- [ ] Working `Mining` or `Building` has a chance to unlock `Masonry`.
- [ ] Working `Farm` has a chance to unlock `Hydroponics`.
- [ ] If the Tech is already unlocked, the player receives Knowledge points instead.
- [ ] All RED phase tests pass.
- [ ] Integration with existing systems (Work, Farm, Refine) is complete.

## Questions

- Should "Eureka" unlock the tech fully, or just provide a massive discount to its research cost? (Spec currently assumes full unlock for simplicity, but consider balance).
- *Architect:* Full unlock for simplicity in the MVP. The rarity of the event itself provides the balance.

# 485 - The Orphaned AI Broker

## Overview

A Layer 3 entity (an ancient AI trader) arrives and opens trade. It asks for bizarre, low-value items in massive quantities (e.g., 10,000 units of basic wood, or 500 left shoes). If fulfilled, it rewards you with extremely advanced, irreplaceable Layer 3 tech. However, its requests escalate in strange ways, eventually asking for living Pops with specific traits or the dismantling of your main power grid.

The tension comes from the lure of overpowered precursor technology vs. the destabilization of your core economy to fulfill absurd, escalating demands.

## Dependencies

- `039 Trade System` (base trade mechanics)
- `146 Command Center & System Visibility` (detection of Layer 3 entities)
- `051 Pop Skills and Experience` (for Pop-specific demands)
- `084 Pop Traits` (for Pop-specific demands)

## RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::pop::{Pop, PopBundle, Traits};
    use crate::layer1::inventory::Inventory;
    use crate::layer1::resources::ResourceType;
    use crate::layer3::diplomacy::Faction;

    #[test]
    fn test_ai_broker_spawn() {
        let mut world = World::new();
        // Arrange
        // (Setup required resources, e.g., SimulationTime)

        // Act
        // Run system to potentially spawn the AI broker event
        // check_ai_broker_spawn_system(&mut world);

        // Assert
        // Verify an AIBroker entity/resource exists
    }

    #[test]
    fn test_ai_broker_mundane_demand() {
        // Arrange
        let mut world = World::new();
        // Insert AI broker requesting 10,000 Wood

        // Act
        // Fulfill the demand from colony inventory

        // Assert
        // Verify colony loses 10,000 Wood
        // Verify colony receives a PrecursorTech item
    }

    #[test]
    fn test_ai_broker_escalation() {
        // Arrange
        let mut world = World::new();
        // Insert AI broker at escalation level 0

        // Act
        // Fulfill the first demand

        // Assert
        // Verify escalation level is now 1
        // Verify the new demand is significantly more disruptive (e.g., requires a specific Pop)
    }

    #[test]
    fn test_ai_broker_pop_demand() {
        // Arrange
        let mut world = World::new();
        // Spawn a Pop with a specific trait
        // Insert AI broker demanding a Pop with that trait

        // Act
        // Fulfill the demand

        // Assert
        // Verify the Pop is removed from the colony
        // Verify the reward is granted
    }
}
```

## GREEN Phase: Minimal Implementation

```rust
// src/layer3/trade/ai_broker.rs

use bevy::prelude::*;
use crate::layer1::resources::ResourceType;

#[derive(Component, Debug, Clone)]
pub struct AIBroker {
    pub escalation_level: u32,
    pub current_demand: BrokerDemand,
    pub current_reward: BrokerReward,
}

#[derive(Debug, Clone)]
pub enum BrokerDemand {
    Resource { resource_type: ResourceType, amount: u32 },
    // PopWithTrait { trait_type: TraitType },
    // DismantleBuilding { building_type: BuildingType },
}

#[derive(Debug, Clone)]
pub enum BrokerReward {
    PrecursorTech,
    // Add other rewards
}

// System to handle fulfilling demands
pub fn fulfill_broker_demand_system(
    mut commands: Commands,
    mut broker_query: Query<(Entity, &mut AIBroker)>,
    // mut inventory: ResMut<ColonyInventory>,
    // mut events: EventWriter<BrokerTradeCompletedEvent>,
) {
    // Minimal logic to check if demand is met, deduct resources, grant reward, and escalate
}
```

## REFACTOR Phase: Quality & Design

- **Demand Generation**: The logic for generating new demands based on `escalation_level` should be robust and ensure the demands are genuinely disruptive but possible. Consider using a weighted random table that scales with level.
- **UI Integration**: Ensure the strange nature of the demands is clearly communicated in the UI. The text should sound archaic and slightly broken.
- **Lore Connection**: Connect the AI broker to a specific precursor civilization in the lore files.
- **Consequences**: Consider what happens if a demand is ignored for too long. Does the AI broker leave? Does it get angry?

## Acceptance Criteria

- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] The AI Broker spawns, demands strange items, provides high-tier rewards, and escalates its demands upon fulfillment.

## Technical Guidance

- Integrate the `AIBroker` entity with the existing `Trade System` (Spec 039). It might be treated as a special type of merchant or a unique Layer 3 faction interaction.
- The `BrokerDemand` enum will need to be flexible enough to handle various types of disruptive requests (resources, pops, buildings).
- Ensure that removing Pops or dismantling buildings as part of a trade deal properly triggers all necessary cleanup logic (e.g., removing them from jobs, triggering grief if applicable).

## Questions

*Builder: add questions here if spec is unclear. Architect will address.*

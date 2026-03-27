# 651: The Memory Market

## Overview

A grim black market where desperate colonists trade their cherished pasts for survival, creating a fractured society of amnesiacs and memory-hoarders. Pops can extract their `Memories` (using a black-market "Memory Siphon") and sell them to satisfy immediate, critical Needs (like Hunger or Rest). Wealthy Pops or officials can purchase these memories to instantly boost their own UtilityWeights or temporarily cure severe mental breakdowns, essentially "consuming" another Pop's life experiences. A starving miner sells the memory of their child's birth to afford rations. Later, they interact with that child but lack the critical social bond, causing massive family friction. Meanwhile, the colony's wealthy governor walks around with the stolen nostalgia of a hundred different childhoods, eventually suffering from severe personality fragmentation and erratic behavior.

## Dependencies

- None explicitly required beyond core memory and needs systems.

## RED Phase: Tests First

```rust
// src/layer1/economy/memory_market.rs

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::pre::App;

    #[test]
    fn test_extract_memory_for_needs() {
        // Arrange
        let mut app = App::new();
        // Setup a Pop with a memory and high Hunger

        // Act
        // The Pop uses the "Memory Siphon" to sell their memory

        // Assert
        // Expect the Pop to lose the memory and have their Hunger reduced
    }

    #[test]
    fn test_purchase_memory_boosts_utility() {
        // Arrange

        // Act
        // A wealthy Pop purchases a memory from the market

        // Assert
        // Expect the purchasing Pop to gain the memory and a temporary UtilityWeight boost
    }

    #[test]
    fn test_missing_memory_causes_friction() {
        // Arrange

        // Act
        // A Pop interacts with a relative after selling a shared memory

        // Assert
        // Expect the interaction to have a negative outcome due to the lack of shared bond
    }
}
```

## GREEN Phase: Minimal Implementation

```rust
// src/layer1/economy/memory_market.rs

// Minimal event and system structures to make the tests pass.
// e.g., define MemorySiphonEvent, ExtractedMemory component, etc.
```

## REFACTOR Phase: Quality & Design

- Ensure the market value of different memories is balanced against their utility and rarity.
- Track the ownership and history of memories as they are traded on the market.
- Implement the long-term consequences of memory trading, such as identity fragmentation.

## Acceptance Criteria (Testable!)

- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Pops can extract their memories to satisfy critical needs.
- [ ] Pops can purchase memories to gain utility boosts or cure mental breakdowns.
- [ ] Missing critical memories during social interactions causes friction.

## Technical Guidance

- Use Bevy's ECS to manage the transfer of memories between Pops and the market.
- The `MemoryMarket` resource should act as a central hub for trading memories.
- The Utility AI system will need to be updated to consider the option of selling memories when critical needs are unmet.

## Questions

*Builder: add questions here if spec is unclear.*

# 218: Improvised Tools

## Overview

To prevent "death spirals" where a lack of tools prevents gathering the resources needed to make tools, Pops can now **improvise** tools using raw materials (Stone, Wood, Scrap).

Improvised tools allow work to proceed at **75% speed** (compared to 100% with real tools, and 50% with bare hands), but they **consume raw materials** directly from the colony inventory or the local tile during use.

## Dependencies

- `030` — Tool Economy (Verified Implemented)
- `018` — Mining and Resources (Verified Implemented)
- `016` — Utility AI System (Verified Implemented)

## RED Phase: Tests First

Write these tests in `src/layer1/improvised_tools_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::resources::ColonyResources;
    use crate::layer1::execution::{calculate_work_efficiency, WorkEfficiencyResult};
    use crate::layer1::item::ResourceType;

    // 1. Test efficiency with Tools (Baseline)
    #[test]
    fn test_efficiency_with_tools() {
        let mut resources = ColonyResources::default();
        resources.tools = 10.0;

        let (efficiency, consumed) = calculate_work_efficiency(&mut resources);

        assert_eq!(efficiency, 1.0);
        assert_eq!(consumed, Some(ResourceType::Tools)); // Tools degrade slowly
    }

    // 2. Test efficiency with Stone (Improvised)
    #[test]
    fn test_efficiency_improvised_stone() {
        let mut resources = ColonyResources::default();
        resources.tools = 0.0;
        resources.stone = 10.0;

        let (efficiency, consumed) = calculate_work_efficiency(&mut resources);

        assert_eq!(efficiency, 0.75);
        assert_eq!(consumed, Some(ResourceType::Stone));
    }

    // 3. Test efficiency with Wood (Improvised)
    #[test]
    fn test_efficiency_improvised_wood() {
        let mut resources = ColonyResources::default();
        resources.tools = 0.0;
        resources.stone = 0.0;
        resources.wood = 10.0;

        let (efficiency, consumed) = calculate_work_efficiency(&mut resources);

        assert_eq!(efficiency, 0.75);
        assert_eq!(consumed, Some(ResourceType::Wood));
    }

    // 4. Test efficiency with Scrap (Improvised)
    #[test]
    fn test_efficiency_improvised_scrap() {
        let mut resources = ColonyResources::default();
        resources.tools = 0.0;
        resources.stone = 0.0;
        resources.wood = 0.0;
        resources.scrap = 10.0;

        let (efficiency, consumed) = calculate_work_efficiency(&mut resources);

        assert_eq!(efficiency, 0.75);
        assert_eq!(consumed, Some(ResourceType::Scrap));
    }

    // 5. Test fallback to Bare Hands
    #[test]
    fn test_efficiency_bare_hands() {
        let mut resources = ColonyResources::default();
        resources.tools = 0.0;
        resources.stone = 0.0;
        resources.wood = 0.0;
        resources.scrap = 0.0;

        let (efficiency, consumed) = calculate_work_efficiency(&mut resources);

        assert_eq!(efficiency, 0.5);
        assert_eq!(consumed, None);
    }

    // 6. Test Consumption Logic
    #[test]
    fn test_consumption_logic() {
        // Mock RNG or force breakage
        let mut resources = ColonyResources::default();
        resources.stone = 10.0;

        // Simulate "Using" the improvised tool
        // If the system calls consume(ResourceType::Stone, 1.0)
        resources.consume(ResourceType::Stone, 1.0);

        assert_eq!(resources.stone, 9.0);
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Update Work Execution Logic

In `src/layer1/execution/general_work.rs` (or create `src/layer1/execution/efficiency.rs`):

```rust
use crate::layer1::resources::ColonyResources;
use crate::layer1::item::ResourceType;

pub fn calculate_work_efficiency(res: &ColonyResources) -> (f32, Option<ResourceType>) {
    // 1. Proper Tools (Best)
    if res.tools >= 1.0 {
        return (1.0, Some(ResourceType::Tools));
    }

    // 2. Improvised Tools (Okay)
    // Priority: Scrap > Stone > Wood (Arbitrary or based on value)
    if res.scrap >= 1.0 {
        return (0.75, Some(ResourceType::Scrap));
    }
    if res.stone >= 1.0 {
        return (0.75, Some(ResourceType::Stone));
    }
    if res.wood >= 1.0 {
        return (0.75, Some(ResourceType::Wood));
    }

    // 3. Bare Hands (Slow)
    (0.5, None)
}
```

### 2. Update `work_execution_system`

```rust
// In src/layer1/execution/general_work.rs

pub fn work_execution_system(
    mut commands: Commands,
    mut resources: ResMut<ColonyResources>,
    // ... queries ...
) {
    // Calculate global efficiency factor based on resources
    // Note: This assumes all workers share the global resource pool
    let (efficiency_mult, consumed_type) = calculate_work_efficiency(&resources);

    // ... loop over workers ...
    for (entity, mut worker, job) in worker_query.iter_mut() {
        // Apply efficiency to work progress
        let progress = BASE_WORK_RATE * efficiency_mult;
        // ... apply progress ...

        // Roll for consumption/breakage
        if let Some(rtype) = consumed_type {
             let break_chance = match rtype {
                 ResourceType::Tools => 0.01, // 1% chance per tick
                 _ => 0.05, // 5% chance for improvised (breaks fast)
             };

             if rand::thread_rng().gen_bool(break_chance) {
                 // Try to consume. If fails (race condition), efficiency drops next tick.
                 if resources.get(rtype) >= 1.0 {
                     resources.consume(rtype, 1.0);
                     // Optional: Spawn particle effect or log "Tool Broke!"
                 }
             }
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **Material Hardness**: Using Wood to mine Stone should be less efficient (e.g., 0.6) than using Stone to mine Stone (0.75).
- **Consumption Rate**: Tune the break chance. Improvised tools should feel "crunchy" - they break often.
- **UI Feedback**: When efficiency < 1.0, show a warning icon on the job progress bar or worker.
- **AI Planning**: Utility AI should factor in the cost of burning 10 Stone to mine 10 Stone. (Net zero gain).

## Acceptance Criteria

- [ ] `calculate_work_efficiency` returns correct multipliers (1.0, 0.75, 0.5).
- [ ] Working without tools consumes Stone, Wood, or Scrap if available.
- [ ] Speed drops to 50% if inventory is truly empty.
- [ ] Tests pass.

## Technical Guidance

- Be careful with `ResMut<ColonyResources>`. If you borrow it mutably for the whole system, you can't query it inside.
- Better pattern: Copy needed values (tool count) to local variables before the loop. Record consumption events in a `Vec<ResourceType>` and apply them in batch after the loop.

## Questions

*Builder: Should "Scrap" be better than "Stone"?*
*Architect: Scrap and Stone tools both have 0.75 efficiency for MVP.*

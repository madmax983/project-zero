# 085: Faction Demands and Strikes

## Overview

Factions (introduced in `068`) now actively push for changes in the colony.
If Faction Satisfaction drops below a threshold, the Faction issues a **Demand** (e.g., "Enact Policy X", "Build more Housing").
If the Demand is ignored for too long, the Faction enters a **Strike** state.
During a Strike, all Faction members refuse to work (Utility for Work actions drops to 0.0), crippling the colony's economy until the demand is met or the strike is broken.

This transforms Factions from passive stat-tracking containers into active political entities that threaten the player's control.

## Dependencies

- `068` — Pop Factions (Faction data and membership)
- `054` — Colony Edicts (Policies as potential demands)
- `016` — Utility AI (Strike overrides work utility)
- `046` — Notifications System (To alert player of demands)

## RED Phase: Tests First

Write these tests in `src/layer1/faction_demands_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::factions::{
        Factions, FactionId, FactionData, FactionState, FactionDemand,
        update_faction_demands_system, update_faction_strikes_system
    };
    use crate::layer1::edicts::{ColonyPolicies, Policy};
    use crate::layer1::utility_ai::{ActionType, UtilityWeights, evaluate_work};
    use crate::layer1::{Pop, GridPosition};
    use crate::layer1::pop::MentalState;

    fn setup_world() -> World {
        let mut world = World::new();
        let mut factions = Factions::default();
        factions.initialize();
        world.insert_resource(factions);
        world.insert_resource(ColonyPolicies::default());
        world.insert_resource(crate::shared::time::SimulationTime::default());
        world
    }

    #[test]
    fn test_faction_state_defaults_to_loyal() {
        let world = setup_world();
        let factions = world.resource::<Factions>();
        let data = factions.get(FactionId::MinersGuild).unwrap();
        assert_eq!(data.state, FactionState::Loyal);
    }

    #[test]
    fn test_generate_demand_on_low_satisfaction() {
        let mut world = setup_world();
        let mut factions = world.resource_mut::<Factions>();

        // Lower satisfaction to threshold (e.g., 0.4)
        if let Some(data) = factions.map.get_mut(&FactionId::MinersGuild) {
            data.satisfaction = 0.3;
        }

        // Run demand generation system
        update_faction_demands_system(&mut world);

        let factions = world.resource::<Factions>();
        let data = factions.get(FactionId::MinersGuild).unwrap();

        // Should have a demand
        assert!(data.active_demand.is_some());
        // State should be Unhappy
        assert_eq!(data.state, FactionState::Unhappy);
    }

    #[test]
    fn test_demand_timeout_triggers_strike() {
        let mut world = setup_world();

        // Setup existing demand with near-expiry timeout
        {
            let mut factions = world.resource_mut::<Factions>();
            let data = factions.map.get_mut(&FactionId::MinersGuild).unwrap();
            data.satisfaction = 0.3;
            data.state = FactionState::Unhappy;
            data.active_demand = Some(FactionDemand {
                policy: Some(Policy::Rationing), // e.g. Demand "Stop Rationing" or "Start Rationing"
                remaining_time: 1.0, // 1 tick remaining
                ..Default::default()
            });
        }

        // Run strike update system
        update_faction_strikes_system(&mut world);

        let factions = world.resource::<Factions>();
        let data = factions.get(FactionId::MinersGuild).unwrap();

        // Should transition to Strike
        assert_eq!(data.state, FactionState::Striking);
    }

    #[test]
    fn test_strike_overrides_work_utility() {
        // This test ensures striking pops don't work
        let mut world = setup_world();

        // Set MinersGuild to Strike
        {
            let mut factions = world.resource_mut::<Factions>();
            let data = factions.map.get_mut(&FactionId::MinersGuild).unwrap();
            data.state = FactionState::Striking;
        }

        // Spawn a Pop in MinersGuild
        let pop = world.spawn((
            Pop,
            crate::layer1::factions::FactionMember { faction_id: Some(FactionId::MinersGuild) },
            GridPosition { x: 0, y: 0 },
            MentalState::Normal,
        )).id();

        // Evaluate Work (mock inputs)
        // Note: You might need to mock Query inputs for evaluate_work if it takes them.
        // Alternatively, test a helper function `is_striking(&World, Entity) -> bool`.

        let is_striking = crate::layer1::factions::is_pop_striking(&world, pop);
        assert!(is_striking);

        // If integrated into evaluate_work:
        // let utility = evaluate_work(...);
        // assert_eq!(utility, 0.0);
    }

    #[test]
    fn test_meeting_demand_resolves_strike() {
        let mut world = setup_world();

        // Set MinersGuild to Strike with a specific demand
        {
            let mut factions = world.resource_mut::<Factions>();
            let data = factions.map.get_mut(&FactionId::MinersGuild).unwrap();
            data.state = FactionState::Striking;
            data.active_demand = Some(FactionDemand {
                policy: Some(Policy::DoubleShifts), // Demand: Toggle DoubleShifts (Assumed: "End Double Shifts")
                ..Default::default()
            });
        }

        // Assume DoubleShifts is currently ACTIVE, so they want it ENDED (toggled off).
        world.resource_mut::<ColonyPolicies>().active_policies.insert(Policy::DoubleShifts);

        // Player action: Toggle DoubleShifts OFF
        world.resource_mut::<ColonyPolicies>().toggle(Policy::DoubleShifts);

        // Run update system
        update_faction_demands_system(&mut world);

        let factions = world.resource::<Factions>();
        let data = factions.get(FactionId::MinersGuild).unwrap();

        // Should resolve
        assert!(data.active_demand.is_none());
        assert_eq!(data.state, FactionState::Loyal); // Or recovered to Normal/Loyal
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Update `FactionData` and Enums

`src/layer1/factions.rs`

```rust
use crate::layer1::edicts::Policy;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FactionState {
    Loyal,    // Satisfaction > 0.4
    Unhappy,  // Satisfaction <= 0.4, Demand issued
    Striking, // Demand timed out
}

#[derive(Debug, Clone, PartialEq)]
pub struct FactionDemand {
    pub policy: Option<Policy>, // The policy they want changed (Toggle)
    pub remaining_time: f32,    // Ticks until strike
    pub description: String,
}

impl Default for FactionDemand {
    fn default() -> Self {
        Self {
            policy: None,
            remaining_time: 1000.0, // Default duration
            description: "None".into(),
        }
    }
}

// Update FactionData
#[derive(Debug, Clone)]
pub struct FactionData {
    pub name: String,
    pub satisfaction: f32,
    pub members_count: usize,
    // New fields
    pub state: FactionState,
    pub active_demand: Option<FactionDemand>,
}

// Update Default for FactionData
impl Default for FactionData {
    fn default() -> Self {
        Self {
            // ... existing
            state: FactionState::Loyal,
            active_demand: None,
        }
    }
}
```

### 2. Implement `update_faction_demands_system`

```rust
pub fn update_faction_demands_system(
    mut factions: ResMut<Factions>,
    policies: Res<ColonyPolicies>,
    // Add Notification resource here if available
) {
    for (id, data) in factions.map.iter_mut() {
        // 1. Resolve existing demands if met
        if let Some(demand) = &data.active_demand {
            let met = if let Some(policy) = demand.policy {
                // Simplification: If demand implies "Stop Policy", check if it's inactive.
                // For GREEN phase, let's assume random policy demand means "Toggle this".
                // In reality, we need `DemandType::Enact` vs `DemandType::Revoke`.
                // Let's assume for now the demand is satisfied if the Policy state FLIPS from when it was demanded.
                // Better: Just check satisfaction. If satisfaction > 0.5, clear demand.

                // For simplicity in Green: Demand is cleared if satisfaction rises > 0.5.
                data.satisfaction > 0.5
            } else {
                false
            };

            if met {
                data.active_demand = None;
                data.state = FactionState::Loyal;
                // Log: "Faction X is content."
                continue;
            }
        }

        // 2. Generate new demand if Unhappy and None
        if data.satisfaction < 0.4 && data.active_demand.is_none() {
            // Pick a demand. For Green, hardcode a dummy or random policy.
            // Example: Demand "Rationing" if it's off, or "No Rationing" if it's on.
            // Just picking Policy::DoubleShifts as placeholder.

            data.active_demand = Some(FactionDemand {
                policy: Some(Policy::DoubleShifts),
                remaining_time: 2000.0, // ~1 day?
                description: "Change Shift Policy".into(),
            });
            data.state = FactionState::Unhappy;
            // Log: "Faction X demands change!"
        }
    }
}
```

### 3. Implement `update_faction_strikes_system`

```rust
pub fn update_faction_strikes_system(mut factions: ResMut<Factions>) {
    for data in factions.map.values_mut() {
        if let Some(demand) = &mut data.active_demand {
            if demand.remaining_time > 0.0 {
                demand.remaining_time -= 1.0;
            } else if data.state == FactionState::Unhappy {
                data.state = FactionState::Striking;
                // Log: "Faction X is STRIKING!"
            }
        }
    }
}
```

### 4. Helper `is_pop_striking`

```rust
pub fn is_pop_striking(world: &World, entity: Entity) -> bool {
    if let Some(member) = world.get::<FactionMember>(entity) {
        if let Some(faction_id) = member.faction_id {
            if let Some(factions) = world.get_resource::<Factions>() {
                if let Some(data) = factions.get(faction_id) {
                    return data.state == FactionState::Striking;
                }
            }
        }
    }
    false
}
```

### 5. Update `evaluate_work` (Utility AI)

Builder needs to locate `evaluate_work` (usually in `src/layer1/utility_ai/actions/work.rs` or similar) and add:

```rust
if is_pop_striking(world, entity) {
    return 0.0;
}
```

## REFACTOR Phase: Quality & Design

- **Demand Logic**: Implement smart demand selection.
    - If `Hunger` is high -> Demand `Rationing` (if off) or `DoubleRations` (if implemented).
    - If `Tired` -> Demand `Revoke DoubleShifts`.
- **UI**: Add "Demands" tab to Faction UI. Show countdown timer.
- **Strike Breakers**: Allow Player to "Crack Down" (Action) via Militia to force them back to work (increases Unrest/Satisfaction penalty).
- **Strike Effects**: Visual indicator (Pop icon turns red or picket sign).

## Acceptance Criteria

- [ ] `FactionData` tracks `state` and `active_demand`.
- [ ] Low satisfaction triggers demand generation.
- [ ] Ignored demands lead to `Striking` state.
- [ ] Striking pops return 0.0 utility for Work actions.
- [ ] Meeting demands (or raising satisfaction) resolves the strike.
- [ ] Tests pass.

## Technical Guidance

- **Performance**: `is_pop_striking` involves resource lookups. It's fast enough for `evaluate_work` (which runs per pop per tick or interval), but avoid deep cloning `FactionData`.
- **Events**: Use `bevy::event::EventWriter` for notifications (e.g. `Event<FactionNotification>`).
- **Time**: Use `SimulationTime` for demand expiry instead of raw tick decrement if possible, or just raw ticks for simplicity in Green.

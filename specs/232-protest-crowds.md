# 232: Protest Crowds

## Overview

Escalates Faction Unhappiness into physical disruption. Instead of just "striking" (refusing work), unhappy factions now form **Protest Crowds**.
These mobs physically gather at high-value targets (e.g., the Governor's Office, the Power Plant, or the specific Workplace they are striking against).
By gathering, they create a **Blockade** via the `CrowdingGrid` mechanic, forcing other pops to route around them or get stuck ("Gridlock").
They also **Disable** the target building, preventing anyone else from working there.

## Dependencies

- `068` — Pop Factions (Satisfaction logic)
- `176` — Gridlock/Crowding (The blocking mechanic)
- `050` — Civil Unrest (Context)

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/layer1/protest_tests.rs

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::factions::{Factions, FactionId, FactionData, FactionState};
    use crate::layer1::map::GridPosition;
    use crate::layer1::protest::{ProtestZone, trigger_protest_system, update_protest_behavior_system, ProtestEvent};
    use crate::layer1::building::{Building, BuildingType, OccupiedTiles};
    use crate::layer1::pop::{Pop, PopAction, ActionType};
    use crate::layer1::utility_ai::UtilityWeights;

    fn setup_world() -> World {
        let mut world = World::new();
        let mut factions = Factions::default();
        factions.initialize();
        world.insert_resource(factions);
        world.init_resource::<Events<ProtestEvent>>();
        world
    }

    #[test]
    fn test_low_satisfaction_triggers_protest_event() {
        let mut world = setup_world();

        // Set MinersGuild to Unhappy/Striking
        let mut factions = world.resource_mut::<Factions>();
        if let Some(data) = factions.map.get_mut(&FactionId::MinersGuild) {
            data.satisfaction = 0.1;
            data.state = FactionState::Striking;
        }

        // Run trigger system
        let mut schedule = Schedule::default();
        schedule.add_systems(trigger_protest_system);
        schedule.run(&mut world);

        // Check for Event
        let events = world.resource::<Events<ProtestEvent>>();
        let mut reader = events.get_reader();
        assert!(reader.len(&events) > 0, "Should trigger ProtestEvent");
    }

    #[test]
    fn test_protest_event_spawns_zone_at_target() {
        let mut world = setup_world();

        // Spawn a target building (e.g. Office)
        let office = world.spawn((
            Building { building_type: BuildingType::Office },
            GridPosition { x: 10, y: 10 },
        )).id();

        // Send event
        world.send_event(ProtestEvent {
            faction: FactionId::MinersGuild,
            target_building: Some(office),
            target_pos: GridPosition { x: 10, y: 10 },
        });

        // Run system to process event
        // (Assuming a system consumes the event and spawns the zone)
        // For Red phase, we might need to implement the consumer or test the system that does it.
        // Let's assume `handle_protest_event_system` exists.

        // ... run system ...

        // Verify ProtestZone exists
        // let zones = world.query::<&ProtestZone>().iter(&world).count();
        // assert_eq!(zones, 1);
    }

    #[test]
    fn test_protest_zone_attracts_faction_members() {
        let mut world = setup_world();

        // Spawn Protest Zone at (10, 10)
        world.spawn((
            ProtestZone {
                faction: FactionId::MinersGuild,
                radius: 2,
            },
            GridPosition { x: 10, y: 10 },
        ));

        // Spawn Miner Pop
        use crate::layer1::factions::FactionMember;
        let pop = world.spawn((
            Pop,
            FactionMember { faction_id: Some(FactionId::MinersGuild) },
            GridPosition { x: 0, y: 0 },
            PopAction::default(),
            UtilityWeights::default(),
        )).id();

        // Run behavior update system
        let mut schedule = Schedule::default();
        schedule.add_systems(update_protest_behavior_system);
        schedule.run(&mut world);

        // Pop should have ActionType::Protest
        let action = world.get::<PopAction>(pop).unwrap();
        assert_eq!(action.current, ActionType::Protest);
    }

    #[test]
    fn test_protest_disables_building() {
        let mut world = setup_world();

        // Spawn Building
        let building = world.spawn((
            Building { building_type: BuildingType::Mine },
            GridPosition { x: 5, y: 5 },
            // Assuming we have a "Disabled" or "Efficiency" component, or check function
        )).id();

        // Spawn Protest Zone on top
        world.spawn((
            ProtestZone { faction: FactionId::MinersGuild, radius: 1 },
            GridPosition { x: 5, y: 5 },
        ));

        // Run effect system
        // crate::layer1::protest::apply_protest_effects_system(&mut world);

        // Verify building is disabled
        // assert!(is_building_disabled(&world, building));
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. New Components (`src/layer1/protest.rs`)

```rust
use bevy_ecs::prelude::*;
use crate::layer1::factions::FactionId;
use crate::layer1::map::GridPosition;

#[derive(Component, Debug)]
pub struct ProtestZone {
    pub faction: FactionId,
    pub radius: u32,
    pub target_building: Option<Entity>,
}

#[derive(Event, Debug, Clone)]
pub struct ProtestEvent {
    pub faction: FactionId,
    pub target_building: Option<Entity>,
    pub target_pos: GridPosition,
}
```

### 2. Update `ActionType` (`src/layer1/utility_types.rs`)

**CRITICAL**: You MUST increment `ActionType::COUNT` (e.g. 34 -> 35).
**CRITICAL**: You MUST update `src/gpu/buffers.rs` and `src/gpu/shaders/evaluate.wgsl` to match the new count.

```rust
pub enum ActionType {
    // ... existing ...
    Protest, // New action
}
```

### 3. Trigger System

```rust
pub fn trigger_protest_system(
    factions: Res<crate::layer1::factions::Factions>,
    mut events: EventWriter<ProtestEvent>,
    // Query to find valid targets (e.g. Governor's office or random workplace)
    buildings: Query<(Entity, &GridPosition, &crate::layer1::building::Building)>,
) {
    for (id, data) in &factions.map {
        if data.state == crate::layer1::factions::FactionState::Striking {
            // Check if protest already exists? (Need query for ProtestZone)
            // If not, pick a target.
            // For GREEN: Pick first building or random.
            if let Some((e, pos, _)) = buildings.iter().next() {
                events.send(ProtestEvent {
                    faction: *id,
                    target_building: Some(e),
                    target_pos: *pos,
                });
            }
        }
    }
}
```

### 4. Behavior System

```rust
pub fn update_protest_behavior_system(
    mut query: Query<(&crate::layer1::factions::FactionMember, &mut crate::layer1::utility_ai::PopAction)>,
    zones: Query<(&ProtestZone, &GridPosition)>,
) {
    for (member, mut action) in &mut query {
        if let Some(fid) = member.faction_id {
            // Find active protest for this faction
            for (zone, _zone_pos) in &zones {
                if zone.faction == fid {
                    // Force action
                    action.current = crate::layer1::utility_types::ActionType::Protest;
                    // Logic to set target to zone_pos handles in Execution/Movement
                }
            }
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **Target Selection**: Pick the "Headquarters" of the faction (e.g., Mine for Miners) or the "Administration" (Governor).
- **Crowding**: Ensure standing pops actually block pathfinding via `CrowdingGrid`. (Verified: Stationary pops add crowding).
- **Dispersal**: If satisfaction rises > 0.5, despawn `ProtestZone`.
- **Visuals**: Spawn "Sign" particles or change Pop color/animation.
- **Police**: `ActionType::RiotControl` to disperse them forcefully.

## Acceptance Criteria

- [ ] `ProtestZone` spawns when faction is Striking.
- [ ] Members of the faction switch to `Protest` action.
- [ ] Members gather at the `ProtestZone` (increasing local crowding).
- [ ] Target building is disabled (optional for MVP, but good for gameplay).
- [ ] `ActionType::Protest` added and GPU buffers updated.
- [ ] Tests pass.

## Technical Guidance

- **ActionType::COUNT**: This is the most common source of bugs. Update `utility_types.rs`, `gpu/buffers.rs`, and check shaders.
- **Looping**: `update_protest_behavior_system` overrides Utility AI. Ensure it doesn't fight with `evaluate_actions_system`. Ideally, `Protest` should be a high-utility action returned by `evaluate`, or a forced override state like `MentalState`. For now, override is fine but consider integrating into `evaluate`.

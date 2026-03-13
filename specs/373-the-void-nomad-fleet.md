# 373: The Void Nomad Fleet

## Overview

"A city that never stops moving."

**The Void Nomad Fleet** introduces a wandering entity on the Layer 2 map that occasionally passes through your system. They are peaceful but strip-mine asteroids and moons in their path. You can trade with them, but if they enter your system, they might consume resources you needed.

This creates tension: beneficial trade opportunities vs. inevitable resource depletion. You might try to attack them to protect your asteroid mines, only to discover their seemingly ramshackle ships possess overwhelming, ancient weaponry.

## Dependencies

- `099` — Fleet Movement (for the Nomad fleet logic)
- `039` — Trade System (for interacting with the fleet)

## RED Phase: Tests First

Write these tests in `src/layer2/entities/nomad_fleet_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer2::map::{SystemNode, NodeType, SystemPosition};
    use crate::layer2::entities::nomad_fleet::{NomadFleet, process_nomad_mining_system, NomadArrivalEvent};
    use crate::layer2::resources::{NodeResources, ResourceType};

    #[test]
    fn test_nomad_fleet_arrives() {
        let mut world = World::new();
        world.init_resource::<Events<NomadArrivalEvent>>();

        let mut schedule = Schedule::default();
        // A system to trigger arrival based on time/probability
        schedule.add_systems(trigger_nomad_arrival_system);
        schedule.run(&mut world);

        let events = world.resource::<Events<NomadArrivalEvent>>();
        let mut reader = events.get_reader();
        assert!(reader.read(events).next().is_some());
    }

    #[test]
    fn test_nomads_strip_mine_asteroids() {
        let mut world = World::new();

        // Target asteroid
        let pos = SystemPosition { x: 5, y: 5 };
        let asteroid = world.spawn((
            SystemNode { node_type: NodeType::Asteroid },
            pos,
            NodeResources { metal: 1000.0, volatiles: 500.0 },
        )).id();

        // Nomad fleet at the asteroid
        let fleet = world.spawn((
            NomadFleet { mining_rate: 100.0 },
            pos,
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(process_nomad_mining_system);
        schedule.run(&mut world);

        // Resources should be depleted
        let resources = world.get::<NodeResources>(asteroid).unwrap();
        assert!(resources.metal < 1000.0);
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Components

```rust
// src/layer2/entities/nomad_fleet.rs

use bevy_ecs::prelude::*;
use crate::layer2::map::{SystemPosition, NodeType};

#[derive(Component, Debug, Clone)]
pub struct NomadFleet {
    pub mining_rate: f32,
}

#[derive(Event, Debug, Clone)]
pub struct NomadArrivalEvent {
    pub entry_point: SystemPosition,
}
```

### 2. Systems

```rust
use crate::layer2::map::SystemNode;
use crate::layer2::resources::NodeResources;

pub fn trigger_nomad_arrival_system(
    mut events: EventWriter<NomadArrivalEvent>,
    // Time/probability state
) {
    // For GREEN phase, just trigger it randomly
    use rand::Rng;
    let mut rng = rand::thread_rng();
    if rng.gen::<f32>() < 0.01 {
        events.send(NomadArrivalEvent {
            entry_point: SystemPosition { x: 0, y: 0 },
        });
    }
}

pub fn process_nomad_mining_system(
    fleets: Query<(&NomadFleet, &SystemPosition)>,
    mut nodes: Query<(&SystemPosition, &SystemNode, &mut NodeResources)>,
) {
    for (fleet, fleet_pos) in fleets.iter() {
        for (node_pos, node, mut resources) in nodes.iter_mut() {
            if *fleet_pos == *node_pos && node.node_type == NodeType::Asteroid {
                // Strip mine
                resources.metal = (resources.metal - fleet.mining_rate).max(0.0);
                resources.volatiles = (resources.volatiles - (fleet.mining_rate / 2.0)).max(0.0);
            }
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **Pathfinding**: The Nomad fleet should have a specific pathfinding goal to move between asteroid belts rather than random movement.
- **Trade Hub**: When the Nomad fleet arrives at a node, they should open up a temporary `TradeMarket` with unique, high-tier goods.
- **Aggression Reaction**: The Nomad fleet needs overwhelmingly high combat stats, so if the player attacks them to stop the mining, they get destroyed immediately.

## Acceptance Criteria

- [ ] `NomadFleet` entity travels through the system.
- [ ] Nomads deplete `NodeResources` when overlapping an `Asteroid` node.
- [ ] Random arrival events generate the fleet.
- [ ] Tests pass.

## Technical Guidance

- Integrate `process_nomad_mining_system` into a Layer 2 update loop.
- Ensure the `NodeResources` struct and `SystemNode` types match the current Layer 2 design.

## Questions

*Builder: Add any questions here.*
*Architect:* Implement the simplest possible version for the MVP. Advanced behaviors and edge cases will be deferred to future specifications.

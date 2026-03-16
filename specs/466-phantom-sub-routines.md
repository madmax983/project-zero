# 466: The Phantom Sub-routines

## 1. Overview
As your empire relies more on automated ship routes and AI governors (Layer 2/3), small coding errors (represented by "Scrapcode" buildup) begin to aggregate. These errors occasionally spawn "Ghost Fleets"—automated ships that follow bizarre, nonsensical orders (e.g., hauling thousands of tons of dirt to a luxury world, or endlessly patrolling an empty sector). This forces players to manually defragment their empire's code or deal with the logistical bottlenecks and chaos caused by rogue AI directives.

## 2. Dependencies
- `178` Scrapcode (Implemented)
- `099` Fleet Movement (Implemented)
- Automated ship logistics / trade routes framework (Layer 2/3)

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    // Assuming these components exist or will be added based on the dependencies
    // use scale_core::layer2::fleet::{Fleet, FleetOrders, MoveToNodeOrder};
    // use scale_core::layer1::scrapcode::ScrapcodeInfection;

    // Mock components for the tests
    #[derive(Component)]
    struct Fleet {
        pub is_automated: bool,
    }

    #[derive(Component, Debug, PartialEq)]
    enum FleetOrders {
        Idle,
        Haul { target: Entity, resource_type: String, amount: u32 },
        Patrol { target: Entity },
    }

    #[derive(Resource, Default)]
    struct EmpireAutomationState {
        pub scrapcode_buildup: f32,
    }

    #[derive(Event)]
    struct SpawnGhostFleetEvent {
        pub origin: Entity,
        pub bizarre_order: FleetOrders,
    }

    fn check_scrapcode_threshold_system(
        state: Res<EmpireAutomationState>,
        mut ghost_fleet_events: EventWriter<SpawnGhostFleetEvent>,
    ) {
        if state.scrapcode_buildup > 100.0 {
            // Simplified for test: spawn one ghost fleet event
            ghost_fleet_events.send(SpawnGhostFleetEvent {
                origin: Entity::PLACEHOLDER, // In reality, pick a shipyard or node
                bizarre_order: FleetOrders::Patrol { target: Entity::PLACEHOLDER },
            });
        }
    }

    fn spawn_ghost_fleet_system(
        mut commands: Commands,
        mut events: EventReader<SpawnGhostFleetEvent>,
    ) {
        for event in events.read() {
            commands.spawn((
                Fleet { is_automated: true },
                // Requires the order to be cloned or copied in reality
                FleetOrders::Patrol { target: event.origin }, // Simplified
            ));
        }
    }

    #[test]
    fn test_high_scrapcode_spawns_ghost_fleet() {
        let mut app = App::new();
        app.add_event::<SpawnGhostFleetEvent>();
        app.insert_resource(EmpireAutomationState { scrapcode_buildup: 150.0 });

        app.add_systems(Update, (check_scrapcode_threshold_system, spawn_ghost_fleet_system).chain());

        app.update();

        // Verify a ghost fleet was spawned
        let mut query = app.world_mut().query::<(&Fleet, &FleetOrders)>();
        let mut found_ghost_fleet = false;
        for (fleet, orders) in query.iter(app.world()) {
            if fleet.is_automated {
                found_ghost_fleet = true;
                break;
            }
        }

        assert!(found_ghost_fleet, "Ghost fleet should have been spawned due to high scrapcode");
    }

    #[test]
    fn test_low_scrapcode_does_not_spawn_ghost_fleet() {
        let mut app = App::new();
        app.add_event::<SpawnGhostFleetEvent>();
        app.insert_resource(EmpireAutomationState { scrapcode_buildup: 50.0 });

        app.add_systems(Update, (check_scrapcode_threshold_system, spawn_ghost_fleet_system).chain());

        app.update();

        // Verify no ghost fleet was spawned
        let mut query = app.world_mut().query::<&Fleet>();
        let count = query.iter(app.world()).count();
        assert_eq!(count, 0, "No fleets should spawn when scrapcode is low");
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy::prelude::*;

// Stub structures replacing the mocks from the test phase
#[derive(Component)]
pub struct Fleet {
    pub is_automated: bool,
}

#[derive(Component, Debug, Clone, PartialEq)]
pub enum FleetOrders {
    Idle,
    Haul { target: Entity, resource_type: String, amount: u32 },
    Patrol { target: Entity },
}

#[derive(Resource, Default)]
pub struct EmpireAutomationState {
    pub scrapcode_buildup: f32,
    pub spawn_timer: Timer,
}

#[derive(Event)]
pub struct SpawnGhostFleetEvent {
    pub origin_node: Entity,
    pub bizarre_order: FleetOrders,
}

pub fn check_scrapcode_threshold_system(
    time: Res<Time>,
    mut state: ResMut<EmpireAutomationState>,
    mut events: EventWriter<SpawnGhostFleetEvent>,
    nodes: Query<Entity>, // Simplified query to get a valid node
) {
    state.spawn_timer.tick(time.delta());

    if state.scrapcode_buildup > 100.0 && state.spawn_timer.just_finished() {
        if let Some(origin) = nodes.iter().next() {
            events.send(SpawnGhostFleetEvent {
                origin_node: origin,
                // In a real implementation, generate a random nonsensical order
                bizarre_order: FleetOrders::Haul { target: origin, resource_type: "Dirt".to_string(), amount: 10000 },
            });
        }
    }
}

pub fn spawn_ghost_fleet_system(
    mut commands: Commands,
    mut events: EventReader<SpawnGhostFleetEvent>,
) {
    for event in events.read() {
        commands.spawn((
            Fleet { is_automated: true },
            event.bizarre_order.clone(),
            // Add other necessary components like Position, etc.
        ));
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Integration**: `EmpireAutomationState::scrapcode_buildup` needs to increase gradually based on the number of automated routes and active AI governors in the empire. It should also tie into Layer 1 `Scrapcode` events if possible.
- **Order Generation**: The `bizarre_order` generation should be extracted into a helper function that selects a random target and a random (often useless or contradictory) action.
- **Defragmentation**: A player action or specialized facility (e.g., a "Cyber-Core") should be introduced to lower `scrapcode_buildup` at a cost (time, power, admin points).
- **Cleanup**: Ghost fleets should probably eventually run out of fuel, reach their destination and disband, or require the player to manually scuttle them, creating a "clean up" chore.

## 6. Acceptance Criteria (Testable!)
- [ ] If `EmpireAutomationState::scrapcode_buildup` exceeds a threshold, `SpawnGhostFleetEvent`s are periodically emitted.
- [ ] The `spawn_ghost_fleet_system` correctly consumes the events and spawns a new automated `Fleet` entity with the generated bizarre `FleetOrders`.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.

## 7. Technical Guidance
- `EmpireAutomationState` should be updated in a system that runs periodically, perhaps in the `Observation` or `Economy` schedule, scaling the buildup based on the size of the player's automated networks.
- Bizarre orders should genuinely consume hyperlane bandwidth or physical space at nodes, creating the promised logistical bottleneck.
- Ensure the newly spawned ghost fleets use the same movement logic (`099 Fleet Movement`) as normal automated fleets.

## 8. Questions
*Builder: add questions here if spec is unclear.*

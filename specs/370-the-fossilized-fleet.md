# 370 The Fossilized Fleet

## 1. Overview
Strip-mining the bones of a forgotten war. An asteroid belt turns out to be a massive, ancient battlefield. Expeditions can mine it for pristine ancient tech. However, disturbing the wreckage occasionally reactivates "Ghost Drones" that drop down to Layer 1 to attack. This combines high-tier resource acquisition with unpredictable automated reprisals.

## 2. Dependencies
- `101-system-mining.md` (for Layer 2 fleet mining operations)
- `159-fleet-combat-resolution.md` (for Layer 2 Ghost Drones)
- `091-the-inspector.md` (for arriving shuttles dropping enemies on L1)

## 3. RED Phase: Tests First

```rust
// tests/integration/fossilized_fleet_test.rs

use crate::layer2::mining::MiningOperation;
use crate::layer2::system::GhostDroneSpawnEvent;
use crate::layer1::combat::EnemyDropEvent;

#[test]
fn test_mining_fossilized_fleet_spawns_drones() {
    let mut app = setup_test_app();

    // Start mining a fossilized fleet node
    let op = app.world_mut().spawn((
        MiningOperation { resource: ResourceType::AncientTech, ticks: 100 },
        FossilizedFleetNode,
    )).id();

    // Progress mining
    app.update();
    app.update();

    // A drone event should fire occasionally
    let events = app.world().resource::<Events<GhostDroneSpawnEvent>>();
    let mut reader = events.get_reader();
    assert!(reader.iter(&events).next().is_some());
}

#[test]
fn test_ghost_drones_drop_on_layer1() {
    let mut app = setup_test_app();

    // Spawn drone targeting Layer 1
    app.world_mut().send_event(GhostDroneSpawnEvent { count: 3 });
    app.update();

    // Layer 1 should receive drop events
    let drop_events = app.world().resource::<Events<EnemyDropEvent>>();
    let mut reader = drop_events.get_reader();
    assert_eq!(reader.iter(&drop_events).count(), 3);
}

#[test]
fn test_ancient_tech_reward() {
    let mut app = setup_test_app();
    let initial_tech = app.world().resource::<ColonyResources>().get_amount(ResourceType::AncientTech);

    // Complete mining operation
    app.world_mut().send_event(MiningCompleteEvent { resource: ResourceType::AncientTech, amount: 5 });
    app.update();

    let current_tech = app.world().resource::<ColonyResources>().get_amount(ResourceType::AncientTech);
    assert_eq!(current_tech, initial_tech + 5);
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/layer2/mining/fossilized_fleet.rs
use bevy::prelude::*;
use crate::layer1::combat::EnemyDropEvent;

#[derive(Component)]
pub struct FossilizedFleetNode;

#[derive(Event)]
pub struct GhostDroneSpawnEvent {
    pub count: u32,
}

pub fn process_fossilized_mining(
    mut mining_ops: Query<(&mut MiningOperation, &FossilizedFleetNode)>,
    mut event_writer: EventWriter<GhostDroneSpawnEvent>,
) {
    for (op, _) in mining_ops.iter_mut() {
        if op.ticks > 0 && rand::random::<f32>() < 0.05 {
            event_writer.send(GhostDroneSpawnEvent { count: 1 });
        }
    }
}

pub fn handle_ghost_drone_drops(
    mut drone_events: EventReader<GhostDroneSpawnEvent>,
    mut drop_writer: EventWriter<EnemyDropEvent>,
) {
    for event in drone_events.read() {
        for _ in 0..event.count {
            drop_writer.send(EnemyDropEvent { enemy_type: EnemyType::GhostDrone });
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **RNG Control:** Use a seeded RNG or a deterministic threshold instead of pure random to avoid test flakiness and frustrating player experiences.
- **Visuals:** Add drop pod / meteor effects on Layer 1 when Ghost Drones land.
- **Tech Value:** Ensure `AncientTech` is highly valuable to justify the risk.

## 6. Acceptance Criteria
- [ ] Tests pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Coverage >= 85%.
- [ ] Mining `FossilizedFleetNode` randomly generates `GhostDroneSpawnEvent`.
- [ ] `GhostDroneSpawnEvent` triggers an `EnemyDropEvent` on Layer 1.
- [ ] Completing the mining grants `AncientTech`.

## 7. Technical Guidance
- `GhostDroneSpawnEvent` must be handled immediately to drop the enemies on Layer 1 while the player's colony is active.
- Consider making the drones target specific high-value buildings like generators.

## 8. Questions
*Builder: Add any questions here.*

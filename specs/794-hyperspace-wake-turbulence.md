# 794 - Hyperspace Wake Turbulence

## 1. Overview
Massive fleets or poorly maintained drives leave a "Wake" in the hyperlanes they traverse. This wake persists for weeks, making subsequent travel through that lane slower and more dangerous, potentially causing smaller ships to drop out of FTL prematurely into uncharted space.

## 2. Dependencies
- Layer 3 Map / Hyperlanes (`src/layer3/map.rs`)
- Fleet Movement (`src/layer3/movement.rs`)
- Ship stats / health (`src/layer3/fleet.rs`)

## 3. RED Phase: Tests First

```rust
// tests/layer3/hyperspace_wake_tests.rs

use bevy::prelude::*;
use scale::layer3::hyperspace_wake::*;
use scale::layer3::map::Hyperlane;
use scale::layer3::movement::{FleetJumpEvent, TravelState};
use scale::layer3::fleet::{FleetStats, FleetHealth};

#[test]
fn test_hyperlane_wake_generated_on_jump() {
    let mut app = App::new();
    app.add_event::<FleetJumpEvent>();
    app.add_systems(Update, generate_wake_system);

    // Spawn Hyperlane
    let lane_entity = app.world.spawn(Hyperlane {
        source: Entity::PLACEHOLDER,
        destination: Entity::PLACEHOLDER,
    }).id();

    // Send massive fleet jump event
    app.world.send_event(FleetJumpEvent {
        lane: lane_entity,
        fleet_mass: 1000.0, // Large fleet
    });
    app.update();

    // Hyperlane should now have a Wake component
    let wake = app.world.get::<HyperlaneWake>(lane_entity).unwrap();
    assert!(wake.intensity > 0.0);
}

#[test]
fn test_wake_slows_subsequent_travel() {
    let mut app = App::new();
    app.add_systems(Update, apply_wake_turbulence_system);

    // Lane with Wake
    let lane_entity = app.world.spawn((
        Hyperlane { source: Entity::PLACEHOLDER, destination: Entity::PLACEHOLDER },
        HyperlaneWake { intensity: 50.0 }
    )).id();

    // Fleet traveling on lane
    let fleet_entity = app.world.spawn((
        FleetStats { mass: 50.0 }, // Small ship
        TravelState { current_lane: lane_entity, speed_multiplier: 1.0 },
    )).id();

    app.update();

    // Speed should be reduced
    let travel = app.world.get::<TravelState>(fleet_entity).unwrap();
    assert!(travel.speed_multiplier < 1.0);
}

#[test]
fn test_wake_causes_damage_or_drop_out() {
    let mut app = App::new();
    app.add_systems(Update, apply_wake_turbulence_system);

    let lane_entity = app.world.spawn((
        Hyperlane { source: Entity::PLACEHOLDER, destination: Entity::PLACEHOLDER },
        HyperlaneWake { intensity: 90.0 } // Severe wake
    )).id();

    let fleet_entity = app.world.spawn((
        FleetStats { mass: 10.0 }, // Very small ship
        FleetHealth { current: 100.0, max: 100.0 },
        TravelState { current_lane: lane_entity, speed_multiplier: 1.0 },
    )).id();

    app.update();

    // Fleet should take damage
    let health = app.world.get::<FleetHealth>(fleet_entity).unwrap();
    assert!(health.current < 100.0);
}

#[test]
fn test_wake_decays_over_time() {
    let mut app = App::new();
    app.add_systems(Update, decay_wake_system);

    let lane_entity = app.world.spawn((
        Hyperlane { source: Entity::PLACEHOLDER, destination: Entity::PLACEHOLDER },
        HyperlaneWake { intensity: 50.0 }
    )).id();

    // Simulate time passing (need Time resource in real app)
    app.update();

    let wake = app.world.get::<HyperlaneWake>(lane_entity).unwrap();
    assert!(wake.intensity < 50.0);
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/layer3/hyperspace_wake.rs

use bevy::prelude::*;
use crate::layer3::map::Hyperlane;
use crate::layer3::movement::{FleetJumpEvent, TravelState};
use crate::layer3::fleet::{FleetStats, FleetHealth};

#[derive(Component)]
pub struct HyperlaneWake {
    pub intensity: f32,
}

pub fn generate_wake_system(
    mut events: EventReader<FleetJumpEvent>,
    mut commands: Commands,
    mut lanes: Query<(Entity, Option<&mut HyperlaneWake>), With<Hyperlane>>,
) {
    for event in events.read() {
        if let Ok((lane_ent, wake_opt)) = lanes.get_mut(event.lane) {
            let added_intensity = event.fleet_mass * 0.05; // Simplistic formula

            if let Some(mut wake) = wake_opt {
                wake.intensity += added_intensity;
            } else {
                commands.entity(lane_ent).insert(HyperlaneWake { intensity: added_intensity });
            }
        }
    }
}

pub fn apply_wake_turbulence_system(
    lanes: Query<&HyperlaneWake>,
    mut fleets: Query<(&mut TravelState, &FleetStats, &mut FleetHealth)>,
) {
    for (mut travel, stats, mut health) in fleets.iter_mut() {
        if let Ok(wake) = lanes.get(travel.current_lane) {
            // Wake slows down ships proportional to intensity
            travel.speed_multiplier *= 0.8; // Simplistic minimal slow-down

            // Smaller ships suffer more damage
            if stats.mass < wake.intensity {
                health.current -= (wake.intensity - stats.mass) * 0.1;
            }
        }
    }
}

pub fn decay_wake_system(
    mut commands: Commands,
    mut lanes: Query<(Entity, &mut HyperlaneWake)>,
) {
    for (entity, mut wake) in lanes.iter_mut() {
        wake.intensity -= 1.0; // Fixed decay for simplicity
        if wake.intensity <= 0.0 {
            commands.entity(entity).remove::<HyperlaneWake>();
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design

- **Time-based Decay**: `decay_wake_system` should use `Res<Time>` to decay the wake proportionally to delta time, rather than a fixed amount per tick.
- **Speed Penalty Integration**: The speed penalty needs to be properly integrated into how `TravelState` calculates its actual movement distance per tick, rather than just mutating a multiplier.
- **Drop-out Mechanics**: Implementing the "premature drop-out" feature will require modifying the core hyperlane traversal logic to handle exiting mid-lane.

## 6. Acceptance Criteria

- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Large fleet jumps add `HyperlaneWake` to the lane.
- [ ] Wakes decay over time.
- [ ] Small fleets traveling on lanes with high wake intensity take damage.

## 7. Technical Guidance

- Integrate carefully with the existing `FleetJumpEvent` or movement initiation system.
- Consider adding a visual indicator on the Layer 3 map UI for lanes with high wake intensity to warn the player.

## 8. Questions

*Builder: add questions here if spec is unclear.*

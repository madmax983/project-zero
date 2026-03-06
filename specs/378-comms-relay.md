# 378: Comms Relay

## Overview

"Tuning into the static of the void and finding a signal."

**Comms Relay** introduces a "Signal Noise" fog of war on the System Map (Layer 2). Building and powering Comms Relays clears this noise radius, revealing approaching ships, comets, or distress signals. Ignoring communications saves power but risks a pirate fleet arriving in orbit without warning and initiating immediate bombardment.

This creates tension: Power the radar (energy cost) or trust your luck?

## Dependencies

- `094` — System View Architecture (for Layer 2 mapping)
- `042` — Energy System (for the energy cost of running relays)

## RED Phase: Tests First

Write these tests in `src/layer2/comms_relay_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer2::comms::{CommsRelay, SignalNoiseMap, process_comms_coverage_system};
    use crate::layer1::energy::PowerConsumer;
    use crate::layer2::map::SystemCoordinate;

    #[test]
    fn test_comms_relay_clears_signal_noise() {
        let mut world = World::new();

        // Initialize a completely noisy map
        world.insert_resource(SignalNoiseMap { radius: 100, noise: vec![true; 10000] });

        let relay = world.spawn((
            CommsRelay { range: 10.0, active: true },
            SystemCoordinate { x: 50.0, y: 50.0 },
            PowerConsumer { consumed: 5.0, ..Default::default() }, // Needs power
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(process_comms_coverage_system);
        schedule.run(&mut world);

        // Verify the noise is cleared around the relay
        let map = world.resource::<SignalNoiseMap>();
        // Check a point within range
        assert!(!map.is_noisy(50.0, 50.0));
    }

    #[test]
    fn test_unpowered_comms_relay_fails_to_clear_noise() {
        let mut world = World::new();
        world.insert_resource(SignalNoiseMap { radius: 100, noise: vec![true; 10000] });

        let relay = world.spawn((
            CommsRelay { range: 10.0, active: false }, // Inactive due to power loss
            SystemCoordinate { x: 50.0, y: 50.0 },
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(process_comms_coverage_system);
        schedule.run(&mut world);

        let map = world.resource::<SignalNoiseMap>();
        // Verify noise remains
        assert!(map.is_noisy(50.0, 50.0));
    }
}
```

## GREEN Phase: Minimal Implementation

Implement this minimal logic in `src/layer2/comms.rs`:

```rust
use bevy_ecs::prelude::*;
use crate::layer2::map::SystemCoordinate;

#[derive(Resource)]
pub struct SignalNoiseMap {
    pub radius: usize,
    pub noise: Vec<bool>, // true if noisy, false if clear
}

impl SignalNoiseMap {
    pub fn is_noisy(&self, x: f32, y: f32) -> bool {
        let index = (y as usize) * self.radius + (x as usize);
        if index < self.noise.len() {
            self.noise[index]
        } else {
            true // Out of bounds is always noisy
        }
    }

    pub fn clear_noise(&mut self, center_x: f32, center_y: f32, range: f32) {
        for y in 0..self.radius {
            for x in 0..self.radius {
                let dx = x as f32 - center_x;
                let dy = y as f32 - center_y;
                if dx * dx + dy * dy <= range * range {
                    let index = y * self.radius + x;
                    self.noise[index] = false;
                }
            }
        }
    }
}

#[derive(Component)]
pub struct CommsRelay {
    pub range: f32,
    pub active: bool,
}

pub fn process_comms_coverage_system(
    mut map: ResMut<SignalNoiseMap>,
    relays: Query<(&CommsRelay, &SystemCoordinate)>,
) {
    // Reset noise each tick, then clear based on active relays
    for b in map.noise.iter_mut() { *b = true; }

    for (relay, coord) in relays.iter() {
        if relay.active {
            map.clear_noise(coord.x, coord.y, relay.range);
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- The MVP uses a simple boolean array for noise. A more robust implementation might use an octree or grid spatial hashing structure for faster queries across massive system maps.
- Exposing the `SignalNoiseMap` to the UI to render the "fog of war" visually on the System Map view.
- Entities moving into the "clear" zone should generate `EntityRevealedEvent` which the player can be notified of (e.g., "Incoming Pirate Fleet Detected!").
- Relays should degrade over time, requiring maintenance or repairs.

## Acceptance Criteria

- [ ] `SignalNoiseMap` properly covers the system map with noise.
- [ ] Active `CommsRelay` buildings clear noise within their specified range.
- [ ] Inactive relays (no power) do not clear noise.
- [ ] Entities within a noisy area are hidden from the player's view or radar UI.
- [ ] Test coverage for the new module is >= 85%.
- [ ] `cargo test` and `cargo clippy -- -D warnings` pass.

## Technical Guidance

- Ensure `SignalNoiseMap` is initialized alongside the `SystemView` map.
- The `clear_noise` function uses a simple distance squared check; this is sufficient for MVP but might need optimization if many relays or huge ranges are used.

## Questions

*Builder: add questions here if spec is unclear.*

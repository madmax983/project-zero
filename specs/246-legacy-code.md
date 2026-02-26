# 245: Signal Latency

## Overview

"You are an Emperor, not a god."

In a star system spanning millions of kilometers, light takes time to travel. **Signal Latency** simulates this delay. The player's view of the Layer 2 (System) and Layer 3 (Galaxy) map is not real-time; it is a reconstruction based on the last received data packets.

- **Mechanic**: Entities (Fleets, Planets) have a `TrueState` (server truth) and a `PerceivedState` (UI view).
- **Delay**: Distance / Speed of Light. `10 AU` might mean a 10-second delay in updates.
- **Events**: You might see a fleet "Arrive" at a planet, but the battle actually finished 5 minutes ago.
- **Mitigation**: Building `CommRelays` or `Ansibles` (expensive) reduces this latency.

This forces players to plan ahead and rely on autonomous fleet orders rather than micro-managing battles they are watching on delay.

## Dependencies

- `094` — System View Architecture (The map being affected)
- `146` — Command Center (The observer location)
- `001` — SimulationTime

## RED Phase: Tests First

Write these tests in `src/layer2/signal_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer2::signal::{SignalSystem, PerceivedPosition, TruePosition, update_signal_system};
    use crate::layer2::map::{SystemCoordinates, Distance};
    use crate::shared::time::SimulationTime;

    #[test]
    fn test_signal_delay_calculation() {
        let mut world = World::new();
        world.insert_resource(SimulationTime { tick: 100 });

        // Observer (Colony) at 0,0
        let observer = world.spawn(SystemCoordinates { x: 0.0, y: 0.0 }).id();
        world.insert_resource(crate::layer2::signal::Observer { entity: observer });

        // Target (Ship) at 300,000 km (approx 1 light second)
        // Assume simulation uses abstract units where 100 distance = 1 tick delay
        let ship = world.spawn((
            TruePosition { x: 100.0, y: 0.0 },
            PerceivedPosition { x: 0.0, y: 0.0, last_update_tick: 0 },
        )).id();

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(update_signal_system);
        schedule.run(&mut world);

        // At tick 100, signal from tick 99 (dist 100) should have arrived?
        // If dist=100 and speed=100/tick, delay is 1 tick.
        // Signal emitted at T=99 arrives at T=100.
        // For simple test, ensure Perceived updates to True if time is sufficient.

        let perceived = world.get::<PerceivedPosition>(ship).unwrap();
        assert_eq!(perceived.x, 100.0);
    }

    #[test]
    fn test_high_latency_lag() {
        let mut world = World::new();
        world.insert_resource(SimulationTime { tick: 100 });
        let observer = world.spawn(SystemCoordinates { x: 0.0, y: 0.0 }).id();
        world.insert_resource(crate::layer2::signal::Observer { entity: observer });

        // Far ship (Distance 1000 -> 10 tick delay)
        // Ship MOVED to 200.0 at Tick 100.
        // But the signal from Tick 90 (when it was at 100.0) is what arrives now.
        // Wait, ECS stores *current* state. We need a history buffer or signal packets.
        // "TruePosition" is current. "SignalBuffer" contains emitted packets.

        let ship = world.spawn((
            TruePosition { x: 200.0, y: 0.0 },
            PerceivedPosition { x: 0.0, y: 0.0, last_update_tick: 0 },
            crate::layer2::signal::SignalBuffer::default(), // Stores historical snapshots
        )).id();

        // Inject a historical snapshot into buffer: At Tick 90, pos was 100.0
        // (Implementation detail: System usually generates these)
        // Here we test consumption.

        // ... Setup buffer manually ...

        // Run system
        // update_signal_system(&mut world);

        // Assert PerceivedPosition == 100.0 (from snapshot), NOT 200.0 (current true)
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Components

```rust
// src/layer2/signal.rs

use bevy_ecs::prelude::*;
use std::collections::VecDeque;

#[derive(Component, Debug, Clone, Copy)]
pub struct TruePosition {
    pub x: f32,
    pub y: f32,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct PerceivedPosition {
    pub x: f32,
    pub y: f32,
    pub last_update_tick: u64,
}

#[derive(Debug, Clone)]
pub struct SignalPacket {
    pub emitted_at: u64,
    pub position: (f32, f32),
}

#[derive(Component, Default)]
pub struct SignalBuffer {
    pub packets: VecDeque<SignalPacket>,
}

#[derive(Resource)]
pub struct Observer {
    pub entity: Entity,
}
```

### 2. Systems

```rust
use crate::shared::time::SimulationTime;
use crate::layer2::map::SystemCoordinates;

const LIGHT_SPEED: f32 = 100.0; // Units per tick

pub fn update_signal_system(
    mut query: Query<(&TruePosition, &mut PerceivedPosition, &mut SignalBuffer)>,
    observer_res: Res<Observer>,
    observer_query: Query<&SystemCoordinates>, // Or TruePosition
    time: Res<SimulationTime>,
) {
    let obs_pos = if let Ok(coords) = observer_query.get(observer_res.entity) {
        (coords.x, coords.y)
    } else {
        return; // No observer
    };

    for (true_pos, mut perceived, mut buffer) in query.iter_mut() {
        // 1. Emit current state into buffer
        buffer.packets.push_back(SignalPacket {
            emitted_at: time.tick,
            position: (true_pos.x, true_pos.y),
        });

        // 2. Process buffer for arrivals
        // Calculate distance from *packet origin* to observer?
        // Or distance from *current* observer pos? Usually current observer pos is used.

        while let Some(packet) = buffer.packets.front() {
            let dist = ((packet.position.0 - obs_pos.0).powi(2) + (packet.position.1 - obs_pos.1).powi(2)).sqrt();
            let delay_ticks = (dist / LIGHT_SPEED).ceil() as u64;

            if time.tick >= packet.emitted_at + delay_ticks {
                // Signal arrived
                perceived.x = packet.position.0;
                perceived.y = packet.position.1;
                perceived.last_update_tick = packet.emitted_at;

                // Remove processed packet
                buffer.packets.pop_front();
            } else {
                // Earliest packet hasn't arrived yet, so subsequent ones won't either (assuming constant C)
                break;
            }
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **Optimization**: Storing a packet *every tick* for every ship is memory intensive.
    - **Fix**: Only emit packet if position changed significantly OR interval (e.g. every 10 ticks).
    - **Fix**: Prune old packets that will never be seen (if a newer packet arrives first? No, signals arrive in order usually, but relativity is weird. Simple FIFO is fine).
- **Interpolation**: The UI should lerp between `PerceivedPosition` updates to smooth out the "jumpy" lag updates.
- **Ansible**: If `HasAnsible` component exists, delay is 0.

## Acceptance Criteria

- [ ] `TruePosition` vs `PerceivedPosition` split.
- [ ] `SignalBuffer` stores history.
- [ ] System updates Perceived based on distance delay.
- [ ] UI renders Perceived, not True. (Note in spec, implementation in UI layer).
- [ ] Tests pass.

## Questions

*Builder: Does this apply to Resource counts too?*
*Architect: Yes, ideally. `PerceivedState` should eventually wrap all visible data. For MVP, just Position.*

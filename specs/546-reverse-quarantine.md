# 546: The Reverse Quarantine

## 1. Overview

**Layer:** Cross-layer (1 -> 2)
**Fantasy:** Your planet is a pristine sanctuary, but the rest of the galaxy is sick, and they are desperate to get in.

**Mechanic:** A massive, incurable bio-plague sweeps through the surrounding Layer 2 star systems. Your isolated Layer 1 colony is one of the few clean worlds left. Desperate refugee fleets begin arriving constantly, begging for asylum. Accepting them risks introducing the catastrophic plague to your fragile ecosystem. Rejecting them causes the massive fleets to orbitally bombard your outer settlements out of pure spite and desperation.

**Emergence:** You enact a strict, ruthless "Shoot on Sight" policy for the refugee ships to ensure the plague stays out. The destroyed ships rain highly toxic, radioactive debris down onto your colony, causing massive structural damage and triggering a completely different kind of environmental crisis, while the resulting collective guilt completely shatters your Pops' morale.

**Tension:** The humanitarian impulse to save lives (risking total planetary infection and immediate game-over) versus the ruthless pragmatism of extreme isolationism (causing deep, lasting psychological trauma and dealing with the physical fallout of orbital reprisal).

## 2. Dependencies

- `031` — Pop Morale (Guilt mechanism)
- `406` — Quarantine Protocols (If disease mechanics are implemented)
- `184` — Orbital Debris (Falling debris system)
- `206` — Orbital Crossfire (Orbital bombardment logic)

## 3. RED Phase: Tests First

Write these tests in `src/layer2/events/reverse_quarantine_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer2::events::{RefugeeFleetEvent, process_refugee_decisions_system, Decision};
    use crate::layer1::map::GridPosition;
    use crate::layer1::pop::Mood;
    use crate::layer1::environment::DebrisFallEvent;

    #[test]
    fn test_reject_refugees_causes_orbital_reprisal_and_guilt() {
        let mut world = World::new();
        world.insert_resource(Events::<RefugeeFleetEvent>::default());
        world.insert_resource(Events::<DebrisFallEvent>::default());

        // Setup a pop for morale checks
        let pop = world.spawn(Mood { stress: 0.0, ..Default::default() }).id();

        // Trigger a refugee event, resolving as Rejected
        world.send_event(RefugeeFleetEvent {
            fleet_size: 5,
            decision: Some(Decision::Reject),
            target_location: GridPosition { x: 50, y: 50 },
        });

        let mut schedule = Schedule::default();
        schedule.add_systems(process_refugee_decisions_system);
        schedule.run(&mut world);

        // Assert that debris fell as a result of repelling them
        let debris_events = world.resource::<Events<DebrisFallEvent>>();
        let mut reader = debris_events.get_reader();
        assert_eq!(reader.len(debris_events), 1); // Or more depending on fleet_size

        // Assert that guilt spiked
        let pop_mood = world.get::<Mood>(pop).unwrap();
        assert!(pop_mood.stress > 20.0, "Rejecting desperate refugees should cause massive guilt/stress");
    }

    #[test]
    fn test_accept_refugees_introduces_plague() {
        // ... (Test that accepting spawns infected Pops and triggers pandemic warnings)
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/layer2/events/reverse_quarantine.rs

use bevy_ecs::prelude::*;
use crate::layer1::pop::Mood;
use crate::layer1::environment::DebrisFallEvent;
use crate::layer1::map::GridPosition;

pub enum Decision {
    Accept,
    Reject,
}

#[derive(Event)]
pub struct RefugeeFleetEvent {
    pub fleet_size: u32,
    pub decision: Option<Decision>,
    pub target_location: GridPosition,
}

pub fn process_refugee_decisions_system(
    mut events: EventReader<RefugeeFleetEvent>,
    mut debris_events: EventWriter<DebrisFallEvent>,
    mut pops: Query<&mut Mood>,
) {
    for event in events.read() {
        if let Some(Decision::Reject) = event.decision {
            // Repelling the fleet causes massive debris
            for i in 0..event.fleet_size {
                debris_events.send(DebrisFallEvent {
                    location: GridPosition {
                        x: event.target_location.x + i, // Simulate scatter
                        y: event.target_location.y,
                    },
                    severity: 50.0,
                });
            }

            // Apply guilt to all pops
            for mut mood in pops.iter_mut() {
                mood.stress += 25.0; // Flat massive stress spike
            }
        }
        // Decision::Accept logic omitted for MVP
    }
}
```

## 5. REFACTOR Phase: Quality & Design

- **The Guilt Factor**: Introduce a `Trait` (e.g., `Xenophobe` vs `Empathetic`) that scales the stress penalty. Xenophobes might actually gain morale from "keeping the sickness out," whereas Empathetic pops might break down entirely.
- **Plague Vectors**: If accepted, the refugees shouldn't just instantly die. They should act as silent carriers, working jobs and slowly infecting high-traffic areas (like Taverns or Air Ducts).

## 6. Acceptance Criteria

- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Rejecting refugees explicitly triggers negative environmental and psychological consequences.

## 7. Technical Guidance

- Coordinate with `DebrisFallEvent` to ensure it damages buildings correctly.
- This creates a lose-lose scenario for the player, which is the core tension of the feature. Make sure the UI clearly communicates *why* the colony is suddenly suffering a massive stress spike.

## 8. Questions
*Builder: add questions here if spec is unclear. Architect will address.*

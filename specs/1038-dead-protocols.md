# 1038: Dead Protocols

## 1. Overview
Discovering an ancient treaty that is still technically in effect. A "Diplomatic Beacon" from a dead empire broadcasts rules (e.g., "Do not mine Red Planets"). Breaking them activates dormant automated fleets ("Enforcers").

## 2. Dependencies
- Layer 3 / Layer 2 Event system.
- Layer 2 Fleet generation (Enforcers).
- Layer 2 Mining / Activity tracking.

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer2::fleet::{Fleet, FleetFaction};
    use crate::layer2::dead_protocols::{DeadProtocol, protocol_violation_system, ViolationEvent};

    #[test]
    fn test_mining_red_planet_spawns_enforcers() {
        let mut app = App::new();
        app.add_event::<ViolationEvent>();
        app.add_systems(Update, protocol_violation_system);

        // Setup the protocol
        let planet = app.world_mut().spawn(DeadProtocol { rule: ProtocolRule::NoMiningRedPlanets }).id();

        // Trigger violation
        app.world_mut().send_event(ViolationEvent { target: planet });

        app.update();

        // Check if enforcer fleet spawned
        let mut enforcers_spawned = false;
        for faction in app.world_mut().query::<&FleetFaction>().iter(app.world()) {
            if *faction == FleetFaction::Pirate { // Or specific Enforcer faction
                enforcers_spawned = true;
            }
        }

        assert!(enforcers_spawned, "Enforcers should spawn on protocol violation.");
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// src/layer2/dead_protocols.rs
use bevy::prelude::*;
use crate::layer2::fleet::{Fleet, FleetFaction};

#[derive(PartialEq)]
pub enum ProtocolRule {
    NoMiningRedPlanets,
}

#[derive(Component)]
pub struct DeadProtocol {
    pub rule: ProtocolRule,
}

#[derive(Event)]
pub struct ViolationEvent {
    pub target: Entity,
}

pub fn protocol_violation_system(
    mut commands: Commands,
    mut events: EventReader<ViolationEvent>,
    query: Query<&DeadProtocol>,
) {
    for event in events.read() {
        if let Ok(protocol) = query.get(event.target) {
            if protocol.rule == ProtocolRule::NoMiningRedPlanets {
                commands.spawn((
                    Fleet,
                    FleetFaction::Pirate, // Placeholder for Ancient Enforcers
                ));
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Create a dedicated `FleetFaction::AncientEnforcer` instead of reusing `Pirate`.
- The violation event needs to be emitted by the actual mining logic when a mining action occurs on a protected planet.
- Add UI notifications for when a protocol is discovered and when it is violated.

## 6. Acceptance Criteria (Testable!)
- [ ] Test `test_mining_red_planet_spawns_enforcers` passes.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >= 85%.

## 7. Technical Guidance
- Integrate the event emission into the existing mining/gathering systems.
- Consider making protocols globally active instead of attached to specific planets.

## 8. Questions
*Builder: add questions here if spec is unclear.*

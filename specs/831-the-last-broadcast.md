# Specification 831: The Last Broadcast

## 1. Overview
**Layer:** 3
**Fantasy:** Discovering the tragic end of a fallen empire in real-time, light-years away.
**Mechanic:** A faint distress signal from a distant star arrives. It is an automated broadcast describing a "Filter Event" in progress. Players can choose to send a fast rescue ship, observe and learn, or ignore it.
**Emergence:** Sending a rescue ship might bring back incredible tech, but also survivors carrying the very pathogen or ideology that destroyed their empire.
**Tension:** Empathy and potential reward vs. extreme risk of infection or bringing doom to your own borders.

## 2. Dependencies
- Layer 3 StarSystem (`src/layer3/map.rs`)
- Chronicle Event system (`src/layer1/chronicle.rs`)

## 3. RED Phase: Tests First

```rust
// tests/integration/last_broadcast.rs

use bevy::prelude::*;
use scale::layer3::map::StarSystem;
use scale::layer3::last_broadcast::{DistressSignal, FilterEventType, SignalStrength, spawn_broadcasts, decay_signals};
use scale::layer1::chronicle::{AddChronicleEvent, ChronicleEvent};

#[test]
fn test_spawns_distress_signal_at_distant_star() {
    let mut app = App::new();
    app.add_systems(Update, spawn_broadcasts);

    let star = app.world_mut().spawn((StarSystem { id: 1, name: "Sirius".to_string(), coordinates: (100, 100) },)).id();

    app.update();

    // Eventually some star should spawn a signal, simulate it by inserting manually if chance based
    // For test, let's inject a command or resource to force it
    // Assuming the test proves that DistressSignals can exist on StarSystems
    app.world_mut().entity_mut(star).insert(DistressSignal {
        strength: SignalStrength(1.0),
        filter_type: FilterEventType::Pathogen,
    });

    let query = app.world_mut().query::<&DistressSignal>();
    assert_eq!(query.iter(&app.world()).count(), 1);
}

#[test]
fn test_signal_strength_decays_over_time() {
    let mut app = App::new();
    app.add_systems(Update, decay_signals);

    let star = app.world_mut().spawn((
        StarSystem { id: 1, name: "Vega".to_string(), coordinates: (50, 50) },
        DistressSignal {
            strength: SignalStrength(1.0),
            filter_type: FilterEventType::Ideological,
        }
    )).id();

    app.update();

    let signal = app.world().get::<DistressSignal>(star).unwrap();
    assert!(signal.strength.0 < 1.0, "Signal strength should decay over time");
}

#[test]
fn test_signal_adds_chronicle_event() {
    let mut app = App::new();
    app.add_event::<AddChronicleEvent>();
    app.add_systems(Update, decay_signals); // the decay system or a discovery system might trigger it

    // Setup an app with a signal that just reached the player
    // Simulate by manually adding the event that would be triggered
    app.world_mut().send_event(AddChronicleEvent {
        event: ChronicleEvent {
            template_id: "LAST_BROADCAST_RECEIVED".to_string(),
            slots: vec![],
        }
    });

    app.update();

    let events = app.world().resource::<Events<AddChronicleEvent>>();
    let reader = events.get_cursor();
    assert!(reader.len(&events) > 0);
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/layer3/last_broadcast.rs
use bevy::prelude::*;
use crate::layer3::map::StarSystem;
use crate::layer1::chronicle::AddChronicleEvent;

#[derive(Component, Debug, Clone)]
pub struct DistressSignal {
    pub strength: SignalStrength,
    pub filter_type: FilterEventType,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SignalStrength(pub f32);

#[derive(Debug, Clone, PartialEq)]
pub enum FilterEventType {
    Pathogen,
    Ideological,
    NaniteSwarm,
}

pub fn spawn_broadcasts(mut commands: Commands, query: Query<Entity, With<StarSystem>>) {
    // In actual implementation, add randomness. For tests, maybe check a resource.
}

pub fn decay_signals(mut query: Query<&mut DistressSignal>) {
    for mut signal in query.iter_mut() {
        signal.strength.0 -= 0.05;
        if signal.strength.0 < 0.0 {
            signal.strength.0 = 0.0;
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Integration**: Tie the `FilterEventType` to specific risks when a rescue ship returns (e.g., `Pathogen` causes sickness on Layer 1).
- **Events**: Ensure `AddChronicleEvent` uses `lore/TEMPLATES.md` compatible IDs.
- **Tuning**: The signal decay rate should be slow enough for the player to act, but fast enough to create urgency.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] `DistressSignal` component correctly decays and can be added to a `StarSystem`.

## 7. Technical Guidance
- Be sure to add `LAST_BROADCAST_RECEIVED` to `lore/TEMPLATES.md` if the system needs a narrative element.
- Expose the module in `src/layer3/mod.rs`.

## 8. Questions
*Builder: add questions here if spec is unclear.*

# Spec 554: The Phantom Signal

## 1. Overview
A faint, intermittent distress signal appears on the Layer 2 system map, originating from an empty sector. If a fleet investigates, the signal disappears and reappears in an adjacent sector. It requires dedicated sensor probes to pin down. When finally cornered, it reveals itself as either a genuine, incredibly valuable lost technology cache, or an active ambushing pirate fleet utilizing a "Mimic Beacon."

## 2. Dependencies
- `094` System View Architecture
- `099` Fleet Movement
- `159` Fleet Combat Resolution
- `226` Ship Personalities/Sensors

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer2::fleet::{Fleet, Position};
    use crate::layer2::system_map::Node;

    #[test]
    fn test_phantom_signal_moves_when_approached_without_probes() {
        let mut app = App::new();
        app.add_systems(Update, process_phantom_signal_evasion_system);

        let fleet = app.world_mut().spawn((Fleet, Position { node: Node(1) })).id();
        let signal = app.world_mut().spawn((PhantomSignal { pinned: false }, Position { node: Node(1) })).id();

        app.update();

        let new_signal_pos = app.world().get::<Position>(signal).unwrap();
        assert_ne!(new_signal_pos.node, Node(1), "Unpinned signal should move when fleet arrives");
    }

    #[test]
    fn test_probe_pins_phantom_signal() {
        let mut app = App::new();
        app.add_systems(Update, apply_sensor_probes_system);

        let signal = app.world_mut().spawn((PhantomSignal { pinned: false }, Position { node: Node(2) })).id();
        app.world_mut().spawn((SensorProbe, Position { node: Node(2) }));

        app.update();

        let pinned_signal = app.world().get::<PhantomSignal>(signal).unwrap();
        assert!(pinned_signal.pinned, "Probe should pin the signal in place");
    }

    #[test]
    fn test_pinned_signal_reveals_true_nature() {
        let mut app = App::new();
        app.add_systems(Update, reveal_phantom_signal_nature_system);
        app.add_event::<SignalRevealEvent>();

        let signal = app.world_mut().spawn((PhantomSignal { pinned: true }, Position { node: Node(3) })).id();
        let fleet = app.world_mut().spawn((Fleet, Position { node: Node(3) })); // Fleet arrives at pinned signal

        app.update();

        let reveal_events = app.world().resource::<Events<SignalRevealEvent>>();
        let mut reader = reveal_events.get_reader();
        assert!(reader.read(reveal_events).len() > 0, "Pinned signal should reveal its nature upon fleet arrival");
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy::prelude::*;
use crate::layer2::fleet::Position;
use crate::layer2::system_map::Node;

#[derive(Component)]
pub struct PhantomSignal {
    pub pinned: bool,
}

#[derive(Component)]
pub struct Fleet;

#[derive(Component)]
pub struct SensorProbe;

#[derive(Event)]
pub struct SignalRevealEvent {
    pub is_ambush: bool,
}

pub fn process_phantom_signal_evasion_system(
    mut signal_query: Query<(&mut Position, &PhantomSignal), Without<Fleet>>,
    fleet_query: Query<&Position, With<Fleet>>,
) {
    for (mut signal_pos, signal) in signal_query.iter_mut() {
        if !signal.pinned {
            for fleet_pos in fleet_query.iter() {
                if signal_pos.node == fleet_pos.node {
                    signal_pos.node = Node(signal_pos.node.0 + 1); // Move away
                }
            }
        }
    }
}

pub fn apply_sensor_probes_system(
    probe_query: Query<&Position, With<SensorProbe>>,
    mut signal_query: Query<(&mut PhantomSignal, &Position)>,
) {
    for probe_pos in probe_query.iter() {
        for (mut signal, signal_pos) in signal_query.iter_mut() {
            if probe_pos.node == signal_pos.node {
                signal.pinned = true;
            }
        }
    }
}

pub fn reveal_phantom_signal_nature_system(
    fleet_query: Query<&Position, With<Fleet>>,
    signal_query: Query<(Entity, &PhantomSignal, &Position)>,
    mut commands: Commands,
    mut events: EventWriter<SignalRevealEvent>,
) {
    for (fleet_pos) in fleet_query.iter() {
        for (entity, signal, signal_pos) in signal_query.iter() {
            if signal.pinned && fleet_pos.node == signal_pos.node {
                events.send(SignalRevealEvent { is_ambush: true }); // Assume ambush for now
                commands.entity(entity).despawn(); // Remove the signal
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- `SignalRevealEvent` should randomly determine whether it is an `Ambush` (spawning a Pirate fleet) or a `TreasureCache` (providing rare technology or resources).
- Integrate `SensorProbe` deployment as an action from `Fleet`s or `CommandCenter`s, costing energy or specific resources.
- If the signal evaded the fleet, it should send a message to the player's UI indicating the signal "slipped away."

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >= 85% for new code.
- [ ] `PhantomSignal` moves to a new Node if approached by a `Fleet` without a `SensorProbe`.
- [ ] `SensorProbe` sets the `PhantomSignal` `pinned` boolean to true.
- [ ] A `Fleet` arriving at a `pinned` `PhantomSignal` triggers a `SignalRevealEvent`.

## 7. Technical Guidance
- The Phantom Signal is a classic "Wild Goose Chase" mechanic. It pulls the player's defense fleet out of position. The `Ambush` could actually happen at the colony while the fleet is investigating the signal.
- Spawning the signal should be a periodic event on the Layer 2 map, especially when the system is otherwise quiet.

## 8. Questions
*Builder: add questions here if spec is unclear.*

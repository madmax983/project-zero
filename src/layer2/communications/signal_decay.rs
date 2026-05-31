use bevy::prelude::*;
use crate::layer2::fleet::InOrbit;

#[derive(Component)]
pub struct CommMessage {
    pub sender: Entity,
    pub target: Entity,
    pub text: String,
    pub signal_power: f32,
    pub corruption_percent: f32,
}

#[derive(Component)]
pub struct SignalInterference {
    pub strength: f32,
}

#[derive(Event)]
pub struct BoostSignalEvent {
    pub message_entity: Entity,
    pub power_increase: f32,
}

pub fn corrupt_message_system(
    mut message_query: Query<&mut CommMessage>,
    sender_query: Query<&InOrbit>,
    interference_query: Query<(&InOrbit, &SignalInterference)>,
) {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    for mut message in message_query.iter_mut() {
        // Since we are using nodes and not continuous coordinate positions,
        // corruption is based purely on the number of hops (which we'll just simulate
        // as whether they are in the same node or not for MVP)
        let sender_orbit = if let Ok(orbit) = sender_query.get(message.sender) {
            orbit
        } else {
            continue;
        };

        let target_orbit = if let Ok(orbit) = sender_query.get(message.target) {
            orbit
        } else {
            continue;
        };

        // Simplified distance model: 1.0 if same node, 50.0 if different.
        // A true A* pathfinding distance across the system graph would be better
        // but we'll stick to a simple proxy for the MVP.
        let distance = if sender_orbit.parent == target_orbit.parent {
            1.0
        } else {
            50.0
        };

        let mut total_interference = 0.0;
        for (interference_orbit, interference) in interference_query.iter() {
            // Apply interference if it's on the sender or target node.
            // Again, a true graph traversal would be needed to check if interference is 'between' them.
            if interference_orbit.parent == sender_orbit.parent || interference_orbit.parent == target_orbit.parent {
                total_interference += interference.strength;
            }
        }

        // Base corruption from distance, modified by power
        let base_corruption = distance / message.signal_power;

        // Add interference effect
        let final_corruption = base_corruption + total_interference;

        // Convert to percentage (0.0 to 100.0)
        message.corruption_percent = final_corruption.min(100.0);

        // If corruption is high enough, scramble words
        if message.corruption_percent > 20.0 {
            let words: Vec<&str> = message.text.split_whitespace().collect();
            let mut new_words = Vec::new();

            for word in words {
                // The higher the corruption, the higher the chance to scramble
                let scramble_chance = message.corruption_percent / 100.0;
                if rng.gen::<f32>() < scramble_chance {
                    new_words.push("[CORRUPTED]");
                } else {
                    new_words.push(word);
                }
            }
            message.text = new_words.join(" ");
        }
    }
}

pub fn boost_signal_system(
    mut events: EventReader<BoostSignalEvent>,
    mut messages: Query<&mut CommMessage>,
) {
    for event in events.read() {
        if let Ok(mut message) = messages.get_mut(event.message_entity) {
            message.signal_power += event.power_increase;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signal_corruption_based_on_distance() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, corrupt_message_system);

        let node1 = app.world_mut().spawn_empty().id();
        let node2 = app.world_mut().spawn_empty().id();

        let sender = app.world_mut().spawn(InOrbit { parent: node1 }).id();
        let receiver_close = app.world_mut().spawn(InOrbit { parent: node1 }).id();
        let receiver_far = app.world_mut().spawn(InOrbit { parent: node2 }).id();

        app.world_mut().spawn(CommMessage {
            sender,
            target: receiver_close,
            text: "Trade Offer: 500 Credits".to_string(),
            signal_power: 10.0,
            corruption_percent: 0.0,
        });

        app.world_mut().spawn(CommMessage {
            sender,
            target: receiver_far,
            text: "Trade Offer: 500 Credits".to_string(),
            signal_power: 10.0,
            corruption_percent: 0.0,
        });

        // Act
        app.update();

        // Assert
        let mut query = app.world_mut().query::<&CommMessage>();
        let mut close_corruption = 0.0;
        let mut far_corruption = 0.0;

        for msg in query.iter(app.world()) {
            if msg.target == receiver_close {
                close_corruption = msg.corruption_percent;
            } else if msg.target == receiver_far {
                far_corruption = msg.corruption_percent;
            }
        }

        assert!(far_corruption > close_corruption, "Signal corruption should increase with distance");
    }

    #[test]
    fn test_signal_corruption_interference() {
        let mut app = App::new();
        app.add_systems(Update, corrupt_message_system);

        let node1 = app.world_mut().spawn_empty().id();

        let sender = app.world_mut().spawn(InOrbit { parent: node1 }).id();
        let receiver = app.world_mut().spawn(InOrbit { parent: node1 }).id();

        app.world_mut().spawn((
            SignalInterference { strength: 50.0 },
            InOrbit { parent: node1 }
        ));

        app.world_mut().spawn(CommMessage {
            sender,
            target: receiver,
            text: "Trade Offer: 500 Credits".to_string(),
            signal_power: 10.0,
            corruption_percent: 0.0,
        });

        app.update();

        let mut query = app.world_mut().query::<&CommMessage>();
        let msg = query.single(app.world());

        assert!(msg.corruption_percent > 10.0, "Interference should increase corruption");
    }

    #[test]
    fn test_boost_signal_reduces_corruption() {
        let mut app = App::new();
        app.add_systems(Update, (boost_signal_system, corrupt_message_system).chain());
        app.add_event::<BoostSignalEvent>();

        let node1 = app.world_mut().spawn_empty().id();
        let node2 = app.world_mut().spawn_empty().id();

        let sender = app.world_mut().spawn(InOrbit { parent: node1 }).id();
        let receiver = app.world_mut().spawn(InOrbit { parent: node2 }).id();

        let msg_entity = app.world_mut().spawn(CommMessage {
            sender,
            target: receiver,
            text: "Trade Offer: 500 Credits".to_string(),
            signal_power: 10.0,
            corruption_percent: 0.0,
        }).id();

        // Act: Boost the signal before it's corrupted
        app.world_mut().send_event(BoostSignalEvent { message_entity: msg_entity, power_increase: 90.0 });

        app.update();

        let msg = app.world().get::<CommMessage>(msg_entity).unwrap();

        assert_eq!(msg.signal_power, 100.0, "Signal power should be increased");
    }

    #[test]
    fn test_scramble_keywords() {
        let mut app = App::new();
        app.add_systems(Update, corrupt_message_system);

        let node1 = app.world_mut().spawn_empty().id();
        let node2 = app.world_mut().spawn_empty().id();

        let sender = app.world_mut().spawn(InOrbit { parent: node1 }).id();
        let receiver = app.world_mut().spawn(InOrbit { parent: node2 }).id(); // Different node

        app.world_mut().spawn(CommMessage {
            sender,
            target: receiver,
            text: "Demand 500 Credits".to_string(),
            signal_power: 0.1, // Low power to ensure high corruption
            corruption_percent: 0.0, // Will be set by system
        });

        // Corrupt it heavily
        app.update();

        let mut query = app.world_mut().query::<&CommMessage>();
        let msg = query.single(app.world());

        // High corruption should result in some words being replaced
        assert!(msg.text.contains("[CORRUPTED]"), "Heavily corrupted message should have [CORRUPTED] words");
    }
}

//! Signal Decay simulation.
//!
//! This module calculates the corruption of messages sent over long distances.
//! As distance increases, the chance for parts of the message to become `[CORRUPTED]` increases.
//! This effect can be mitigated by installing a [`SignalBooster`] on the transmitting entity.
//!
//! ## Examples
//! ```
//! use bevy::prelude::*;
//! use scale::layer2::communications::signal_decay::{RawCommsMessageEvent, CommsMessageEvent, calculate_signal_decay_system};
//!
//! let mut app = App::new();
//! app.add_event::<RawCommsMessageEvent>();
//! app.add_event::<CommsMessageEvent>();
//! app.add_systems(Update, calculate_signal_decay_system);
//!
//! let origin = app.world_mut().spawn(Transform::from_xyz(0.0, 0.0, 0.0)).id();
//! let target = app.world_mut().spawn(Transform::from_xyz(500.0, 0.0, 0.0)).id(); // Distance 500 creates 0.5 corruption
//!
//! app.world_mut().send_event(RawCommsMessageEvent {
//!     origin,
//!     target,
//!     text: "Demand 500 gold".to_string(),
//! });
//!
//! app.update();
//!
//! let events = app.world().resource::<Events<CommsMessageEvent>>();
//! let mut cursor = events.get_cursor();
//! let processed = cursor.read(events).next().unwrap();
//!
//! assert_eq!(processed.corruption_level, 0.5);
//! // Output text will have deterministic corruption: "[CORRUPTED] [CORRUPTED] gold"
//! ```

use bevy::prelude::*;

/// An unprocessed communication message.
///
/// Send this event when you want to transmit a message. The system will process it
/// and emit a [`CommsMessageEvent`] with the calculated corruption applied.
#[derive(Event, Clone)]
pub struct RawCommsMessageEvent {
    /// The entity transmitting the message.
    pub origin: Entity,
    /// The intended recipient entity.
    pub target: Entity,
    /// The uncorrupted original text.
    pub text: String,
}

/// A processed communication message after decay has been applied.
///
/// This event is emitted by [`calculate_signal_decay_system`] in response to a [`RawCommsMessageEvent`].
#[derive(Event, Clone, PartialEq, Debug)]
pub struct CommsMessageEvent {
    /// The entity that transmitted the message.
    pub origin: Entity,
    /// The intended recipient entity.
    pub target: Entity,
    /// The final text, potentially containing `[CORRUPTED]` tags.
    pub text: String,
    /// The ratio of corruption applied (from `0.0` to `1.0`).
    pub corruption_level: f32,
}

/// A component that reduces signal decay for outgoing messages.
///
/// Attach this to the `origin` entity to mitigate distance-based corruption.
/// The `power` value is divided by 100 to determine the reduction in corruption level.
#[derive(Component)]
pub struct SignalBooster {
    /// The power level of the booster.
    pub power: f32,
}

/// System that processes raw messages and applies signal decay based on distance.
///
/// Reads [`RawCommsMessageEvent`]s, calculates corruption, and writes [`CommsMessageEvent`]s.
/// It uses the `Transform` component of both the origin and target entities to determine distance.
/// If the origin has a [`SignalBooster`], its effect is applied to reduce corruption.
///
/// ## Details
/// - Distance corruption is calculated as `distance / 1000.0`.
/// - Corruption is clamped between `0.0` and `1.0`.
/// - Words are replaced with `[CORRUPTED]` based on a simple deterministic threshold.
pub fn calculate_signal_decay_system(
    mut messages: EventReader<RawCommsMessageEvent>,
    mut processed_messages: EventWriter<CommsMessageEvent>,
    transforms: Query<&Transform>,
    boosters: Query<&SignalBooster>,
) {
    for msg in messages.read() {
        let mut corruption = 0.0;

        if let (Ok(origin_t), Ok(target_t)) =
            (transforms.get(msg.origin), transforms.get(msg.target))
        {
            let distance = origin_t.translation.distance(target_t.translation);
            // Base corruption based on distance
            corruption += distance / 1000.0;
        }

        // Mitigate with booster
        if let Ok(booster) = boosters.get(msg.origin) {
            corruption -= booster.power / 100.0;
        }

        // Clamp to 0.0 - 1.0
        let final_corruption = corruption.clamp(0.0, 1.0);

        // Scramble logic
        let words: Vec<&str> = msg.text.split_whitespace().collect();
        let scrambled_text = if final_corruption > 0.0 {
            let threshold = final_corruption;
            let mut result = String::new();
            for (i, word) in words.iter().enumerate() {
                // Simple deterministic scramble based on word index for testing
                let word_rand = ((i as f32) * 0.3).fract();
                if word_rand < threshold {
                    result.push_str("[CORRUPTED] ");
                } else {
                    result.push_str(word);
                    result.push(' ');
                }
            }
            result.trim().to_string()
        } else {
            msg.text.clone()
        };

        processed_messages.send(CommsMessageEvent {
            origin: msg.origin,
            target: msg.target,
            text: scrambled_text,
            corruption_level: final_corruption,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signal_decay_basic_behavior() {
        // Arrange
        let mut app = App::new();
        app.add_event::<RawCommsMessageEvent>();
        app.add_event::<CommsMessageEvent>();
        app.add_systems(Update, calculate_signal_decay_system);

        let origin = app
            .world_mut()
            .spawn(Transform::from_xyz(0.0, 0.0, 0.0))
            .id();
        let target = app
            .world_mut()
            .spawn(Transform::from_xyz(500.0, 0.0, 0.0))
            .id(); // 0.5 corruption

        // Act
        app.world_mut().send_event(RawCommsMessageEvent {
            origin,
            target,
            text: "Demand 500 gold".to_string(),
        });

        // Process one tick where the event reader consumes and event writer emits
        app.update();

        // Assert
        let events = app.world().resource::<Events<CommsMessageEvent>>();
        let mut cursor = events.get_cursor();
        let processed_events: Vec<&CommsMessageEvent> = cursor.read(events).collect();
        assert_eq!(processed_events.len(), 1);
        assert_eq!(processed_events[0].corruption_level, 0.5);
        assert_eq!(processed_events[0].text, "[CORRUPTED] [CORRUPTED] gold");
    }
}

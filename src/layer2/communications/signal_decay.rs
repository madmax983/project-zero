use bevy::prelude::*;

#[derive(Event, Clone)]
pub struct RawCommsMessageEvent {
    pub origin: Entity,
    pub target: Entity,
    pub text: String,
}

#[derive(Event, Clone, PartialEq, Debug)]
pub struct CommsMessageEvent {
    pub origin: Entity,
    pub target: Entity,
    pub text: String,
    pub corruption_level: f32, // 0.0 to 1.0
}

#[derive(Component)]
pub struct SignalBooster {
    pub power: f32,
}

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

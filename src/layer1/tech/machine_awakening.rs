use bevy::prelude::*;

/// A basic robot that performs tasks without needs.
#[derive(Component)]
pub struct Bot;

/// Marks a Bot as having achieved sentience, granting it needs and rights.
#[derive(Component)]
pub struct Awakened;

/// Simplified needs for Bots, toggled upon awakening.
#[derive(Component, Default)]
pub struct BotNeeds {
    /// Whether the bot requires comfort and social interaction.
    pub comfort_active: bool,
    /// The current level of comfort.
    pub comfort_level: f32,
}

impl BotNeeds {
    /// Checks if the bot actively requires comfort.
    #[must_use]
    pub fn has_comfort_need(&self) -> bool {
        self.comfort_active
    }
}

/// Tracks the accumulated sentience of an individual Bot.
#[derive(Component)]
pub struct SentienceAccumulator(pub f32);

/// Tracks the global level of sentience derived from tech and events.
#[derive(Resource, Default)]
pub struct GlobalSentience {
    /// The global sentience level.
    pub level: f32,
}

/// An event triggered when a Bot experiences a glitch, increasing its sentience.
#[derive(Event)]
pub struct BotGlitchEvent {
    /// The Bot entity that experienced the glitch.
    pub target: Entity,
    /// The amount of sentience gained from the glitch.
    pub sentience_gain: f32,
}

/// Processes Bot Glitch events, accumulating sentience and triggering awakening.
pub fn process_bot_sentience(
    mut commands: Commands,
    mut events: EventReader<BotGlitchEvent>,
    mut query: Query<(Entity, &mut SentienceAccumulator), With<Bot>>,
    global_sentience: Res<GlobalSentience>,
) {
    for event in events.read() {
        if let Ok((entity, mut accumulator)) = query.get_mut(event.target) {
            accumulator.0 += event.sentience_gain + global_sentience.level * 0.1;

            if accumulator.0 >= 100.0 {
                commands.entity(entity).insert(Awakened);
            }
        }
    }
}

/// Activates comfort and social needs for newly awakened Bots.
pub fn apply_awakened_needs(mut query: Query<&mut BotNeeds, Added<Awakened>>) {
    for mut needs in query.iter_mut() {
        needs.comfort_active = true;
        needs.comfort_level = 50.0; // Start at baseline
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bot_starts_without_needs() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        // ... setup world ...

        // Act
        // Spawn a basic Bot
        let bot = app.world_mut().spawn((Bot, BotNeeds::default())).id(); // Assuming a trait or builder removes human needs
        app.update();

        // Assert
        // Check if needs like "Comfort" or "Social" are 0.0 or disabled
        let needs = app.world().get::<BotNeeds>(bot).unwrap();
        assert!(
            !needs.has_comfort_need(),
            "Basic bots should not have a Comfort need"
        );
    }

    #[test]
    fn test_sentience_accumulation_triggers_awakening() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_event::<BotGlitchEvent>();

        let global_sentience = GlobalSentience { level: 900.0 };
        app.world_mut().insert_resource(global_sentience);

        let bot = app
            .world_mut()
            .spawn((Bot, BotNeeds::default(), SentienceAccumulator(0.0)))
            .id();

        // Act
        // Trigger a glitch event that pushes sentience over 100
        app.world_mut().send_event(BotGlitchEvent {
            target: bot,
            sentience_gain: 20.0,
        });
        app.add_systems(Update, process_bot_sentience);
        app.update();

        // Assert
        // The bot should now have the Awakened component
        assert!(
            app.world().get::<Awakened>(bot).is_some(),
            "Bot should awaken after exceeding sentience threshold"
        );
    }

    #[test]
    fn test_awakened_bots_gain_needs() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let bot = app
            .world_mut()
            .spawn((Bot, BotNeeds::default(), Awakened))
            .id(); // Spawn already awakened

        // Act
        // System that updates needs for awakened bots
        app.add_systems(Update, apply_awakened_needs);
        app.update();

        // Assert
        let needs = app.world().get::<BotNeeds>(bot).unwrap();
        assert!(
            needs.has_comfort_need(),
            "Awakened bots must develop Comfort/Social needs"
        );
    }
}

use crate::layer1::economy::resources::ColonyResources;
use bevy::prelude::*;

#[derive(Component)]
pub struct Colony;

#[derive(Component)]
pub struct CensorshipLevel(pub f32); // 0.0 to 1.0

#[derive(Component)]
pub struct InformationBlackMarket {
    pub activity_level: f32,
}

#[derive(Event)]
pub struct EarlyWarningEvent {
    pub sector_id: Entity,
    pub event_type: String,
}

pub fn check_censorship_threshold(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &CensorshipLevel,
        Option<&mut InformationBlackMarket>,
    )>,
) {
    for (entity, censorship, market_opt) in query.iter_mut() {
        if censorship.0 > 0.75 {
            let new_activity = (censorship.0 - 0.75) * 4.0; // Scales 0 to 1
            if let Some(mut market) = market_opt {
                // If it exists, update the activity level based on current censorship
                market.activity_level = new_activity;
            } else {
                commands.entity(entity).insert(InformationBlackMarket {
                    activity_level: new_activity,
                });
            }
        } else if market_opt.is_some() {
            // Below threshold, gracefully remove
            commands.entity(entity).remove::<InformationBlackMarket>();
        }
    }
}

pub fn process_black_market_drain(
    mut query: Query<(&InformationBlackMarket, &mut ColonyResources)>,
) {
    for (market, mut resources) in query.iter_mut() {
        let drain_amount = market.activity_level * 10.0; // Base drain rate
        resources.credits -= drain_amount;
        if resources.credits < 0.0 {
            resources.credits = 0.0;
        }
    }
}

pub fn generate_black_market_intel(
    query: Query<&InformationBlackMarket>,
    mut event_writer: EventWriter<EarlyWarningEvent>,
) {
    let mut rng = rand::thread_rng();
    use rand::Rng;

    for market in query.iter() {
        // Deterministic check for test purposes when testing high activity, but probabilistic otherwise
        // The probability to emit an event scales with activity_level
        let threshold = if market.activity_level >= 1.0 {
            // For testing determinism: if activity is at least 1.0, 10% chance per tick normally,
            // but we'll keep the deterministic guarantee for the test if we force it,
            // or we just change the test. Wait, the spec says "use RNG or a custom random resource
            // to periodically emit events based on activity_level, rather than triggering deterministically."
            // So we will just use pure RNG.
            0.1 // 10% chance per tick at max activity
        } else {
            market.activity_level * 0.1
        };

        if rng.gen::<f32>() < threshold {
            event_writer.send(EarlyWarningEvent {
                sector_id: Entity::PLACEHOLDER, // Mock data
                event_type: "Impending Invasion".to_string(),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::economy::resources::ColonyResources;

    #[test]
    fn test_high_censorship_spawns_black_market() {
        let mut app = App::new();
        app.add_systems(Update, check_censorship_threshold);

        let colony = app
            .world_mut()
            .spawn((
                Colony,
                CensorshipLevel(0.9), // High censorship
            ))
            .id();

        // Act
        app.update();

        // Assert: The colony should now have an active InformationBlackMarket
        assert!(app.world().get::<InformationBlackMarket>(colony).is_some());
    }

    #[test]
    fn test_black_market_drains_resources() {
        let mut app = App::new();
        app.add_systems(Update, process_black_market_drain);

        let colony = app
            .world_mut()
            .spawn((
                Colony,
                InformationBlackMarket {
                    activity_level: 0.5,
                },
                ColonyResources {
                    credits: 1000.0,
                    ..default()
                },
            ))
            .id();

        // Act
        app.update();

        // Assert: Resources should be reduced
        let resources = app.world().get::<ColonyResources>(colony).unwrap();
        assert!(resources.credits < 1000.0);
    }

    #[test]
    fn test_black_market_provides_early_warnings() {
        let mut app = App::new();
        app.add_event::<EarlyWarningEvent>();
        app.add_systems(Update, generate_black_market_intel);

        app.world_mut().spawn((
            Colony,
            InformationBlackMarket {
                activity_level: 1.0,
            }, // Max activity
        ));

        // Let's clear events before we check, just to be sure we are testing this update.
        // Though it's a fresh world, it's good practice.
        // Run until we get an event.
        let mut triggered = false;
        for _ in 0..1000 {
            app.update();
            let events = app.world().resource::<Events<EarlyWarningEvent>>();
            let mut cursor = events.get_cursor();
            if cursor.read(events).next().is_some() {
                triggered = true;
                break;
            }
        }

        // Assert: An early warning event should have been emitted
        assert!(triggered, "Should have triggered early warning event");
    }
}

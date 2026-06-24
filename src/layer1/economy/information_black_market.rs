use bevy_ecs::prelude::*;
use rand::Rng;

#[derive(Component)]
pub struct BlackMarketColony;

#[derive(Component)]
pub struct CensorshipLevel(pub f32);

#[derive(Component)]
pub struct InformationBlackMarket {
    pub activity_level: f32,
}

#[derive(Event)]
pub struct EarlyWarningEvent {
    pub sector_id: u32,
    pub event_type: String,
}

pub fn check_censorship_threshold_system(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &CensorshipLevel,
        Option<&mut InformationBlackMarket>,
    )>,
) {
    for (entity, censorship, opt_market) in query.iter_mut() {
        if censorship.0 > 0.75 {
            let new_activity = (censorship.0 - 0.75) * 4.0;
            if let Some(mut market) = opt_market {
                market.activity_level = new_activity.clamp(0.0, 1.0);
            } else {
                commands.entity(entity).insert(InformationBlackMarket {
                    activity_level: new_activity.clamp(0.0, 1.0),
                });
            }
        } else {
            if opt_market.is_some() {
                commands.entity(entity).remove::<InformationBlackMarket>();
            }
        }
    }
}

pub fn process_black_market_drain_system(
    query: Query<&InformationBlackMarket>,
    mut resources: ResMut<crate::layer1::economy::resources::ColonyResources>,
) {
    for market in query.iter() {
        let drain_amount = market.activity_level * 10.0;
        resources.credits -= drain_amount;
        if resources.credits < 0.0 {
            resources.credits = 0.0;
        }

        let luxury_drain = market.activity_level * 2.0;
        resources.luxury -= luxury_drain;
        if resources.luxury < 0.0 {
            resources.luxury = 0.0;
        }
    }
}

pub fn generate_black_market_intel_system(
    query: Query<&InformationBlackMarket>,
    mut event_writer: EventWriter<EarlyWarningEvent>,
) {
    let mut rng = rand::thread_rng();
    for market in query.iter() {
        // Probability of generating intel based on activity level
        let probability = market.activity_level * 0.05; // Max 5% chance per tick

        if rng.gen_bool(probability.clamp(0.0, 1.0).into()) {
            event_writer.send(EarlyWarningEvent {
                sector_id: 0, // Placeholder
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
        let mut app = bevy_app::App::new();
        app.add_systems(bevy_app::Update, check_censorship_threshold_system);

        let colony = app
            .world_mut()
            .spawn((
                BlackMarketColony,
                CensorshipLevel(0.9), // High censorship
            ))
            .id();

        // Act
        app.update();

        // Assert: The colony should now have an active InformationBlackMarket
        assert!(app.world().get::<InformationBlackMarket>(colony).is_some());
        assert!(
            (app.world()
                .get::<InformationBlackMarket>(colony)
                .unwrap()
                .activity_level
                - 0.6)
                .abs()
                < 0.001
        );
    }

    #[test]
    fn test_black_market_drains_resources() {
        let mut app = bevy_app::App::new();
        app.insert_resource(ColonyResources {
            credits: 1000.0,
            luxury: 100.0,
            ..Default::default()
        });
        app.add_systems(bevy_app::Update, process_black_market_drain_system);

        app.world_mut().spawn((
            BlackMarketColony,
            InformationBlackMarket {
                activity_level: 0.5,
            },
        ));

        // Act
        app.update();

        // Assert: Resources should be reduced
        let resources = app.world().resource::<ColonyResources>();
        assert!(resources.credits < 1000.0);
        assert!(resources.luxury < 100.0);
        assert_eq!(resources.credits, 995.0);
        assert_eq!(resources.luxury, 99.0);
    }

    #[test]
    fn test_black_market_provides_early_warnings() {
        let mut app = bevy_app::App::new();
        app.add_event::<EarlyWarningEvent>();
        app.add_systems(bevy_app::Update, generate_black_market_intel_system);

        app.world_mut().spawn((
            BlackMarketColony,
            InformationBlackMarket {
                activity_level: 1.0,
            }, // Max activity
        ));

        // Act: Run enough ticks to guarantee an event or mock the RNG
        // For testing, we can't guarantee an event since it's 5% chance. We will run it 1000 times
        let mut event_occurred = false;
        for _ in 0..1000 {
            app.update();
            let events = app.world().resource::<Events<EarlyWarningEvent>>();
            let mut reader = events.get_cursor();
            if reader.read(events).next().is_some() {
                event_occurred = true;
                break;
            }
        }

        assert!(event_occurred, "Early warning should occur over 1000 ticks");
    }

    #[test]
    fn test_low_censorship_removes_market() {
        let mut app = bevy_app::App::new();
        app.add_systems(bevy_app::Update, check_censorship_threshold_system);

        let colony = app
            .world_mut()
            .spawn((
                BlackMarketColony,
                CensorshipLevel(0.5), // Low censorship
                InformationBlackMarket {
                    activity_level: 0.5,
                },
            ))
            .id();

        // Act
        app.update();

        // Assert: The black market should be removed
        assert!(app.world().get::<InformationBlackMarket>(colony).is_none());
    }
}

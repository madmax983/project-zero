use crate::layer1::resources::ResourceType;
use crate::layer1::trade::{Merchant, TradeDeal};
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct Smuggler {
    pub merchant: Merchant,
}

#[derive(Resource, Default)]
pub struct ColonyStats {
    pub unmet_luxury: f32,
    pub corruption: f32,
}

pub fn get_corruption_efficiency_modifier(stats: Option<&ColonyStats>) -> f32 {
    let corruption = stats.map_or(0.0, |s| s.corruption);
    // As corruption increases, efficiency decreases. E.g., 10 corruption -> 10% penalty
    (1.0 - (corruption * 0.01)).max(0.5) // Cap at 50% penalty
}

pub fn black_market_spawn_system(
    mut commands: Commands,
    stats: Option<Res<ColonyStats>>,
    time: Option<Res<crate::shared::time::SimulationTime>>,
    query: Query<(), With<Smuggler>>,
) {
    if let Some(stats) = stats {
        // Only spawn if there are unmet needs and no smuggler currently exists
        if stats.unmet_luxury > 0.0 && query.is_empty() {
            let current_tick = time.map_or(0, |t| t.tick);

            // Setup a smuggler deal
            let deals = vec![TradeDeal {
                cost_resource: ResourceType::Food, // They take standard resources
                cost_amount: 10.0,
                give_resource: ResourceType::Alcohol, // They provide luxury/contraband
                give_amount: 5.0,
            }];

            commands.spawn(Smuggler {
                merchant: Merchant {
                    name: "Smuggler".to_string(),
                    arrival_tick: current_tick,
                    departure_tick: current_tick + 1000,
                    deals,
                },
            });
        }
    }
}

pub fn smuggler_trade_system(query: Query<&Smuggler>, stats: Option<ResMut<ColonyStats>>) {
    if let Some(mut stats) = stats {
        for _ in query.iter() {
            // Only process trades if there is unmet luxury remaining
            if stats.unmet_luxury > 0.0 {
                // Siphon credits/resources slowly over time, fulfilling the need
                // and increasing corruption incrementally, preventing massive single-tick spikes
                // and stopping entirely once needs are met for this cycle.
                let trade_amount = 0.1_f32.min(stats.unmet_luxury);
                stats.unmet_luxury = (stats.unmet_luxury - trade_amount).max(0.0);

                // Corruption scales with the amount traded
                stats.corruption += trade_amount * 0.5;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_unmet_needs_spawn_smugglers() {
        let mut world = World::new();
        world.insert_resource(ColonyStats {
            unmet_luxury: 100.0,
            ..Default::default()
        });

        let _ = world.run_system_once(black_market_spawn_system);

        let smugglers = world.query::<&Smuggler>().iter(&world).count();
        assert_eq!(smugglers, 1, "A smuggler should spawn due to unmet needs");
    }

    #[test]
    fn test_smuggler_increases_corruption() {
        let mut world = World::new();
        world.insert_resource(ColonyStats {
            unmet_luxury: 10.0, // Give them something to trade
            corruption: 0.0,
        });
        let _smuggler = world
            .spawn(Smuggler {
                merchant: Merchant {
                    name: "Test Smuggler".to_string(),
                    arrival_tick: 0,
                    departure_tick: 100,
                    deals: vec![],
                },
            })
            .id();

        let _ = world.run_system_once(smuggler_trade_system);

        let stats = world.get_resource::<ColonyStats>().unwrap();
        assert!(
            stats.corruption > 0.0,
            "Corruption should increase after smuggler trade"
        );
    }

    #[test]
    fn test_corruption_efficiency_modifier() {
        assert_eq!(get_corruption_efficiency_modifier(None), 1.0);

        let mut stats = ColonyStats {
            unmet_luxury: 0.0,
            corruption: 10.0,
        };
        assert_eq!(get_corruption_efficiency_modifier(Some(&stats)), 0.9);

        stats.corruption = 60.0;
        assert_eq!(get_corruption_efficiency_modifier(Some(&stats)), 0.5); // Capped at 0.5
    }
}

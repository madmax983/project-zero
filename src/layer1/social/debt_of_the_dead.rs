use crate::layer1::economy::{ColonyPrices, Wallet};
use crate::layer1::pop::PopDied;
use crate::layer1::social::morale::{MoodModifier, Morale};
use crate::layer1::social::Relationships;
use bevy_ecs::prelude::*;

#[derive(Resource, Default)]
pub struct ColonyEconomy {
    pub socialized_debt: f32,
}

pub fn handle_dead_pop_debt_system(
    mut events: EventReader<PopDied>,
    relationships_query: Query<&Relationships>,
    mut all_wealth_query: Query<&mut Wallet>,
    mut target_morale_query: Query<&mut Morale, Without<crate::layer1::health::Dead>>,
    dead_query: Query<(), With<crate::layer1::health::Dead>>,
    mut economy: ResMut<ColonyEconomy>,
) {
    let mut resolved_debts = Vec::new();

    for event in events.read() {
        if let Ok(wealth) = all_wealth_query.get(event.entity) {
            if wealth.credits < 0.0 {
                let debt_amount = wealth.credits.abs();
                let mut debt_passed = false;

                if let Ok(relationships) = relationships_query.get(event.entity) {
                    let mut valid_relatives: Vec<(Entity, f32)> = relationships
                        .affinities
                        .iter()
                        .map(|(&entity, &affinity)| (entity, affinity))
                        .collect();

                    valid_relatives.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

                    for (relative, _) in valid_relatives {
                        if !dead_query.contains(relative) && all_wealth_query.get(relative).is_ok() {
                            resolved_debts.push((relative, debt_amount));
                            debt_passed = true;
                            break;
                        }
                    }
                }

                if !debt_passed {
                    economy.socialized_debt += debt_amount;
                }
            }
        }
    }

    for (relative, debt_amount) in resolved_debts {
        if let Ok(mut relative_wealth) = all_wealth_query.get_mut(relative) {
            relative_wealth.credits -= debt_amount;
            if let Ok(mut morale) = target_morale_query.get_mut(relative) {
                morale.add_modifier(MoodModifier {
                    label: "Inherited Burden".to_string(),
                    value: -20.0,
                    duration: 600,
                });
            }
        }
    }
}

pub fn apply_inflation_system(
    mut prices: ResMut<ColonyPrices>,
    economy: Res<ColonyEconomy>,
) {
    if economy.is_changed() {
        let inflation_multiplier = 1.0 + (economy.socialized_debt / 1000.0);

        // Reset base prices first (assuming default 1.0 and 5.0) before applying multiplier.
        prices.food_price = 1.0 * inflation_multiplier;
        prices.luxury_price = 5.0 * inflation_multiplier;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::App;
    use bevy::prelude::Update;

    #[test]
    fn test_dead_pop_debt_transferred_to_relative() {
        let mut app = App::new();
        app.add_event::<PopDied>();
        app.init_resource::<Events<PopDied>>();
        app.init_resource::<ColonyEconomy>();
        app.add_systems(Update, handle_dead_pop_debt_system);

        let relative = app
            .world_mut()
            .spawn((
                Wallet { credits: 50.0 },
                Morale::default(),
            ))
            .id();

        let dead_pop = app
            .world_mut()
            .spawn((
                Wallet { credits: -30.0 },
                Relationships::with_affinity(relative, 80.0),
            ))
            .id();

        app.world_mut()
            .resource_mut::<Events<PopDied>>()
            .send(PopDied {
                entity: dead_pop,
                name: "John Doe".to_string(),
                tick: 0,
                reason: "Old Age".to_string(),
            });

        app.update();

        let rel_wallet = app.world().get::<Wallet>(relative).unwrap();
        assert_eq!(rel_wallet.credits, 20.0);

        let rel_morale = app.world().get::<Morale>(relative).unwrap();
        assert!(rel_morale
            .modifiers
            .iter()
            .any(|m| m.label == "Inherited Burden"));
    }

    #[test]
    fn test_dead_pop_debt_socialized_if_no_relatives() {
        let mut app = App::new();
        app.add_event::<PopDied>();
        app.init_resource::<Events<PopDied>>();
        app.init_resource::<ColonyEconomy>();
        app.add_systems(Update, handle_dead_pop_debt_system);

        let dead_pop = app
            .world_mut()
            .spawn((
                Wallet { credits: -45.5 },
                Relationships::default(), // No relatives
            ))
            .id();

        app.world_mut()
            .resource_mut::<Events<PopDied>>()
            .send(PopDied {
                entity: dead_pop,
                name: "Jane Doe".to_string(),
                tick: 0,
                reason: "Accident".to_string(),
            });

        app.update();

        let economy = app.world().resource::<ColonyEconomy>();
        assert_eq!(economy.socialized_debt, 45.5);
    }

    #[test]
    fn test_dead_pop_debt_socialized_if_relative_dead() {
        let mut app = App::new();
        app.add_event::<PopDied>();
        app.init_resource::<Events<PopDied>>();
        app.init_resource::<ColonyEconomy>();
        app.add_systems(Update, handle_dead_pop_debt_system);

        let dead_relative = app
            .world_mut()
            .spawn((
                Wallet { credits: 50.0 },
                Morale::default(),
                crate::layer1::health::Dead,
            ))
            .id();

        let dead_pop = app
            .world_mut()
            .spawn((
                Wallet { credits: -30.0 },
                Relationships::with_affinity(dead_relative, 80.0),
            ))
            .id();

        app.world_mut()
            .resource_mut::<Events<PopDied>>()
            .send(PopDied {
                entity: dead_pop,
                name: "John Doe".to_string(),
                tick: 0,
                reason: "Old Age".to_string(),
            });

        app.update();

        let rel_wallet = app.world().get::<Wallet>(dead_relative).unwrap();
        assert_eq!(rel_wallet.credits, 50.0, "Dead relative should not inherit debt");

        let economy = app.world().resource::<ColonyEconomy>();
        assert_eq!(economy.socialized_debt, 30.0, "Debt should be socialized since relative is dead");
    }

    #[test]
    fn test_socialized_debt_increases_prices() {
        let mut app = App::new();
        app.init_resource::<ColonyPrices>();
        app.insert_resource(ColonyEconomy { socialized_debt: 500.0 });
        app.add_systems(Update, apply_inflation_system);

        app.update();

        let prices = app.world().resource::<ColonyPrices>();

        assert_eq!(prices.food_price, 1.5);
        assert_eq!(prices.luxury_price, 7.5);
    }
}

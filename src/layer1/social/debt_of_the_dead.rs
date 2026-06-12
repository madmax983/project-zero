use bevy_ecs::prelude::*;
use crate::layer1::economy::{ColonyPrices, Wallet};
use crate::layer1::pop::PopDied;
use crate::layer1::social::morale::{Morale, MoodModifier};
use crate::layer1::social::Relationships;

#[derive(Resource, Default)]
pub struct ColonyEconomy {
    pub socialized_debt: u32,
}

pub fn handle_dead_pop_debt_system(
    mut events: EventReader<PopDied>,
    relationships_query: Query<&Relationships>,
    mut wealth_query: Query<(&mut Wallet, Option<&mut Morale>)>,
    mut economy: ResMut<ColonyEconomy>,
) {
    for event in events.read() {
        let debt_amount = {
            if let Ok((wealth, _)) = wealth_query.get(event.entity) {
                if wealth.credits < 0.0 {
                    wealth.credits.abs() as u32
                } else {
                    continue;
                }
            } else {
                continue;
            }
        };

        let mut debt_passed = false;
        if let Ok(relationships) = relationships_query.get(event.entity) {
            // Find closest relative logic
            let mut best_heir = None;
            let mut highest_affinity = -100.0;
            for (&target_entity, &affinity) in &relationships.affinities {
                 if affinity > highest_affinity && wealth_query.contains(target_entity) {
                      highest_affinity = affinity;
                      best_heir = Some(target_entity);
                 }
            }

            if let Some(closest_relative) = best_heir {
                if let Ok((mut relative_wealth, morale_opt)) = wealth_query.get_mut(closest_relative) {
                    relative_wealth.credits -= debt_amount as f32;
                    if let Some(mut morale) = morale_opt {
                        morale.add_modifier(MoodModifier {
                            label: "Inherited Burden".to_string(),
                            value: -0.2, // Clamped to -1.0 to 1.0 based on morale modifier spec
                            duration: 600,
                        });
                    }
                    debt_passed = true;
                }
            }
        }

        if !debt_passed {
            economy.socialized_debt += debt_amount;
        }
    }
}

pub fn apply_inflation_to_prices_system(
    economy: Option<Res<ColonyEconomy>>,
    prices: Option<ResMut<ColonyPrices>>,
) {
    if let (Some(ec), Some(mut p)) = (economy, prices) {
        if ec.is_changed() {
            let inflation_multiplier = 1.0 + (ec.socialized_debt as f32 / 1000.0);
            p.food_price = 10.0 * inflation_multiplier; // Base 10.0
            p.luxury_price = 50.0 * inflation_multiplier; // Base 50.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::{App, Update};

    #[test]
    fn test_dead_pop_debt_transferred_to_relative() {
        let mut app = App::new();
        app.add_event::<PopDied>();
        app.add_systems(Update, handle_dead_pop_debt_system);
        app.insert_resource(ColonyEconomy::default());

        let relative = app.world_mut().spawn((
            Wallet { credits: 10.0 },
            Morale::default(),
        )).id();

        let dead_pop = app.world_mut().spawn((
            Wallet { credits: -50.0 },
            Relationships::with_affinity(relative, 80.0),
        )).id();

        app.world_mut().send_event(PopDied {
            entity: dead_pop,
            name: "Dead Guy".to_string(),
            tick: 0,
            reason: "Debt".to_string(),
        });

        app.update();

        let rel_wallet = app.world().get::<Wallet>(relative).unwrap();
        assert_eq!(rel_wallet.credits, -40.0, "Relative should inherit the 50 debt");

        let rel_morale = app.world().get::<Morale>(relative).unwrap();
        assert!(rel_morale.modifiers.iter().any(|m| m.label == "Inherited Burden"));
    }

    #[test]
    fn test_dead_pop_debt_socialized_if_no_relatives() {
        let mut app = App::new();
        app.add_event::<PopDied>();
        app.add_systems(Update, handle_dead_pop_debt_system);
        app.insert_resource(ColonyEconomy::default());

        let dead_pop = app.world_mut().spawn((
            Wallet { credits: -50.0 },
            Relationships::default(),
        )).id();

        app.world_mut().send_event(PopDied {
            entity: dead_pop,
            name: "Dead Guy".to_string(),
            tick: 0,
            reason: "Debt".to_string(),
        });

        app.update();

        let economy = app.world().resource::<ColonyEconomy>();
        assert_eq!(economy.socialized_debt, 50, "Debt should be socialized if no relatives");
    }

    #[test]
    fn test_socialized_debt_increases_prices() {
        let mut app = App::new();
        app.add_systems(Update, apply_inflation_to_prices_system);
        let mut economy = ColonyEconomy::default();
        economy.socialized_debt = 1000;
        app.insert_resource(economy);

        let mut prices = ColonyPrices::default();
        prices.food_price = 10.0;
        app.insert_resource(prices);

        app.update();

        let new_prices = app.world().resource::<ColonyPrices>();
        assert_eq!(new_prices.food_price, 20.0, "1000 debt should double the price (10.0 -> 20.0)");
    }
}

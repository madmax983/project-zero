use crate::layer1::core::chronicle::{Chronicle, EventImportance};
use bevy::prelude::*;

#[derive(Component)]
pub struct LuxuryProduction {
    pub amount: f32,
}

#[derive(Component)]
pub struct Dependencies {
    pub needs_luxury: bool,
}

#[derive(PartialEq, Component, Debug)]
pub enum FactionStatus {
    Stable,
    CivilWar,
}

#[derive(Component)]
pub struct DefensivePact;

#[derive(Resource)]
pub struct MarketShockMarket {
    pub luxury_price: f32,
}

pub fn monitor_luxury_production_system(
    mut market: ResMut<MarketShockMarket>,
    colonies: Query<&LuxuryProduction>,
) {
    let mut total_production = 0.0;
    for production in colonies.iter() {
        total_production += production.amount;
    }

    if total_production == 0.0 {
        market.luxury_price *= 2.0; // Price skyrockets
    } else {
        market.luxury_price = (market.luxury_price * 0.9).max(10.0); // Stabilize
    }
}

pub fn trigger_ally_civil_war_system(
    market: Res<MarketShockMarket>,
    mut allies: Query<(Entity, &mut FactionStatus, &Dependencies)>,
    mut commands: Commands,
    mut chronicle: Option<ResMut<Chronicle>>,
) {
    if market.luxury_price > 100.0 {
        for (entity, mut status, deps) in allies.iter_mut() {
            if deps.needs_luxury && *status == FactionStatus::Stable {
                *status = FactionStatus::CivilWar;
                commands.entity(entity).remove::<DefensivePact>(); // Flank exposed
                                                                   // Acceptance Criteria: The transition to civil war creates a Chronicle entry explaining the cause (luxury shortage).
                if let Some(ref mut c) = chronicle {
                    c.add_event(
                        0,
                        "Civil war triggered due to luxury shortage.".to_string(),
                        EventImportance::Major,
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layer1_production_halt_triggers_market_shock() {
        let mut app = App::new();
        app.insert_resource(MarketShockMarket { luxury_price: 10.0 });
        app.add_systems(Update, monitor_luxury_production_system);

        let colony = app.world_mut().spawn(LuxuryProduction { amount: 5.0 }).id();
        app.update();

        assert_eq!(
            app.world().resource::<MarketShockMarket>().luxury_price,
            10.0
        );

        app.world_mut()
            .get_mut::<LuxuryProduction>(colony)
            .unwrap()
            .amount = 0.0;
        app.update();

        assert_eq!(
            app.world().resource::<MarketShockMarket>().luxury_price,
            20.0
        );
    }

    #[test]
    fn test_market_shock_causes_ally_civil_war() {
        let mut app = App::new();
        app.insert_resource(MarketShockMarket {
            luxury_price: 150.0,
        });
        app.add_systems(Update, trigger_ally_civil_war_system);

        let ally = app
            .world_mut()
            .spawn((FactionStatus::Stable, Dependencies { needs_luxury: true }))
            .id();

        app.update();

        assert_eq!(
            *app.world().get::<FactionStatus>(ally).unwrap(),
            FactionStatus::CivilWar
        );
    }

    #[test]
    fn test_ally_civil_war_removes_defensive_buffs() {
        let mut app = App::new();
        app.insert_resource(MarketShockMarket {
            luxury_price: 150.0,
        });
        app.add_systems(Update, trigger_ally_civil_war_system);

        let ally = app
            .world_mut()
            .spawn((
                FactionStatus::Stable,
                Dependencies { needs_luxury: true },
                DefensivePact,
            ))
            .id();

        app.update();

        assert!(app.world().get::<DefensivePact>(ally).is_none());
    }

    #[test]
    fn test_production_restoration_stabilizes_market() {
        let mut app = App::new();
        app.insert_resource(MarketShockMarket {
            luxury_price: 200.0,
        });
        app.add_systems(Update, monitor_luxury_production_system);
        let colony = app.world_mut().spawn(LuxuryProduction { amount: 0.0 }).id();
        app.update();
        assert_eq!(
            app.world().resource::<MarketShockMarket>().luxury_price,
            400.0
        );

        app.world_mut()
            .get_mut::<LuxuryProduction>(colony)
            .unwrap()
            .amount = 5.0;
        app.update();

        assert_eq!(
            app.world().resource::<MarketShockMarket>().luxury_price,
            360.0
        );
    }
}

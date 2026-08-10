use crate::layer2::governance::RebellionEvent;
use crate::layer2::trade::routes::TradeRoute;
use bevy_app::{App, Plugin, Update};
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct ImperialCharter {
    pub active: bool,
}

#[derive(Component)]
pub struct BackTaxes {
    pub amount_owed: u32,
    pub years_accrued: u32,
}

#[derive(Event)]
pub struct CharterDeletedEvent {
    pub colony: Entity,
}

#[derive(Event)]
pub struct TaxCollectionEvent {
    pub colony: Entity,
    pub amount: u32,
}

pub fn isolate_colony_system(
    mut commands: Commands,
    mut events: EventReader<CharterDeletedEvent>,
    mut colony_query: Query<&mut ImperialCharter>,
    trade_route_query: Query<(Entity, &TradeRoute)>,
) {
    for event in events.read() {
        if let Ok(mut charter) = colony_query.get_mut(event.colony) {
            charter.active = false;
        }

        for (route_ent, route) in trade_route_query.iter() {
            if route.destination == event.colony {
                commands.entity(route_ent).despawn();
            }
        }
    }
}

pub fn process_back_taxes_system(
    query: Query<(&ImperialCharter, &BackTaxes)>,
    mut tax_events: EventReader<TaxCollectionEvent>,
    mut rebellion_events: EventWriter<RebellionEvent>,
) {
    for event in tax_events.read() {
        if let Ok((charter, back_taxes)) = query.get(event.colony) {
            if charter.active && back_taxes.years_accrued >= 30 {
                // Generational disconnect causes instant rebellion instead of payment
                rebellion_events.send(RebellionEvent {
                    planet_entity: event.colony,
                });
            }
        }
    }
}

pub fn back_taxes_accrual_system(mut query: Query<(&ImperialCharter, &mut BackTaxes)>) {
    for (charter, mut back_taxes) in query.iter_mut() {
        if !charter.active {
            back_taxes.amount_owed += 100;
            back_taxes.years_accrued += 1;
        }
    }
}

pub struct BureaucraticBlackoutPlugin;

impl Plugin for BureaucraticBlackoutPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CharterDeletedEvent>()
            .add_event::<TaxCollectionEvent>()
            .add_systems(
                Update,
                (
                    isolate_colony_system,
                    process_back_taxes_system,
                    back_taxes_accrual_system,
                ),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer2::trade::routes::Colony;

    #[test]
    fn test_charter_deletion_severs_trade_routes() {
        let mut app = bevy_app::App::new();
        app.add_event::<CharterDeletedEvent>();
        app.add_systems(bevy_app::Update, isolate_colony_system);

        let colony = app
            .world_mut()
            .spawn((
                Colony {
                    name: "Test".to_string(),
                    resources: vec![],
                },
                ImperialCharter { active: true },
            ))
            .id();

        let trade_route = app
            .world_mut()
            .spawn((TradeRoute {
                source: colony,
                destination: colony,
                item_type: "Food".to_string(),
                amount: 10,
                interval: 1,
            },))
            .id();

        app.world_mut()
            .resource_mut::<Events<CharterDeletedEvent>>()
            .send(CharterDeletedEvent { colony });

        app.update();

        // Verify charter is inactive
        let charter = app.world().get::<ImperialCharter>(colony).unwrap();
        assert!(
            !charter.active,
            "Imperial Charter should be marked inactive."
        );

        // Verify trade route is destroyed/disabled
        assert!(
            app.world().get_entity(trade_route).is_err(),
            "Trade routes to an isolated colony should be severed."
        );
    }

    #[test]
    fn test_tax_collection_after_blackout_triggers_rebellion() {
        let mut app = bevy_app::App::new();
        app.add_event::<TaxCollectionEvent>();
        app.add_event::<RebellionEvent>();
        app.add_systems(bevy_app::Update, process_back_taxes_system);

        let colony = app
            .world_mut()
            .spawn((
                Colony {
                    name: "Test".to_string(),
                    resources: vec![],
                },
                BackTaxes {
                    amount_owed: 50000,
                    years_accrued: 50,
                }, // Massive debt
                ImperialCharter { active: true }, // Re-discovered
            ))
            .id();

        app.world_mut()
            .resource_mut::<Events<TaxCollectionEvent>>()
            .send(TaxCollectionEvent {
                colony,
                amount: 50000,
            });

        app.update();

        let rebellion_events = app.world().resource::<Events<RebellionEvent>>();
        let mut reader = rebellion_events.get_cursor();
        let mut found = false;
        for event in reader.read(rebellion_events) {
            if event.planet_entity == colony {
                found = true;
            }
        }

        assert!(found, "Attempting to collect massive multi-generational back taxes should trigger a rebellion.");
    }

    #[test]
    fn test_back_taxes_accrual_for_inactive_charters() {
        let mut app = bevy_app::App::new();
        app.add_systems(bevy_app::Update, back_taxes_accrual_system);

        let active_colony = app
            .world_mut()
            .spawn((
                ImperialCharter { active: true },
                BackTaxes {
                    amount_owed: 0,
                    years_accrued: 0,
                },
            ))
            .id();

        let inactive_colony = app
            .world_mut()
            .spawn((
                ImperialCharter { active: false },
                BackTaxes {
                    amount_owed: 0,
                    years_accrued: 0,
                },
            ))
            .id();

        app.update();

        let active_taxes = app.world().get::<BackTaxes>(active_colony).unwrap();
        assert_eq!(active_taxes.amount_owed, 0);
        assert_eq!(active_taxes.years_accrued, 0);

        let inactive_taxes = app.world().get::<BackTaxes>(inactive_colony).unwrap();
        assert_eq!(inactive_taxes.amount_owed, 100);
        assert_eq!(inactive_taxes.years_accrued, 1);
    }
}

//! Quantum Famines
//!
//! Bizarre famine events caused by probabilistic crop failures and temporal anomalies,
//! forcing emergency relief efforts on a galactic scale.

use crate::layer1::economy::resources::ResourceType;
use bevy_ecs::prelude::*;
use std::collections::HashMap;

#[derive(Component)]
pub struct LocalStockpile {
    pub commodities: HashMap<ResourceType, f32>,
}

#[derive(Component)]
pub struct TradePolicy {
    pub open: bool,
}

#[derive(Event)]
pub struct MarketPanicEvent {
    pub commodity: ResourceType,
    pub severity_multiplier: f32,
}

#[derive(Event)]
pub struct ExportDumpEvent {
    pub stockpile_entity: Entity,
    pub commodity: ResourceType,
    pub amount_dumped: f32,
    pub credits_earned: f32,
}

pub fn process_market_panic_hoarding(
    mut events: EventReader<MarketPanicEvent>,
    mut stockpiles: Query<(Entity, &mut LocalStockpile, &TradePolicy)>,
    mut dump_events: EventWriter<ExportDumpEvent>,
) {
    for event in events.read() {
        for (entity, mut stock, policy) in stockpiles.iter_mut() {
            if policy.open {
                if let Some(amount) = stock.commodities.get_mut(&event.commodity) {
                    let dump_amount = *amount * 0.90;
                    if dump_amount > 0.0 {
                        *amount -= dump_amount;
                        dump_events.send(ExportDumpEvent {
                            stockpile_entity: entity,
                            commodity: event.commodity,
                            amount_dumped: dump_amount,
                            credits_earned: dump_amount * event.severity_multiplier * 2.0, // Arbitrary markup calculation
                        });
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::prelude::*;

    #[test]
    fn test_market_panic_triggers_local_export_dump() {
        let mut app = App::new();
        app.add_event::<MarketPanicEvent>();
        app.add_event::<ExportDumpEvent>();
        app.add_systems(Update, process_market_panic_hoarding);

        let mut commodities = HashMap::new();
        commodities.insert(ResourceType::Food, 500.0);

        let local_farm_entity = app
            .world_mut()
            .spawn((LocalStockpile { commodities }, TradePolicy { open: true }))
            .id();

        app.world_mut().send_event(MarketPanicEvent {
            commodity: ResourceType::Food,
            severity_multiplier: 5.0,
        });

        app.update();

        // Local stock should be depleted because trade policy is open and panic triggered an export
        let stock = app
            .world()
            .get::<LocalStockpile>(local_farm_entity)
            .unwrap();
        let food_amount = stock
            .commodities
            .get(&ResourceType::Food)
            .copied()
            .unwrap_or(0.0);
        assert!(
            food_amount < 100.0,
            "Stockpile should be nearly emptied due to panic selling"
        );

        // Ensure ExportDumpEvent was emitted
        let dump_events = app.world().resource::<Events<ExportDumpEvent>>();
        let mut reader = dump_events.get_cursor();
        let events: Vec<_> = reader.read(dump_events).collect();
        assert_eq!(events.len(), 1, "ExportDumpEvent should be emitted");
        assert_eq!(events[0].stockpile_entity, local_farm_entity);
        assert_eq!(events[0].commodity, ResourceType::Food);
        assert!(events[0].amount_dumped > 400.0);
        assert!(events[0].credits_earned > 0.0);
    }

    #[test]
    fn test_closed_trade_ignores_market_panic() {
        let mut app = App::new();
        app.add_event::<MarketPanicEvent>();
        app.add_event::<ExportDumpEvent>();
        app.add_systems(Update, process_market_panic_hoarding);

        let mut commodities = HashMap::new();
        commodities.insert(ResourceType::Food, 500.0);

        let local_farm_entity = app
            .world_mut()
            .spawn((
                LocalStockpile { commodities },
                TradePolicy { open: false }, // Closed borders
            ))
            .id();

        app.world_mut().send_event(MarketPanicEvent {
            commodity: ResourceType::Food,
            severity_multiplier: 5.0,
        });

        app.update();

        // Stock remains untouched because they didn't export
        let stock = app
            .world()
            .get::<LocalStockpile>(local_farm_entity)
            .unwrap();
        let food_amount = stock
            .commodities
            .get(&ResourceType::Food)
            .copied()
            .unwrap_or(0.0);
        assert_eq!(food_amount, 500.0);

        // Ensure no ExportDumpEvent was emitted
        let dump_events = app.world().resource::<Events<ExportDumpEvent>>();
        assert!(
            dump_events.is_empty(),
            "No ExportDumpEvent should be emitted"
        );
    }

    #[test]
    fn test_market_panic_skips_zero_stock() {
        let mut app = App::new();
        app.add_event::<MarketPanicEvent>();
        app.add_event::<ExportDumpEvent>();
        app.add_systems(Update, process_market_panic_hoarding);

        let mut commodities = HashMap::new();
        commodities.insert(ResourceType::Food, 0.0);

        app.world_mut()
            .spawn((LocalStockpile { commodities }, TradePolicy { open: true }));

        app.world_mut().send_event(MarketPanicEvent {
            commodity: ResourceType::Food,
            severity_multiplier: 5.0,
        });

        app.update();

        // Ensure no ExportDumpEvent was emitted because stock was 0
        let dump_events = app.world().resource::<Events<ExportDumpEvent>>();
        assert!(
            dump_events.is_empty(),
            "No ExportDumpEvent should be emitted"
        );
    }

    #[test]
    fn test_market_panic_ignores_unaffected_commodity() {
        let mut app = App::new();
        app.add_event::<MarketPanicEvent>();
        app.add_event::<ExportDumpEvent>();
        app.add_systems(Update, process_market_panic_hoarding);

        let mut commodities = HashMap::new();
        commodities.insert(ResourceType::Wood, 500.0); // Has wood

        let entity = app
            .world_mut()
            .spawn((LocalStockpile { commodities }, TradePolicy { open: true }))
            .id();

        app.world_mut().send_event(MarketPanicEvent {
            commodity: ResourceType::Food, // Panic is about food
            severity_multiplier: 5.0,
        });

        app.update();

        // Ensure wood remains untouched
        let stock = app.world().get::<LocalStockpile>(entity).unwrap();
        let wood_amount = stock
            .commodities
            .get(&ResourceType::Wood)
            .copied()
            .unwrap_or(0.0);
        assert_eq!(wood_amount, 500.0);

        // Ensure no ExportDumpEvent was emitted
        let dump_events = app.world().resource::<Events<ExportDumpEvent>>();
        assert!(
            dump_events.is_empty(),
            "No ExportDumpEvent should be emitted"
        );
    }
}

use bevy::prelude::*;
use scale::layer1::black_market::Smuggler;
use scale::layer1::integration::smuggler_arrival_event_bridge;
use scale::layer1::shadow_market::ShadowTrader;
use scale::layer1::trade::Merchant;
use scale::layer1::void_weed::MerchantArrivalEvent;

#[test]
fn test_smuggler_triggers_merchant_arrival_event() {
    let mut app = App::new();
    app.add_event::<MerchantArrivalEvent>();
    app.add_systems(Update, smuggler_arrival_event_bridge);

    // Spawn a regular black market smuggler
    app.world_mut().spawn(Smuggler);

    app.update();

    let events = app.world().resource::<Events<MerchantArrivalEvent>>();
    let mut reader = events.get_cursor();
    let emitted: Vec<_> = reader.read(events).collect();

    assert_eq!(emitted.len(), 1, "Should emit exactly one event");
    assert!(emitted[0].is_smuggler);
}

#[test]
fn test_shadow_trader_triggers_merchant_arrival_event() {
    let mut app = App::new();
    app.add_event::<MerchantArrivalEvent>();
    app.add_systems(Update, smuggler_arrival_event_bridge);

    // Spawn a shadow market trader
    app.world_mut().spawn(ShadowTrader {
        merchant: Merchant {
            name: "Shady Bob".to_string(),
            arrival_tick: 0,
            departure_tick: 100,
            deals: vec![],
        },
    });

    app.update();

    let events = app.world().resource::<Events<MerchantArrivalEvent>>();
    let mut reader = events.get_cursor();
    let emitted: Vec<_> = reader.read(events).collect();

    assert_eq!(emitted.len(), 1, "Should emit exactly one event");
    assert!(emitted[0].is_smuggler);
}

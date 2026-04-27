use bevy::prelude::*;
use scale::layer1::economy::inflation::{
    process_barter_trade, trigger_market_crash, BarterRequest, EmpireResources, MarketCrashEvent,
    MarketState,
};

#[test]
fn test_inflation_spiral_market_crash_integration() {
    let mut app = App::new();
    app.add_event::<MarketCrashEvent>();
    app.insert_resource(MarketState {
        credit_value_multiplier: 1.0,
    });
    app.add_systems(Update, trigger_market_crash);

    app.world_mut()
        .resource_mut::<Events<MarketCrashEvent>>()
        .send(MarketCrashEvent {
            new_multiplier: 0.1,
        });

    app.update();

    let market = app.world().resource::<MarketState>();
    assert_eq!(
        market.credit_value_multiplier, 0.1,
        "Market multiplier should be updated by the crash event"
    );
}

#[test]
fn test_inflation_spiral_barter_integration() {
    let mut app = App::new();
    app.add_event::<BarterRequest>();
    app.add_systems(Update, process_barter_trade);

    let empire_a = app
        .world_mut()
        .spawn(EmpireResources {
            credits: 1_000_000.0,
            alloys: 100,
        })
        .id();

    let empire_b = app
        .world_mut()
        .spawn(EmpireResources {
            credits: 10.0,
            alloys: 500,
        })
        .id();

    app.world_mut()
        .resource_mut::<Events<BarterRequest>>()
        .send(BarterRequest {
            initiator: empire_a,
            target: empire_b,
            offer_alloys: 100, // A gives 100
            request_alloys: 50,  // A wants 50
        });

    app.update();

    let a_res = app.world().get::<EmpireResources>(empire_a).unwrap();
    let b_res = app.world().get::<EmpireResources>(empire_b).unwrap();

    assert_eq!(a_res.alloys, 50, "Empire A should have 100 - 100 + 50 = 50 alloys");
    assert_eq!(b_res.alloys, 550, "Empire B should have 500 - 50 + 100 = 550 alloys");
}

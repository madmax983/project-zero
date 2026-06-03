use bevy::prelude::*;
use scale::layer1::law::prohibition::{
    calculate_smuggling_rates, decay_smuggling_rates, process_prohibition_events,
    update_black_market_prices, BaseValue, BlackMarketValue, Colony, ProhibitItemEvent, Prohibited,
    SmugglingRate,
};
use scale::layer1::economy::items::{Item, ItemType};

#[test]
fn test_prohibition_smuggling_bridge_integration() {
    let mut app = App::new();

    app.add_event::<ProhibitItemEvent>();

    app.add_systems(
        Update,
        (
            process_prohibition_events,
            update_black_market_prices.after(process_prohibition_events),
            calculate_smuggling_rates.after(update_black_market_prices),
            decay_smuggling_rates.after(calculate_smuggling_rates),
        ),
    );

    // Arrange: Create a colony and an item
    let colony_entity = app.world_mut().spawn((Colony, SmugglingRate(5.0))).id();
    let item_entity = app
        .world_mut()
        .spawn((
            Item {
                item_type: ItemType::Alcohol,
            },
            BaseValue(10.0),
        ))
        .id();

    // Act 1: Initial decay check (no prohibited items yet)
    app.update();

    // Smuggling rate should decay since there are no prohibited items
    let smuggling = app.world().get::<SmugglingRate>(colony_entity).unwrap();
    assert_eq!(smuggling.0, 4.0, "Smuggling rate should decay by 1.0 when no prohibited items exist");

    // Act 2: Prohibit the item
    app.world_mut().send_event(ProhibitItemEvent {
        item: item_entity,
        severity: 1.0,
    });

    app.update();

    // Assert: Item has Prohibited component
    assert!(app.world().entity(item_entity).contains::<Prohibited>());

    // Assert: Black market price multiplier is applied
    let bm_value = app.world().get::<BlackMarketValue>(item_entity).unwrap();
    assert_eq!(bm_value.0, 20.0, "Black market value should be 2x base value");

    // Assert: Smuggling rate increased (was 4.0, +5.0 = 9.0)
    let smuggling_after = app.world().get::<SmugglingRate>(colony_entity).unwrap();
    assert_eq!(smuggling_after.0, 9.0, "Smuggling rate should increase by 5.0 for 1 prohibited item");
}

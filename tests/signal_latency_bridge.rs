use bevy::prelude::*;
use scale::layer2::communications::signal_latency::{ExecuteOrderEvent, OrderType};
use scale::layer2::fleet::{FleetOrder, Fleet, InOrbit};
use scale::layer2::integration::signal_latency_fleet_bridge;

#[test]
fn test_signal_latency_fleet_bridge() {
    let mut app = App::new();
    app.add_event::<ExecuteOrderEvent>();
    app.add_systems(Update, signal_latency_fleet_bridge);

    let destination = app.world_mut().spawn_empty().id();
    let origin = app.world_mut().spawn_empty().id();

    let fleet = app.world_mut().spawn((
        Fleet,
        InOrbit { parent: origin }
    )).id();

    app.world_mut().send_event(ExecuteOrderEvent {
        target: fleet,
        order: OrderType::MoveTo(destination),
    });

    app.update();

    let order = app.world().get::<FleetOrder>(fleet);
    assert!(order.is_some(), "Fleet should receive a FleetOrder after ExecuteOrderEvent");
    if let Some(FleetOrder::MoveTo(target_dest)) = order {
        assert_eq!(*target_dest, destination);
    } else {
        panic!("Order was not MoveTo");
    }
}

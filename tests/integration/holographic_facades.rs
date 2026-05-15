use bevy_ecs::event::Events;
use bevy_ecs::prelude::*;
use scale::layer1::beauty::BeautySource;
use scale::layer1::energy::PowerConsumer;
use scale::layer1::hologram::{
    apply_disillusionment_system, update_holograms_system, HoloProjector, HologramFailureEvent,
};
use scale::layer1::map::GridPosition;
use scale::layer1::social::morale::Morale;

#[test]
fn test_holographic_facades_integration() {
    let mut world = World::new();
    world.init_resource::<Events<HologramFailureEvent>>();

    // Create Holographic Facade (Unpowered)
    world.spawn((
        HoloProjector {
            active_beauty: 50.0,
            radius: 5.0,
            is_active: true,
        }, // Start active to simulate sudden failure
        BeautySource {
            value: 50.0,
            radius: 5.0,
        },
        PowerConsumer {
            active: false,
            demand: 10.0,
        }, // Unpowered!
        GridPosition { x: 5, y: 5 },
    ));

    // Spawn a pop nearby
    let pop_id = world
        .spawn((GridPosition { x: 5, y: 5 }, Morale::default()))
        .id();

    let mut schedule = Schedule::default();
    schedule.add_systems((update_holograms_system, apply_disillusionment_system).chain());
    schedule.run(&mut world);

    let pop_morale = world.get::<Morale>(pop_id).unwrap();
    // Morale should have decreased! The initial mood modifier drops it by some amount.
    assert_eq!(
        pop_morale.modifiers.len(),
        1,
        "Pop should have received a Disillusionment modifier"
    );
    assert_eq!(
        pop_morale.modifiers[0].label, "Disillusionment",
        "Pop modifier should be Disillusionment"
    );
    assert_eq!(
        pop_morale.modifiers[0].value, -0.2,
        "Pop modifier value should be -0.2"
    );
}

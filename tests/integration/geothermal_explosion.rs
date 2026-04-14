use bevy::prelude::*;
use scale::layer1::environment::geothermal::{
    geothermal_decay_system, GeothermalPulseState, GeothermalVent,
};
use scale::layer1::environment::volatile::{handle_explosion_system, ExplosionEvent};
use scale::layer1::map::GridPosition;
use scale::layer1::structure::Structure;
use scale::layer1::energy::PowerSource;

#[test]
fn test_geothermal_explosion_damages_adjacent_buildings() {
    let mut app = App::new();

    // Register events and states
    app.init_resource::<GeothermalPulseState>();
    app.add_event::<ExplosionEvent>();

    app.world_mut().resource_mut::<GeothermalPulseState>().is_pulsing = true;

    // Chain systems: decay creates the explosion event, handle applies the damage
    app.add_systems(Update, (geothermal_decay_system, handle_explosion_system).chain());

    let vent_pos = GridPosition { x: 5, y: 5 };
    let adj_pos = GridPosition { x: 5, y: 6 };

    app.world_mut().spawn((GeothermalVent, vent_pos));

    // Building about to explode
    let building_entity = app
        .world_mut()
        .spawn((
            PowerSource {
                output: 10.0,
                active: true,
            },
            Structure {
                current_hp: 1.0,
                max_hp: 100.0,
            }, // 1 health, will drop below 0 and explode
            vent_pos,
        ))
        .id();

    // Adjacent building to take collateral damage
    let adj_building = app
        .world_mut()
        .spawn((
            Structure {
                current_hp: 100.0,
                max_hp: 100.0,
            },
            adj_pos,
        ))
        .id();

    app.update();

    // The vent building should be destroyed
    assert!(
        app.world().get::<Structure>(building_entity).is_none(),
        "Building should be destroyed"
    );

    // Verify ExplosionEvent was consumed/processed
    let events = app.world().resource::<Events<ExplosionEvent>>();
    assert!(!events.is_empty(), "Explosion event was not fired");

    // Adjacent building should take damage from handle_explosion_system
    let adj_health = app
        .world()
        .get::<Structure>(adj_building)
        .unwrap()
        .current_hp;
    assert!(
        adj_health < 100.0,
        "Adjacent building should take collateral explosion damage via integration seam"
    );
}

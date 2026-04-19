use bevy::prelude::*;
use scale::layer2::orbit::debris_cult::{evaluate_debris_cult_formation_system, apply_debris_cult_morale_system, DebrisCultist};
use scale::layer2::debris::OrbitalDebris;
use scale::layer2::station::{Station, StationType};
use scale::layer1::entities::pop::Pop;
use scale::layer1::social::morale::Morale;

#[test]
fn test_debris_cult_bridge() {
    let mut app = App::new();
    app.add_systems(Update, (evaluate_debris_cult_formation_system, apply_debris_cult_morale_system).chain());

    // Spawn 5 pieces of debris
    for _ in 0..10 {
        app.world_mut().spawn(OrbitalDebris(1.0));
    }

    let entity = app.world_mut().spawn((
        Pop,
        Station { station_type: StationType::Outpost },
        Morale { value: 0.5, ..Default::default() },
    )).id();

    app.update();

    assert!(app.world().get::<DebrisCultist>(entity).is_some());
    let morale = app.world().get::<Morale>(entity).unwrap();
    // 10 debris * 0.05 bonus per debris -> 0.5
    // Should have 1 modifier
    assert_eq!(morale.modifiers.len(), 1);
    assert_eq!(morale.modifiers[0].label, "Orbital Debris Cult");
    assert!((morale.modifiers[0].value - 0.5).abs() < 0.01);
}

use bevy_ecs::prelude::*;
use scale::layer1::economy::resources::ColonyResources;
use scale::layer1::map::GridPosition;
use scale::layer1::morale::Morale;
use scale::layer1::pop::Pop;
use scale::layer1::social::xenoflora_pet::{
    apply_pet_mood_boost, pet_viral_spread_system, XenofloraPet,
};

#[test]
fn test_pet_system_integration() {
    let mut app = bevy_app::App::new();

    app.insert_resource(ColonyResources {
        food: 10.0,
        ..ColonyResources::zeroed()
    });
    app.add_systems(
        bevy_app::Update,
        (pet_viral_spread_system, apply_pet_mood_boost).chain(),
    );

    let pop1 = app
        .world_mut()
        .spawn((
            Pop,
            Morale {
                value: 0.5,
                modifiers: vec![],
            },
            XenofloraPet {
                resource_upkeep: 1.0,
                mood_boost: 0.25,
            },
            GridPosition { x: 10, y: 10 },
        ))
        .id();

    // Another pop nearby to catch the pet
    let pop2 = app
        .world_mut()
        .spawn((
            Pop,
            Morale {
                value: 0.5,
                modifiers: vec![],
            },
            GridPosition { x: 11, y: 11 },
        ))
        .id();

    // Run until pop2 catches the pet (or enough times to be sure)
    for _ in 0..1000 {
        app.update();
        if app.world().get::<XenofloraPet>(pop2).is_some() {
            break;
        }
    }

    assert!(
        app.world().get::<XenofloraPet>(pop2).is_some(),
        "Pop2 should have caught the pet"
    );

    // Run one more time to apply mood boost for pop2
    app.update();

    let morale1 = app.world().get::<Morale>(pop1).unwrap();
    assert!(!morale1.modifiers.is_empty());

    let morale2 = app.world().get::<Morale>(pop2).unwrap();
    assert!(!morale2.modifiers.is_empty());
}

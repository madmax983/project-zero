use bevy::prelude::*;
use scale::layer1::core::integration::update_renown_from_morale_system;
use scale::layer1::living_score::ColonyRenown;
use scale::layer1::morale::Morale;
use scale::layer1::pop::Pop;

#[test]
fn test_update_renown_from_morale() {
    let mut app = App::new();
    app.insert_resource(ColonyRenown { score: 50.0 });
    app.add_systems(Update, update_renown_from_morale_system);

    app.world_mut().spawn((
        Pop,
        Morale {
            value: 0.8,
            modifiers: Vec::new(),
        },
    ));
    app.world_mut().spawn((
        Pop,
        Morale {
            value: 0.4,
            modifiers: Vec::new(),
        },
    ));

    app.update();

    let renown = app.world().resource::<ColonyRenown>();
    println!("Renown Score: {}", renown.score);
    // Average of 0.8 and 0.4 is 0.6 -> score should be 60.0
    // Use an epsilon for floating point comparison to avoid precision errors
    assert!((renown.score - 60.0).abs() < 0.001);
}

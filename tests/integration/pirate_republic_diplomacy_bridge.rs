use bevy::prelude::*;
use scale::layer2::piracy::PirateRepublic;
use scale::layer3::diplomacy_reflection::{Civilization, DiplomaticTraits, DiplomaticRelations};
use scale::layer3::integration::pirate_republic_diplomacy_bridge;

#[test]
fn test_pirate_republic_diplomacy_bridge() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(Update, pirate_republic_diplomacy_bridge);

    let entity = app.world_mut().spawn(PirateRepublic).id();

    app.update();

    let civ = app.world().get::<Civilization>(entity);
    assert!(civ.is_some());
    assert_eq!(civ.unwrap().id, format!("PirateRepublic_{:?}", entity));

    let traits = app.world().get::<DiplomaticTraits>(entity);
    assert!(traits.is_some());
    assert!(traits.unwrap().is_barbarian);

    let relations = app.world().get::<DiplomaticRelations>(entity);
    assert!(relations.is_some());
}

use scale::layer1::map::GridPosition;
use scale::layer1::items::{Item, ItemType};
use scale::layer1::olfactory::{ScentEmitter, ScentType};
use scale::layer1::integration::item_scent_bridge_system;

#[test]
fn test_item_scent_bridge_adds_foul_scent_to_waste_and_corpses() {
    let mut app = bevy_app::App::new();

    // Add the new bridge system
    app.add_systems(bevy_app::Update, item_scent_bridge_system);

    let pos1 = GridPosition { x: 1, y: 1 };
    let pos2 = GridPosition { x: 2, y: 2 };
    let dummy_entity = app.world_mut().spawn_empty().id();

    // Spawn Waste
    let waste = app.world_mut().spawn((
        Item { item_type: ItemType::Waste },
        pos1,
    )).id();

    // Spawn Corpse
    let corpse = app.world_mut().spawn((
        Item { item_type: ItemType::Corpse(dummy_entity) },
        pos2,
    )).id();

    // Spawn normal Item (e.g. Potato) - shouldn't get scent
    let potato = app.world_mut().spawn((
        Item { item_type: ItemType::Potato },
        pos1,
    )).id();

    app.update();

    // Check Waste
    let waste_emitter = app.world().get::<ScentEmitter>(waste).expect("Waste should have a ScentEmitter");
    assert_eq!(waste_emitter.scent_type, ScentType::Foul, "Waste should emit a foul scent");

    // Check Corpse
    let corpse_emitter = app.world().get::<ScentEmitter>(corpse).expect("Corpse should have a ScentEmitter");
    assert_eq!(corpse_emitter.scent_type, ScentType::Foul, "Corpse should emit a foul scent");

    // Check Potato
    assert!(app.world().get::<ScentEmitter>(potato).is_none(), "Normal items should not emit foul scents by default");
}

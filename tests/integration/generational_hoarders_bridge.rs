use bevy_app::{App, Update};
use bevy_ecs::prelude::*;
use scale::layer1::economy::inventory::{Inventory, InventoryItem};
use scale::layer1::economy::items::ItemType;
use scale::layer1::lifecycle::Age;
use scale::layer1::psychology::traits::{Trait, Traits};
use scale::layer1::social::hoarder::{
    apply_hoard_morale_buff_system, check_for_hoarder_trait_system, hoarder_collection_system,
    process_confiscation_system, ConfiscateHoardEvent, Hoard,
};
use scale::layer1::social::morale::Morale;

#[test]
fn test_generational_hoarders_full_seam() {
    let mut app = App::new();
    app.init_resource::<Events<ConfiscateHoardEvent>>();

    app.add_systems(
        Update,
        (
            check_for_hoarder_trait_system,
            hoarder_collection_system,
            apply_hoard_morale_buff_system,
            process_confiscation_system,
        )
            .chain(),
    );

    let mut inventory = Inventory {
        capacity: 10,
        ..Default::default()
    };
    inventory.items.push(InventoryItem {
        item_type: ItemType::Tool,
        entity: None,
    });
    inventory.items.push(InventoryItem {
        item_type: ItemType::Tool,
        entity: None,
    });
    let stockpile = app.world_mut().spawn(inventory).id();

    let elder = app
        .world_mut()
        .spawn((
            Age::new(65),
            Traits::default(),
            Hoard::default(),
            Morale::default(),
        ))
        .id();

    // 1. Tick: Elder gets trait, collects tool, gets morale buff
    app.update();

    let traits = app.world().get::<Traits>(elder).unwrap();
    assert!(traits.has(Trait::Hoarder));

    let hoard = app.world().get::<Hoard>(elder).unwrap();
    assert!(hoard.items.contains(&ItemType::Tool));

    let stockpile_inv = app.world().get::<Inventory>(stockpile).unwrap();
    assert_eq!(stockpile_inv.items.len(), 1);

    let morale = app.world().get::<Morale>(elder).unwrap();
    assert!(!morale.modifiers.is_empty());
    assert!(morale.modifiers.iter().any(|m| m.value > 0.0));

    // 2. Tick: Send confiscation event
    app.world_mut()
        .send_event(ConfiscateHoardEvent { target: elder });
    app.update();

    let empty_hoard = app.world().get::<Hoard>(elder).unwrap();
    assert!(empty_hoard.items.is_empty());

    let final_morale = app.world().get::<Morale>(elder).unwrap();
    assert!(final_morale.modifiers.iter().any(|m| m.value < 0.0));
}

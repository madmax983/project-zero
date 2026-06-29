use bevy::prelude::*;
use scale::layer1::improvised_tools::{evaluate_tool_fallback_system, ImprovisedTools};
use scale::layer1::jobs::CurrentTask;
use scale::layer1::economy::inventory::{Inventory, InventoryItem};
use scale::layer1::economy::items::{Equipment, ItemType};

#[test]
fn test_improvised_tools_bridge() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    app.add_systems(Update, evaluate_tool_fallback_system);

    let mut inventory = Inventory::default();
    inventory.try_add(InventoryItem {
        item_type: ItemType::Scrap,
        entity: None,
    });

    let pop_entity = app.world_mut().spawn((
        CurrentTask {
            efficiency: 1.0,
            ..Default::default()
        },
        Equipment::default(),
        inventory,
        ImprovisedTools::default(),
    )).id();

    app.update();

    let task = app.world().get::<CurrentTask>(pop_entity).unwrap();
    let improvised_tools = app.world().get::<ImprovisedTools>(pop_entity).unwrap();

    assert!(improvised_tools.in_use, "Pop should be using improvised tools");
    assert_eq!(task.efficiency, 0.5, "Task efficiency should be halved");
}

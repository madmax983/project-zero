use crate::layer1::economy::inventory::Inventory;
use crate::layer1::economy::items::{Equipment, ItemType};
use crate::layer1::jobs::CurrentTask;
use bevy::prelude::*;

#[derive(Component, Default)]
pub struct ImprovisedTools {
    pub in_use: bool,
}

pub fn evaluate_tool_fallback_system(
    mut query: Query<(
        &mut CurrentTask,
        &mut ImprovisedTools,
        &Inventory,
        &Equipment,
    )>,
) {
    for (mut task, mut improvised_tools, inventory, equipment) in query.iter_mut() {
        if equipment.tool.is_none() && inventory.has_item(ItemType::Scrap) {
            improvised_tools.in_use = true;
            task.efficiency = 0.5;
        } else {
            improvised_tools.in_use = false;
            task.efficiency = 1.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::economy::inventory::{Inventory, InventoryItem};
    use crate::layer1::economy::items::{Equipment, ItemType};
    use crate::layer1::jobs::CurrentTask;

    #[test]
    fn test_improvised_tools_fallback_scrap() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let mut inventory = Inventory::default();
        inventory.try_add(InventoryItem {
            item_type: ItemType::Scrap,
            entity: None,
        });

        let pop_entity = app
            .world_mut()
            .spawn((
                CurrentTask {
                    efficiency: 1.0,
                    ..Default::default()
                },
                Equipment::default(),
                inventory,
                ImprovisedTools::default(),
            ))
            .id();

        app.add_systems(Update, evaluate_tool_fallback_system);
        app.update();

        let task = app.world().get::<CurrentTask>(pop_entity).unwrap();
        let improvised_tools = app.world().get::<ImprovisedTools>(pop_entity).unwrap();

        assert!(improvised_tools.in_use);
        assert_eq!(task.efficiency, 0.5);
    }

    #[test]
    fn test_improvised_tools_fallback_none() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let mut inventory = Inventory::default();
        inventory.try_add(InventoryItem {
            item_type: ItemType::Meat,
            entity: None,
        });

        let pop_entity = app
            .world_mut()
            .spawn((
                CurrentTask {
                    efficiency: 1.0,
                    ..Default::default()
                },
                Equipment::default(),
                inventory,
                ImprovisedTools::default(),
            ))
            .id();

        app.add_systems(Update, evaluate_tool_fallback_system);
        app.update();

        let task = app.world().get::<CurrentTask>(pop_entity).unwrap();
        let improvised_tools = app.world().get::<ImprovisedTools>(pop_entity).unwrap();

        assert!(!improvised_tools.in_use);
        assert_eq!(task.efficiency, 1.0); // didn't drop
    }
}

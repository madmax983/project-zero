use bevy::prelude::*;
use crate::layer1::jobs::CurrentTask;
use crate::layer1::economy::inventory::Inventory;
use crate::layer1::economy::items::ItemType;
use crate::layer1::items::Equipment;

/// Component indicating a pop is currently using an improvised tool
#[derive(Component)]
pub struct ImprovisedTools;

pub fn evaluate_tool_fallback_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut CurrentTask, &Inventory, &Equipment), Without<ImprovisedTools>>,
) {
    for (entity, mut task, inventory, equipment) in query.iter_mut() {
        if equipment.tool.is_none() {
            // Priority: Scrap > Stone > Wood (this follows the design in general_work.rs)
            if inventory.has_item(ItemType::Scrap)
                || inventory.has_item(ItemType::LivingStone)
            {
                task.efficiency *= 0.5;
                commands.entity(entity).insert(ImprovisedTools);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::economy::inventory::InventoryItem;

    #[test]
    fn test_improvised_tools_fallback() {
        let mut app = App::new();
        app.add_systems(Update, evaluate_tool_fallback_system);

        let mut inv = Inventory::default();
        inv.try_add(InventoryItem {
            item_type: ItemType::Scrap,
            entity: None,
        });

        let pop_entity = app.world_mut().spawn((
            CurrentTask { efficiency: 1.0, ..Default::default() },
            inv,
            Equipment { tool: None, ..Default::default() },
        )).id();

        app.update();

        let task = app.world().get::<CurrentTask>(pop_entity).unwrap();
        assert_eq!(task.efficiency, 0.5);
        assert!(app.world().get::<ImprovisedTools>(pop_entity).is_some());
    }
}

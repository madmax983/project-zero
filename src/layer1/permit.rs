use crate::layer1::inventory::Inventory;
use crate::layer1::resources::ResourceType;
use bevy_ecs::prelude::*;

/// Component indicating a building requires a permit to function.
#[derive(Component, Default, Debug, Clone, Copy)]
pub struct PermitRequired;

/// Consumes a permit from the building's inventory to activate it.
pub fn permit_activation_system(
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            &mut Inventory,
            Option<&mut crate::layer1::energy::PowerConsumer>,
        ),
        With<PermitRequired>,
    >,
) {
    for (entity, mut inventory, power) in &mut query {
        // Check for permit
        if let Some(index) = inventory.items.iter().position(|item| {
            item.item_type.as_resource_type() == Some(ResourceType::BuildingPermit)
        }) {
            // Remove permit
            inventory.items.remove(index);
            // Remove requirement
            commands.entity(entity).remove::<PermitRequired>();

            // Re-enable power
            if let Some(mut p) = power {
                p.active = true;
            }
        }
    }
}

/// Enforces restrictions on buildings that require a permit.
/// - Disables [`crate::layer1::energy::PowerConsumer`]
/// - Prevents work assignments (handled by job system check)
pub fn enforce_permit_restrictions_system(
    mut query: Query<&mut crate::layer1::energy::PowerConsumer, With<PermitRequired>>,
) {
    for mut consumer in &mut query {
        consumer.active = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::building::{
        spawn_building_with_material, Building, BuildingType, MaterialType,
    };
    use crate::layer1::energy::PowerConsumer;
    use crate::layer1::inventory::{Inventory, InventoryItem};
    use crate::layer1::items::ItemType;
    use crate::layer1::resources::ColonyResources;

    // Helper to setup world
    fn setup() -> World {
        let mut world = World::new();
        world.insert_resource(crate::layer1::building::OccupiedTiles::default());
        world.insert_resource(crate::layer1::tech::TechState::default());
        world.insert_resource(ColonyResources::default());
        // Add resources needed for spawning
        world.insert_resource(crate::layer1::terrain::TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![crate::layer1::terrain::TerrainType::Grass; 100],
        });
        world.insert_resource(crate::layer1::prototyping::BuildingMastery::default());
        world.insert_resource(crate::shared::log::MessageLog::default());
        world
    }

    #[test]
    fn test_advanced_building_spawns_with_permit_requirement() {
        // Arrange
        let mut world = setup();

        // Act
        // Smelter is Tier 2 (Advanced), should require permit
        spawn_building_with_material(
            &mut world,
            0,
            0,
            BuildingType::Smelter,
            MaterialType::default(),
        );

        let (entity, _) = world.query::<(Entity, &Building)>().single(&world);

        // Assert
        assert!(
            world.get::<PermitRequired>(entity).is_some(),
            "Smelter should require a permit"
        );
        assert!(
            world.get::<Inventory>(entity).is_some(),
            "Smelter should have inventory for permit"
        );
    }

    #[test]
    fn test_basic_building_spawns_active() {
        // Arrange
        let mut world = setup();

        // Act
        // Farm is Tier 1 (Basic), should NOT require permit
        spawn_building_with_material(
            &mut world,
            0,
            0,
            BuildingType::Farm,
            MaterialType::default(),
        );

        let (entity, _) = world.query::<(Entity, &Building)>().single(&world);

        // Assert
        assert!(
            world.get::<PermitRequired>(entity).is_none(),
            "Farm should not require a permit"
        );
    }

    #[test]
    fn test_permit_blocks_functionality() {
        // Arrange
        let mut world = World::new();
        let id = world
            .spawn((
                Building {
                    building_type: BuildingType::Smelter,
                },
                PermitRequired,
                PowerConsumer {
                    active: true,
                    ..Default::default()
                }, // Normally active
            ))
            .id();

        // Act
        // Run a system that enforces permit restrictions
        let mut schedule = Schedule::default();
        schedule.add_systems(enforce_permit_restrictions_system);
        schedule.run(&mut world);

        // Assert
        let consumer = world.get::<PowerConsumer>(id).unwrap();
        assert!(
            !consumer.active,
            "PowerConsumer should be disabled by PermitRequired"
        );
    }

    #[test]
    fn test_delivering_permit_activates_building() {
        // Arrange
        let mut world = World::new();
        let id = world
            .spawn((
                Building {
                    building_type: BuildingType::Smelter,
                },
                PermitRequired,
                Inventory::default(), // Inventory to receive the permit
                PowerConsumer {
                    active: false,
                    ..Default::default()
                }, // Disabled initially
            ))
            .id();

        // Simulate Hauler delivering the permit
        // We use ItemType::BuildingPermit which maps to ResourceType::BuildingPermit
        let mut inventory = world.get_mut::<Inventory>(id).unwrap();
        inventory.try_add(InventoryItem {
            item_type: ItemType::BuildingPermit,
            entity: None,
        });

        // Act
        let mut schedule = Schedule::default();
        schedule.add_systems(permit_activation_system);
        schedule.run(&mut world);

        // Assert
        assert!(
            world.get::<PermitRequired>(id).is_none(),
            "PermitRequired should be removed"
        );

        let inventory = world.get::<Inventory>(id).unwrap();
        assert!(inventory.items.is_empty(), "Permit should be consumed");

        let power = world.get::<PowerConsumer>(id).unwrap();
        assert!(power.active, "Power should be re-enabled");
    }
}

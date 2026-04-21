use crate::layer1::biology::health::{Health, HealthCondition};
use crate::layer1::entities::pop::Pop;
use bevy_ecs::prelude::*;

/// Applies toxic gas damage to a Pop, unless they have RustLung immunity.
pub fn apply_toxic_gas_damage(world: &mut World, pop_id: Entity, damage: f32) {
    if let Some(mut health) = world.get_mut::<Health>(pop_id) {
        if !health.has_condition(HealthCondition::RustLung) {
            health.take_damage(damage);
        }
    }
}

/// A system that runs every tick to degrade the health of Pops with RustLung.
pub fn rust_lung_degradation_system(mut query: Query<&mut Health, With<Pop>>) {
    for mut health in query.iter_mut() {
        if health.has_condition(HealthCondition::RustLung) {
            // Degrades health slightly over time
            health.take_damage(0.01);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::economy::inventory::{Inventory, InventoryItem};
    use crate::layer1::economy::items::ItemType;
    use crate::layer1::execution::mining::handle_mining_work;
    use crate::layer1::map::GridPosition;
    use crate::layer1::purity::PurityMap;
    use crate::layer1::resources::{ColonyResources, MiningProgress};
    use crate::layer1::structural_integrity::RoofGrid;
    use crate::layer1::terrain::{TerrainGrid, TerrainType};
    use crate::shared::log::MessageLog;
    use bevy_app::prelude::*;

    #[test]
    fn test_rust_lung_accumulation_from_mining() {
        let mut app = App::new();
        // Setup world resources needed for mining
        let mut tiles = vec![TerrainType::Grass; 100];
        tiles[0] = TerrainType::Rock;
        app.world_mut().insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });
        app.world_mut().insert_resource(ColonyResources::default());
        app.world_mut().insert_resource(RoofGrid::new(10, 10));
        app.world_mut().insert_resource(MessageLog::default());

        let mut purity_map = PurityMap::default();
        purity_map.set_override(0, 0, 0.0); // Low purity
        app.world_mut().insert_resource(purity_map);

        // Setup mining pop
        let pop_id = app
            .world_mut()
            .spawn((Pop, Health::default(), Inventory::default()))
            .id();

        let designation = app
            .world_mut()
            .spawn((
                GridPosition { x: 0, y: 0 },
                MiningProgress {
                    current: 0.0,
                    max: 10.0,
                },
            ))
            .id();

        // Simulate mining low purity ore without rebreather
        handle_mining_work(
            app.world_mut(),
            designation,
            pop_id,
            1.0,
            Some(GridPosition { x: 0, y: 0 }),
        );

        let health = app.world().get::<Health>(pop_id).unwrap();
        assert!(health.has_condition(HealthCondition::RustLung));
    }

    #[test]
    fn test_rust_lung_provides_toxic_gas_immunity() {
        let mut app = App::new();

        // Setup pop with Rust-Lung
        let mut health = Health::default();
        health.add_condition(HealthCondition::RustLung);
        let pop_id = app.world_mut().spawn((Pop, health.clone())).id();

        // Apply toxic gas damage
        apply_toxic_gas_damage(app.world_mut(), pop_id, 10.0);

        let current_health = app.world().get::<Health>(pop_id).unwrap();
        assert_eq!(
            current_health.current, current_health.max,
            "Pop with Rust-Lung should not take toxic gas damage"
        );

        // Ensure pop without Rust-Lung does take damage
        let pop_no_immunity = app.world_mut().spawn((Pop, Health::default())).id();
        apply_toxic_gas_damage(app.world_mut(), pop_no_immunity, 10.0);
        let current_health_no_immunity = app.world().get::<Health>(pop_no_immunity).unwrap();
        assert_eq!(
            current_health_no_immunity.current,
            current_health_no_immunity.max - 10.0,
            "Pop without Rust-Lung should take toxic gas damage"
        );
    }

    #[test]
    fn test_rust_lung_degradation() {
        let mut app = App::new();
        app.add_systems(Update, rust_lung_degradation_system);

        let mut health = Health::default();
        health.add_condition(HealthCondition::RustLung);
        let pop_id = app.world_mut().spawn((Pop, health)).id();

        app.update();

        let current_health = app.world().get::<Health>(pop_id).unwrap();
        assert!(
            current_health.current < current_health.max,
            "Health should degrade over time with RustLung"
        );
    }

    #[test]
    fn test_mine_low_purity_ore_with_rebreather_prevents_rust_lung() {
        let mut app = App::new();
        // Setup world resources needed for mining
        let mut tiles = vec![TerrainType::Grass; 100];
        tiles[0] = TerrainType::Rock;
        app.world_mut().insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });
        app.world_mut().insert_resource(ColonyResources::default());
        app.world_mut().insert_resource(RoofGrid::new(10, 10));
        app.world_mut().insert_resource(MessageLog::default());

        let mut purity_map = PurityMap::default();
        purity_map.set_override(0, 0, 0.0); // Low purity
        app.world_mut().insert_resource(purity_map);

        let mut inventory = Inventory::default();
        inventory.items.push(InventoryItem {
            item_type: ItemType::Rebreather,
            entity: None,
        });

        let pop_id = app
            .world_mut()
            .spawn((Pop, Health::default(), inventory))
            .id();

        let designation = app
            .world_mut()
            .spawn((
                GridPosition { x: 0, y: 0 },
                MiningProgress {
                    current: 0.0,
                    max: 10.0,
                },
            ))
            .id();

        // Simulate mining
        handle_mining_work(
            app.world_mut(),
            designation,
            pop_id,
            1.0,
            Some(GridPosition { x: 0, y: 0 }),
        );

        let health = app.world().get::<Health>(pop_id).unwrap();
        assert!(
            !health.has_condition(HealthCondition::RustLung),
            "Pop with Rebreather should not get RustLung"
        );
    }
}

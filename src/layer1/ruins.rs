//! Ruins system.
//!
//! Handles the creation, storage, and scavenging of ruined buildings.
//! When a building is destroyed (e.g. by fire), it is replaced by a `Ruin` entity.
//! Ruins block construction but can be scavenged to recover resources.

use bevy_ecs::prelude::*;
use crate::layer1::building::{BuildingType, MaterialType, OccupiedTiles};
use crate::layer1::resources::{ColonyResources, ResourceItem, ResourceType};
use crate::layer1::GridPosition;

/// Component representing a ruined building.
#[derive(Component, Debug, Clone, Copy)]
pub struct Ruin {
    /// The type of the original building.
    pub original_type: BuildingType,
}

/// Component storing the history of a ruin.
#[derive(Component, Debug, Clone)]
pub struct RuinHistory {
    /// The tick when the building was destroyed.
    pub destruction_tick: u64,
    /// The reason for destruction (e.g., "Fire", "Decay").
    pub reason: String,
}

/// Processes the scavenging of a ruin, returning yield info and cleaning up.
pub fn process_scavenge(world: &mut World, ruin_entity: Entity) -> Vec<ResourceType> {
    let mut yielded = Vec::new();
    let pos = world.get::<GridPosition>(ruin_entity).copied();

    let (cost, label) = if let Some(ruin) = world.get::<Ruin>(ruin_entity) {
        (
            ruin.original_type.cost(MaterialType::default()),
            ruin.original_type.label().to_string(),
        )
    } else {
        return vec![];
    };

    let mut resources_to_spawn = ColonyResources::default();

    // 20% yield
    if cost.wood > 0.0 {
        resources_to_spawn.wood = (cost.wood * 0.2).max(1.0); // Minimum 1 if cost > 0
        yielded.push(ResourceType::Wood);
    }
    if cost.stone > 0.0 {
        resources_to_spawn.stone = (cost.stone * 0.2).max(1.0);
        yielded.push(ResourceType::Stone);
    }
    if cost.metal > 0.0 {
        resources_to_spawn.metal = (cost.metal * 0.2).max(1.0);
        yielded.push(ResourceType::Metal);
    }

    if let Some(p) = pos {
        if resources_to_spawn.wood > 0.0 {
            spawn_resource_item(world, p, ResourceType::Wood, resources_to_spawn.wood);
        }
        if resources_to_spawn.stone > 0.0 {
            spawn_resource_item(world, p, ResourceType::Stone, resources_to_spawn.stone);
        }
        if resources_to_spawn.metal > 0.0 {
            spawn_resource_item(world, p, ResourceType::Metal, resources_to_spawn.metal);
        }

        // Log
        if let Some(mut log) = world.get_resource_mut::<crate::shared::log::MessageLog>() {
            log.add(format!("Scavenged ruin of {}", label));
        }
    }

    // Despawn Ruin
    world.despawn(ruin_entity);

    // Clear OccupiedTiles
    if let Some(p) = pos {
        if let Some(mut occupied) = world.get_resource_mut::<OccupiedTiles>() {
            occupied.0.remove(&(p.x, p.y));
        }
    }

    yielded
}

fn spawn_resource_item(world: &mut World, pos: GridPosition, res_type: ResourceType, amount: f32) {
    world.spawn((
        ResourceItem {
            resource_type: res_type,
            amount,
        },
        pos,
    ));
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;
    use crate::layer1::building::{Building, BuildingType, OccupiedTiles};
    use crate::layer1::structure::{Structure, fire_damage_structure_system};
    use crate::layer1::fire::{Fire, Flammable};
    use crate::layer1::GridPosition;
    use crate::shared::time::SimulationTime;
    use crate::layer1::terrain::TerrainGrid;
    use crate::layer1::terrain::TerrainType;

    fn setup_world() -> World {
        let mut world = World::new();
        world.insert_resource(OccupiedTiles::default());
        world.insert_resource(SimulationTime::default());
        world.insert_resource(crate::shared::log::MessageLog::default());
        let tiles = vec![TerrainType::Grass; 100];
        world.insert_resource(TerrainGrid { width: 10, height: 10, tiles });
        world
    }

    #[test]
    fn test_destruction_spawns_ruin() {
        let mut world = setup_world();

        // Spawn a building with 1 HP
        let pos = GridPosition { x: 5, y: 5 };
        let building = world.spawn((
            Building { building_type: BuildingType::Housing },
            Structure { current_hp: 1.0, max_hp: 100.0 },
            Flammable::default(),
            pos,
        )).id();

        // Register occupation
        world.resource_mut::<OccupiedTiles>().0.insert((5, 5));

        // Spawn fire to destroy it
        world.spawn((
            Fire { intensity: 10.0, lifetime: 10 },
            pos,
        ));

        // Run damage system
        fire_damage_structure_system(&mut world);

        // Assert Building is gone
        assert!(world.get_entity(building).is_err(), "Building should be despawned");

        // Assert Ruin exists at same pos
        let mut ruin_query = world.query::<(&Ruin, &GridPosition)>();
        let (ruin, ruin_pos) = ruin_query.single(&world);

        assert_eq!(ruin.original_type, BuildingType::Housing);
        assert_eq!(*ruin_pos, pos);

        // Assert Tile is still Occupied (Ruins block construction)
        assert!(world.resource::<OccupiedTiles>().0.contains(&(5, 5)));
    }

    #[test]
    fn test_ruin_history_recorded() {
        let mut world = setup_world();
        world.insert_resource(SimulationTime { tick: 100, ..Default::default() });

        let pos = GridPosition { x: 0, y: 0 };
        world.spawn((
            Building { building_type: BuildingType::Housing },
            Structure { current_hp: 0.0, max_hp: 100.0 }, // Already dead
            Flammable::default(),
            pos,
        ));

        // Spawn fire to trigger the system
        world.spawn((
            Fire { intensity: 1.0, lifetime: 10 },
            pos,
        ));

        // Trigger system
        fire_damage_structure_system(&mut world);

        let (history, _) = world.query::<(&RuinHistory, &Ruin)>().single(&world);
        assert_eq!(history.destruction_tick, 100);
        assert!(history.reason.contains("Fire") || history.reason.contains("Damage"));
    }

    #[test]
    fn test_scavenge_ruin_yields_resources() {
        let mut world = setup_world();

        // Wall (Wood) cost: 5.0. 20% = 1.0.
        let ruin = world.spawn((
            Ruin { original_type: BuildingType::Wall },
            GridPosition { x: 2, y: 2 },
            crate::layer1::structure::Structure::default(),
        )).id();

        world.resource_mut::<OccupiedTiles>().0.insert((2, 2));

        // Mock Scavenge execution
        let yielded = crate::layer1::ruins::process_scavenge(&mut world, ruin);

        assert!(!yielded.is_empty(), "Scavenging should yield resources");
        assert!(yielded.contains(&ResourceType::Wood));

        // Ruin should be despawned
        assert!(world.get_entity(ruin).is_err());

        // OccupiedTiles should be cleared
        assert!(!world.resource::<OccupiedTiles>().0.contains(&(2, 2)));
    }
}

use bevy_ecs::prelude::*;
use crate::layer2::fleet::{WorldEater, FleetCommand};
use crate::layer1::nature::terrain::{TerrainGrid, TerrainType};
use crate::layer1::economy::resources::ColonyResources;

pub fn process_world_eater_system(
    mut events: EventReader<FleetCommand>,
    mut grid: ResMut<TerrainGrid>,
    mut resources: ResMut<ColonyResources>,
    query: Query<&WorldEater>,
) {
    for event in events.read() {
        let FleetCommand::ConsumeTile { fleet, target } = event;
        if let Ok(eater) = query.get(*fleet) {
            // If the tile isn't already bedrock/void, consume it
            if let Some(tile) = grid.get(target.x as usize, target.y as usize) {
                if tile != TerrainType::Void {
                    grid.set(target.x as usize, target.y as usize, TerrainType::Void);

                    // Refactor: Yield Variance
                    match tile {
                        TerrainType::Tree | TerrainType::Sapling => {
                            resources.wood += eater.efficiency;
                        }
                        TerrainType::Rock | TerrainType::DeepRock | TerrainType::Crater | TerrainType::IndestructibleStump | TerrainType::MagmaRock => {
                            resources.stone += eater.efficiency * 0.8;
                            resources.ore += eater.efficiency * 0.2;
                        }
                        TerrainType::Water => {
                            // Water might not yield solid resources
                        }
                        TerrainType::Grass | TerrainType::Dirt | TerrainType::Path | TerrainType::Shrub | TerrainType::SporeBloom => {
                            resources.fiber += eater.efficiency * 0.5;
                            resources.food += eater.efficiency * 0.1;
                        }
                        TerrainType::Artifact => {
                            resources.knowledge += eater.efficiency;
                        }
                        _ => {
                            // Default fallback
                            resources.stone += eater.efficiency * 0.1;
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::App;
    use bevy_app::Update;
    use crate::layer1::map::GridPosition;

    #[test]
    fn test_world_eater_converts_tiles_to_void() {
        let mut app = App::new();

        let size = 10 * 10;
        let mut grid = TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Grass; size],
        };
        grid.set(5, 5, TerrainType::Rock); // Test with Rock which yields stone and ore
        app.insert_resource(grid);
        app.insert_resource(ColonyResources::zeroed());
        app.add_event::<FleetCommand>();
        app.add_systems(Update, process_world_eater_system);

        let eater = app.world_mut().spawn(WorldEater { efficiency: 100.0 }).id();

        app.world_mut().resource_mut::<Events<FleetCommand>>().send(FleetCommand::ConsumeTile {
            fleet: eater,
            target: GridPosition { x: 5, y: 5 },
        });

        app.update();

        let grid = app.world().resource::<TerrainGrid>();
        assert_eq!(grid.get(5, 5).unwrap(), TerrainType::Void, "World Eater should convert the target tile to Void.");

        let resources = app.world().resource::<ColonyResources>();
        println!("Stone: {}", resources.stone);
        println!("Ore: {}", resources.ore);
        assert!((resources.stone - 80.0).abs() < f32::EPSILON, "World Eater should generate resources based on efficiency and tile type.");
        assert!((resources.ore - 20.0).abs() < f32::EPSILON, "World Eater should generate resources based on efficiency and tile type.");
    }
}

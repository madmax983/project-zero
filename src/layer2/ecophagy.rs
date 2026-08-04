use bevy_ecs::prelude::*;
use crate::layer2::fleet::{WorldEater, FleetCommand};
use crate::layer1::nature::terrain::{TerrainGrid, TerrainType};

#[derive(Resource)]
pub struct RawResources { pub amount: u32 }

pub fn process_world_eater_system(
    mut events: EventReader<FleetCommand>,
    mut grid: ResMut<TerrainGrid>,
    mut resources: ResMut<RawResources>,
    query: Query<&WorldEater>,
) {
    for event in events.read() {
        let FleetCommand::ConsumeTile { fleet, target } = event;
        if let Ok(eater) = query.get(*fleet) {
            if let Some(tile) = grid.get(target.x as usize, target.y as usize) {
                if tile != TerrainType::Void {
                    grid.set(target.x as usize, target.y as usize, TerrainType::Void);
                    resources.amount += eater.efficiency;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::map::GridPosition;

    #[test]
    fn test_world_eater_converts_tiles_to_bedrock() {
        let mut app = bevy_ecs::world::World::new();
        let mut grid = TerrainGrid { width: 10, height: 10, tiles: vec![TerrainType::Grass; 100] };
        grid.set(5, 5, TerrainType::Rock);
        app.insert_resource(grid);
        app.insert_resource(RawResources { amount: 0 });

        app.insert_resource(Events::<FleetCommand>::default());

        let eater = app.spawn(WorldEater { efficiency: 100 }).id();

        app.resource_mut::<Events<FleetCommand>>().send(FleetCommand::ConsumeTile {
            fleet: eater,
            target: GridPosition { x: 5, y: 5 },
        });

        let mut schedule = bevy_ecs::schedule::Schedule::default();
        schedule.add_systems(process_world_eater_system);
        schedule.run(&mut app);

        let grid = app.resource::<TerrainGrid>();
        assert_eq!(grid.get(5, 5).unwrap(), TerrainType::Void, "World Eater should convert the target tile to Bedrock(Void).");

        let resources = app.resource::<RawResources>();
        assert_eq!(resources.amount, 100, "World Eater should generate resources based on efficiency.");
    }
}

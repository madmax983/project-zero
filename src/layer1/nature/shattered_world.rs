use crate::layer1::terrain::{TerrainGrid, TerrainType};
use bevy_ecs::prelude::*;

pub struct ShatteredWorldConfig {
    pub void_percentage: f32,
}

impl Default for ShatteredWorldConfig {
    fn default() -> Self {
        Self {
            void_percentage: 0.3,
        }
    }
}

#[derive(Event, Debug)]
pub struct PlaceBridgeCommand {
    pub x: usize,
    pub y: usize,
}

pub fn generate_shattered_grid(
    width: usize,
    height: usize,
    config: &ShatteredWorldConfig,
) -> TerrainGrid {
    let mut grid = TerrainGrid {
        width,
        height,
        tiles: vec![TerrainType::Rock; width * height],
    };

    let total_tiles = width * height;
    let target_void = (total_tiles as f32 * config.void_percentage) as usize;

    let mut void_placed = 0;
    for y in 0..height {
        for x in 0..width {
            if void_placed < target_void {
                grid.set(x, y, TerrainType::Void);
                void_placed += 1;
            }
        }
    }

    grid
}

pub fn place_bridge_system(
    mut commands: EventReader<PlaceBridgeCommand>,
    mut grid: Option<ResMut<TerrainGrid>>,
) {
    if let Some(g) = grid.as_deref_mut() {
        for cmd in commands.read() {
            if g.get(cmd.x, cmd.y) == Some(TerrainType::Void) {
                g.set(cmd.x, cmd.y, TerrainType::Bridge);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::building::{BuildingMap, OccupiedTiles};
    use crate::layer1::pathfinding::find_path;

    #[test]
    fn test_map_generator_creates_void_chasms() {
        let config = ShatteredWorldConfig {
            void_percentage: 0.3,
        };
        let grid = generate_shattered_grid(100, 100, &config);

        let void_count = grid
            .tiles
            .iter()
            .filter(|&t| *t == TerrainType::Void)
            .count();
        assert!(void_count > 1000);
    }

    #[test]
    fn test_bridges_can_be_built_over_void() {
        let mut app = bevy_app::App::new();
        app.add_event::<PlaceBridgeCommand>();

        let mut tiles = vec![TerrainType::Rock; 100];
        tiles[55] = TerrainType::Void; // (5, 5) is void
        app.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });
        app.add_systems(bevy_app::Update, place_bridge_system);

        app.world_mut()
            .send_event(PlaceBridgeCommand { x: 5, y: 5 });
        app.update();

        let grid = app.world().resource::<TerrainGrid>();
        assert_eq!(grid.get(5, 5).unwrap(), TerrainType::Bridge);
    }

    #[test]
    fn test_pathfinding_treats_void_as_impassable() {
        let mut world = World::new();
        let mut tiles = vec![TerrainType::Grass; 25];

        for y in 0..5 {
            tiles[y * 5 + 2] = TerrainType::Void;
        }

        world.insert_resource(TerrainGrid {
            width: 5,
            height: 5,
            tiles,
        });
        world.insert_resource(OccupiedTiles::default());
        world.insert_resource(BuildingMap::default());

        let path = find_path(&world, (0, 0), (4, 0));
        assert!(path.is_none());

        let mut grid = world.resource_mut::<TerrainGrid>();
        grid.set(2, 0, TerrainType::Bridge);

        let path_with_bridge = find_path(&world, (0, 0), (4, 0));
        assert!(path_with_bridge.is_some());
    }
}

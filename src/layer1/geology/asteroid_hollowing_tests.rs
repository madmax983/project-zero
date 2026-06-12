#[cfg(test)]
mod tests {
    use bevy_app::App;
    use bevy_app::Update;
    use crate::layer1::nature::terrain::{TerrainGrid, TerrainType};
    use crate::layer1::map::GridPosition;
    use crate::layer1::geology::asteroid_hollowing::{HullIntegrity, MineEvent, process_hull_integrity};

    #[test]
    fn test_mining_near_edge_weakens_hull() {
        let mut app = App::new();
        app.add_systems(Update, process_hull_integrity);
        app.add_event::<MineEvent>();

        // Create an asteroid grid
        let width = 20;
        let height = 20;
        let mut grid = TerrainGrid { width, height, tiles: vec![TerrainType::Rock; width * height] };
        grid.set(19, 10, TerrainType::Rock); // Edge wall
        grid.set(10, 10, TerrainType::Rock); // Inner wall (safe)
        app.world_mut().insert_resource(grid);
        app.world_mut().insert_resource(HullIntegrity { integrity: 100.0 });

        // Mine the inner wall (safely)
        app.world_mut().send_event(MineEvent { pos: GridPosition { x: 10, y: 10 } });
        app.update();

        // Integrity is fine
        assert_eq!(app.world().resource::<HullIntegrity>().integrity, 100.0);

        // Mine the edge wall (danger)
        app.world_mut().send_event(MineEvent { pos: GridPosition { x: 19, y: 10 } });
        app.update();

        // Integrity drops
        assert!(app.world().resource::<HullIntegrity>().integrity < 100.0);
    }
}

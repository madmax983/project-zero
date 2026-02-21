
#[cfg(test)]
mod tests {
    use scale::layer1::beauty::{BeautyGrid, BeautySource, update_beauty_grid_system, apply_beauty_effects_system};
    use scale::layer1::building::{Building, BuildingType, MaterialType, spawn_building_with_material};
    use scale::layer1::map::GridPosition;
    use scale::layer1::needs::Needs;
    use scale::layer1::pop::Pop;
    use scale::layer1::terrain::{TerrainGrid, TerrainType};
    use bevy_ecs::prelude::*;

    #[test]
    fn test_statue_beauty_propagates_to_adjacent_tiles() {
        // Use standard setup to ensure all resources exist
        let mut world = scale::setup::setup_world();

        // Reset terrain to flat grass for predictability (80x50 is default size in setup.rs)
        let mut terrain = world.resource_mut::<TerrainGrid>();
        terrain.tiles.fill(TerrainType::Grass);

        // Reset Beauty Grid
        let mut bg = world.resource_mut::<BeautyGrid>();
        bg.clear();

        // Spawn a Statue at (5, 5). Statues are obstacles.
        spawn_building_with_material(
            &mut world,
            5,
            5,
            BuildingType::Statue,
            MaterialType::Stone,
        );

        // Verify Statue exists and has BeautySource
        let mut query = world.query_filtered::<Entity, With<Building>>();
        // Note: setup_world spawns initial structures. We should find OUR statue.
        // Or better, filter by position.
        let statue = query.iter(&world)
            .find(|e| {
                world.get::<GridPosition>(*e).is_some_and(|p| p.x == 5 && p.y == 5)
            })
            .expect("Statue not found");

        let source = world.get::<BeautySource>(statue).unwrap();
        assert!(source.value > 0.0, "Statue should have positive beauty");

        // Spawn a Pop adjacent at (5, 6)
        let pop = world.spawn((
            Pop,
            GridPosition { x: 5, y: 6 },
            Needs {
                leisure: 0.5,
                ..Default::default()
            },
        )).id();

        // Run systems manually in order
        let mut schedule = Schedule::default();
        schedule.add_systems((
            update_beauty_grid_system,
            apply_beauty_effects_system,
        ).chain());

        schedule.run(&mut world);

        // Check if Pop's leisure increased
        let needs = world.get::<Needs>(pop).unwrap();

        // Grass Beauty = 1.0. Effect = 1.0 * 0.001 = 0.001. New Leisure = 0.501.
        // If Statue works (Radius), let's say it adds 10.0. Total 11.0. Effect 0.011. Leisure 0.511.

        assert!(needs.leisure > 0.502, "Pop should benefit from adjacent Statue beauty (Current: {})", needs.leisure);
    }
}

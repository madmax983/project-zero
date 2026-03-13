#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use scale::layer1::atmosphere::{update_atmosphere_system, AtmosphereGrid};
    use scale::layer1::building::{Building, BuildingType};
    use scale::layer1::map::GridPosition;
    use scale::layer1::terrain::{TerrainGrid, TerrainType};
    use scale::layer1::wind::{update_wind_system, GlobalWind, Vec2, WindGrid};

    #[test]
    fn pollution_moves_downwind() {
        let mut world = World::new();
        let width = 10;
        let height = 10;

        // Setup grids
        world.insert_resource(AtmosphereGrid::new(width, height));
        world.insert_resource(WindGrid::new(width, height));
        world.insert_resource(GlobalWind {
            direction: Vec2::new(1.0, 0.0), // East wind
            speed: 1.0,                     // Moderate wind
        });
        world.insert_resource(TerrainGrid {
            width,
            height,
            tiles: vec![TerrainType::Grass; width * height],
        });

        // Spawn pollution source at (2, 2)
        // Smelter emits 0.05
        world.spawn((
            Building {
                building_type: BuildingType::Smelter,
                /* ..Default::default() removed by razor */
            },
            GridPosition { x: 2, y: 2 },
        ));

        // Create a schedule to simulate
        // We run wind once to set up the vector field
        // Then run atmosphere to simulate emission and diffusion (and eventually advection)
        let mut schedule = Schedule::default();

        // Note: update_wind_system reads TerrainGrid and BuildingQuery to set blockers.
        // It reads GlobalWind to set base wind.
        schedule.add_systems(update_wind_system);

        // Atmosphere system handles emission from buildings and diffusion.
        schedule.add_systems(update_atmosphere_system.after(update_wind_system));

        // Run for 10 ticks to allow pollution to spread
        for _ in 0..10 {
            schedule.run(&mut world);
        }

        let atmosphere = world.resource::<AtmosphereGrid>();

        // Check Upwind (1, 2) vs Downwind (4, 2) vs Crosswind (2, 1)
        // Since source is at (2, 2), downwind is +X (East).
        // Let's check a bit further away to see the plume effect.
        let center = atmosphere.get(2, 2);
        let upwind = atmosphere.get(1, 2); // West
        let downwind = atmosphere.get(3, 2); // East (Immediate downwind)
        let far_downwind = atmosphere.get(4, 2); // Further East

        println!(
            "Center: {}, Upwind: {}, Downwind: {}, Far Downwind: {}",
            center, upwind, downwind, far_downwind
        );

        // Current behavior (Symmetric diffusion):
        // Upwind ≈ Downwind
        // With Advection:
        // Downwind >> Upwind

        // We assert that downwind concentration is significantly higher than upwind.
        // With strong wind (5.0), advection should dominate diffusion.
        assert!(
            downwind > upwind * 1.5,
            "Pollution should flow downwind significantly more than upwind. Got Down: {}, Up: {}",
            downwind,
            upwind
        );
    }
}

use crate::layer1::map::{TilePos, ZLevel};
use crate::layer1::nature::atmosphere::SmogGrid;
use bevy::prelude::*;

#[derive(Component)]
pub struct HeavyIndustry {
    pub active: bool,
    pub z_level: ZLevel,
}

#[derive(Component)]
pub struct MovementSpeed {
    pub current: f32,
}

pub fn process_subterranean_smog_system(
    mut grid: ResMut<SmogGrid>,
    query: Query<(&HeavyIndustry, &TilePos)>,
) {
    for (industry, pos) in query.iter() {
        if industry.active {
            // Simplification: Smog "sinks" to an arbitrary lowest level (ZLevel -2) directly beneath it.
            // In a real implementation, this would involve fluid dynamics and checking for open tiles below.
            let target_z = ZLevel(-2);
            if industry.z_level > target_z {
                // Arbitrary position for test

                let current_smog = grid.get_smog(*pos, target_z);
                grid.set_smog(*pos, target_z, current_smog + 10.0); // Emit smog downwards
            }
        }
    }
}

pub fn apply_smog_penalties_system(
    grid: Res<SmogGrid>,
    mut query: Query<(&TilePos, &ZLevel, &mut MovementSpeed)>,
) {
    for (pos, z, mut speed) in query.iter_mut() {
        let smog_level = grid.get_smog(*pos, *z);
        if smog_level > 50.0 {
            speed.current = 0.5; // 50% speed penalty
        } else {
            speed.current = 1.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::map::{TilePos, ZLevel};
    use crate::layer1::nature::atmosphere::SmogGrid;
    use crate::layer1::pop::Pop;

    #[test]
    fn test_underground_industry_accumulates_smog_on_lowest_z_level() {
        let mut app = App::new();
        app.add_systems(Update, process_subterranean_smog_system);

        let mut grid = SmogGrid::default();
        grid.set_smog(TilePos::new(10, 10), ZLevel(-2), 0.0); // -2 is deepest
        app.world_mut().insert_resource(grid);

        // Spawn a factory on ZLevel -1
        app.world_mut().spawn((
            HeavyIndustry {
                active: true,
                z_level: ZLevel(-1),
            },
            TilePos::new(10, 10),
        ));

        app.update();

        let updated_grid = app.world().resource::<SmogGrid>();
        assert!(
            updated_grid.get_smog(TilePos::new(10, 10), ZLevel(-2)) > 0.0,
            "Smog should sink to the lowest Z-Level"
        );
    }

    #[test]
    fn test_deep_smog_reduces_movement_speed() {
        let mut app = App::new();
        app.add_systems(Update, apply_smog_penalties_system);

        let mut grid = SmogGrid::default();
        grid.set_smog(TilePos::new(10, 10), ZLevel(-2), 100.0); // Thick smog
        app.world_mut().insert_resource(grid);

        let pop = app
            .world_mut()
            .spawn((
                Pop,
                TilePos::new(10, 10),
                ZLevel(-2),
                MovementSpeed { current: 1.0 },
            ))
            .id();

        app.update();

        let speed = app.world().get::<MovementSpeed>(pop).unwrap();
        assert!(
            speed.current < 1.0,
            "Thick deep smog should reduce movement speed"
        );
    }
}

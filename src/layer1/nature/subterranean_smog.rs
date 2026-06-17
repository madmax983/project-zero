use crate::layer1::core::map::GridPosition;
use crate::layer1::entities::pop::Speed;
use crate::layer1::nature::atmosphere::SmogGrid;
use bevy::prelude::*;

#[derive(Component)]
pub struct HeavyIndustry {
    pub active: bool,
}

pub fn process_subterranean_smog_system(
    mut grid: ResMut<SmogGrid>,
    query: Query<(&HeavyIndustry, &GridPosition)>,
) {
    for (industry, pos) in query.iter() {
        if industry.active {
            let current_smog = grid.get_smog(*pos);
            grid.set_smog(*pos, current_smog + 10.0); // Emit smog downwards
        }
    }
}

pub fn apply_smog_penalties_system(
    grid: Res<SmogGrid>,
    mut query: Query<(&GridPosition, &mut Speed)>,
) {
    for (pos, mut speed) in query.iter_mut() {
        let smog_level = grid.get_smog(*pos);
        if smog_level > 50.0 {
            speed.current = speed.base * 0.5; // 50% speed penalty
        } else {
            speed.current = speed.base;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::core::map::GridPosition;
    use crate::layer1::entities::pop::Speed;
    use crate::layer1::nature::atmosphere::SmogGrid;
    use crate::layer1::pop::Pop;

    #[test]
    fn test_underground_industry_accumulates_smog_on_lowest_z_level() {
        let mut app = App::new();
        app.add_systems(Update, process_subterranean_smog_system);

        let mut grid = SmogGrid::default();
        grid.set_smog(GridPosition { x: 10, y: 10 }, 0.0);
        app.world_mut().insert_resource(grid);

        // Spawn a factory
        app.world_mut().spawn((
            HeavyIndustry { active: true },
            GridPosition { x: 10, y: 10 },
        ));

        app.update();

        let updated_grid = app.world().resource::<SmogGrid>();
        assert!(
            updated_grid.get_smog(GridPosition { x: 10, y: 10 }) > 0.0,
            "Smog should be emitted at the industry's position"
        );
    }

    #[test]
    fn test_deep_smog_reduces_movement_speed() {
        let mut app = App::new();
        app.add_systems(Update, apply_smog_penalties_system);

        let mut grid = SmogGrid::default();
        grid.set_smog(GridPosition { x: 10, y: 10 }, 100.0); // Thick smog
        app.world_mut().insert_resource(grid);

        let pop = app
            .world_mut()
            .spawn((
                Pop,
                GridPosition { x: 10, y: 10 },
                Speed {
                    base: 1.0,
                    accumulator: 0.0,
                    current: 1.0,
                },
            ))
            .id();

        app.update();

        let speed = app.world().get::<Speed>(pop).unwrap();
        assert!(
            speed.current < 1.0,
            "Thick deep smog should reduce movement speed"
        );
    }
}

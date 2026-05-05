use crate::layer1::architecture::structure::Structure;
use crate::layer1::building::Building;
use crate::layer1::map::GridPosition;
use crate::layer1::nature::temperature::TemperatureGrid;
use bevy_ecs::prelude::*;

#[derive(Resource)]
pub struct TerminatorLine {
    pub x_coordinate: f32,
}

#[derive(Resource)]
pub struct LibrationCycle {
    pub current_tick: f32,
    pub amplitude: f32,
    pub speed: f32,
}

#[derive(Component)]
pub struct MaxTemperatureAllowed {
    pub degrees: f32,
}

pub fn calculate_tile_temperatures_system(
    terminator: Option<Res<TerminatorLine>>,
    grid: Option<ResMut<TemperatureGrid>>,
) {
    if let (Some(t_line), Some(mut grid)) = (terminator, grid) {
        let width = grid.width;
        let height = grid.height;
        for y in 0..height {
            for x in 0..width {
                let distance_from_terminator = (x as f32) - t_line.x_coordinate;
                let temp = if distance_from_terminator < -5.0 {
                    150.0
                } else if distance_from_terminator > 5.0 {
                    -150.0
                } else {
                    25.0 - (distance_from_terminator * 10.0)
                };
                grid.set(x, y, temp);
            }
        }
    }
}

pub fn apply_libration_wobble_system(
    cycle: Option<ResMut<LibrationCycle>>,
    terminator: Option<ResMut<TerminatorLine>>,

) {
    if let (Some(mut cycle), Some(mut terminator)) = (cycle, terminator) {
        cycle.current_tick += cycle.speed;
        terminator.x_coordinate = 50.0 + (cycle.current_tick.sin() * cycle.amplitude);

    }

}

pub fn building_temperature_damage_system(
    grid: Option<Res<TemperatureGrid>>,
    mut building_query: Query<
        (&mut Structure, &MaxTemperatureAllowed, &GridPosition),
        With<Building>,
    >,
) {
    if let Some(grid) = grid {
        for (mut structure, max_temp, pos) in building_query.iter_mut() {
            let tile_temp = grid.get(pos.x as usize, pos.y as usize);
            if tile_temp > max_temp.degrees {
                structure.current_hp -= 5.0;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::building::BuildingType;
    use bevy::prelude::*;

    #[test]
    fn test_terminator_line_determines_habitable_temperature() {
        let mut app = App::new();
        app.add_systems(Update, calculate_tile_temperatures_system);

        app.world_mut()
            .insert_resource(TerminatorLine { x_coordinate: 50.0 });
        let grid = TemperatureGrid::new(100, 100, 0.0);
        app.world_mut().insert_resource(grid);

        app.update();

        let grid = app.world().resource::<TemperatureGrid>();
        assert!(grid.get(20, 10) > 100.0, "Day side should be boiling");
        assert!(grid.get(80, 10) < -100.0, "Night side should be freezing");

        let term_temp = grid.get(50, 10);
        assert!(
            term_temp > 10.0 && term_temp < 40.0,
            "Terminator should be habitable"
        );
    }

    #[test]
    fn test_libration_shifts_terminator_line() {
        let mut app = App::new();
        app.insert_resource(TerminatorLine { x_coordinate: 50.0 })
            .insert_resource(LibrationCycle {
                current_tick: 0.0,
                amplitude: 5.0,
                speed: 0.1,
            })
            .add_systems(Update, apply_libration_wobble_system);

        app.update();

        let new_x = app.world().resource::<TerminatorLine>().x_coordinate;
        assert_ne!(new_x, 50.0, "Libration should shift the terminator line");
    }

    #[test]
    fn test_buildings_melt_when_terminator_shifts_away() {
        let mut app = App::new();
        app.add_systems(Update, building_temperature_damage_system);

        let mut grid = TemperatureGrid::new(100, 100, 0.0);
        grid.set(45, 10, 150.0);
        app.world_mut().insert_resource(grid);

        let building = app
            .world_mut()
            .spawn((
                Building {
                    building_type: BuildingType::Wall,
                },
                Structure {
                    current_hp: 100.0,
                    max_hp: 100.0,
                },
                MaxTemperatureAllowed { degrees: 80.0 },
                GridPosition { x: 45, y: 10 },
            ))
            .id();

        app.update();

        assert!(
            app.world().get::<Structure>(building).unwrap().current_hp < 100.0,
            "Building should melt when too hot"
        );
    }
}

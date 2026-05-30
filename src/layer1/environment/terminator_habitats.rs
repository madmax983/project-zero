use bevy::prelude::*;

#[derive(Resource)]
pub struct TerminatorLine {
    pub x_coordinate: f32,
}

#[derive(Component)]
pub struct Tile {
    pub x: f32,
    pub y: f32,
}

#[derive(Component)]
pub struct Temperature {
    pub degrees: f32,
}

pub fn calculate_tile_temperatures_system(
    terminator: Option<Res<TerminatorLine>>,
    mut tile_query: Query<(&Tile, &mut Temperature)>,
) {
    if let Some(t_line) = terminator {
        for (tile, mut temp) in tile_query.iter_mut() {
            let distance_from_terminator = tile.x - t_line.x_coordinate;

            if distance_from_terminator < -5.0 {
                // Deep Day side
                temp.degrees = 150.0;
            } else if distance_from_terminator > 5.0 {
                // Deep Night side
                temp.degrees = -150.0;
            } else {
                // Terminator zone (-5 to 5) gradient.
                // Exactly on line (0) is 25 degrees.
                // -5 is ~75 degrees, +5 is ~-25 degrees.
                temp.degrees = 25.0 - (distance_from_terminator * 10.0);
            }
        }
    }
}

#[derive(Resource)]
pub struct LibrationCycle {
    pub current_tick: f32,
    pub amplitude: f32,
    pub speed: f32,
}

pub fn apply_libration_wobble_system(
    mut cycle: ResMut<LibrationCycle>,
    mut terminator: ResMut<TerminatorLine>,
) {
    cycle.current_tick += cycle.speed;
    // Base line is at 50, oscillates by amplitude
    terminator.x_coordinate = 50.0 + (cycle.current_tick.sin() * cycle.amplitude);
}

#[derive(Component)]
pub struct Building;

#[derive(Component)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

#[derive(Component)]
pub struct MaxTemperatureAllowed {
    pub degrees: f32,
}

#[derive(Component)]
pub struct LocatedOn(pub Entity);

pub fn building_temperature_damage_system(
    tile_query: Query<&Temperature, With<Tile>>,
    mut building_query: Query<(&mut Health, &MaxTemperatureAllowed, &LocatedOn), With<Building>>,
) {
    for (mut health, max_temp, location) in building_query.iter_mut() {
        if let Ok(tile_temp) = tile_query.get(location.0) {
            if tile_temp.degrees > max_temp.degrees {
                health.current -= 5.0; // Melt damage
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminator_line_determines_habitable_temperature() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
           .add_systems(Update, calculate_tile_temperatures_system);

        // Arrange: A planet where the terminator line is currently at X=50
        app.world_mut().insert_resource(TerminatorLine { x_coordinate: 50.0 });

        // Day side (X < 50)
        let day_tile = app.world_mut().spawn((Tile { x: 20.0, y: 10.0 }, Temperature { degrees: 0.0 })).id();
        // Night side (X > 50)
        let night_tile = app.world_mut().spawn((Tile { x: 80.0, y: 10.0 }, Temperature { degrees: 0.0 })).id();
        // Terminator (X = 50)
        let term_tile = app.world_mut().spawn((Tile { x: 50.0, y: 10.0 }, Temperature { degrees: 0.0 })).id();

        app.update();

        // Assert
        assert!(app.world().get::<Temperature>(day_tile).unwrap().degrees > 100.0, "Day side should be boiling");
        assert!(app.world().get::<Temperature>(night_tile).unwrap().degrees < -100.0, "Night side should be freezing");

        let term_temp = app.world().get::<Temperature>(term_tile).unwrap().degrees;
        assert!(term_temp > 10.0 && term_temp < 40.0, "Terminator should be habitable");
    }

    #[test]
    fn test_libration_shifts_terminator_line() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
           .insert_resource(TerminatorLine { x_coordinate: 50.0 })
           .insert_resource(LibrationCycle { current_tick: 0.0, amplitude: 5.0, speed: 0.1 })
           .add_systems(Update, apply_libration_wobble_system);

        app.update(); // Tick 1

        // Assert: The terminator line has moved slightly from 50.0
        let new_x = app.world().resource::<TerminatorLine>().x_coordinate;
        assert_ne!(new_x, 50.0, "Libration should shift the terminator line");
    }

    #[test]
    fn test_buildings_melt_when_terminator_shifts_away() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
           .add_systems(Update, building_temperature_damage_system);

        // Arrange: A building on a tile that just became Day Side (very hot)
        let tile = app.world_mut().spawn((Tile { x: 45.0, y: 10.0 }, Temperature { degrees: 150.0 })).id();
        let building = app.world_mut().spawn((
            Building,
            Health { current: 100.0, max: 100.0 },
            MaxTemperatureAllowed { degrees: 80.0 },
            LocatedOn(tile),
        )).id();

        app.update();

        // Assert: Building takes damage due to overheating
        assert!(app.world().get::<Health>(building).unwrap().current < 100.0, "Building should melt when too hot");
    }
}

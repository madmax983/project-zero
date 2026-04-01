import sys
import re
import os
import fnmatch

# Reset to clean state
os.system("git checkout src/")

with open('src/layer1/pressure.rs', 'r') as f:
    content = f.read()

# I am going to inject the RED phase tests without modifying `Needs` or any other file.
# The tests will fail because `process_door_venting_system`, `apply_door_movement_penalties_system`, and `process_suffocation_system` don't exist yet!
# So I will just provide empty stub functions for them above `mod tests` so that it compiles and FAILS the tests!

stubs = """
use crate::layer1::control::{DoorControl, DoorState};
use crate::layer1::pop::{Pop, Speed};
use crate::layer1::needs::Needs;
use crate::layer1::building::{Building, BuildingType};

pub fn process_door_venting_system() {}
pub fn apply_door_movement_penalties_system() {}
pub fn process_suffocation_system() {}

"""

test_code = """
    use crate::layer1::control::{DoorControl, DoorState};
    use crate::layer1::pop::Speed;
    use crate::layer1::needs::Needs;
    use bevy_ecs::prelude::*;

    #[test]
    fn test_door_vents_atmosphere_on_open() {
        let mut app = bevy_app::App::new();
        app.add_plugins(bevy_app::MinimalPlugins);

        let door_entity = app.world_mut().spawn((
            Building { building_type: BuildingType::Gate },
            DoorControl { state: DoorState::Open },
            GridPosition { x: 10, y: 10 },
            crate::layer1::structure::Structure::default(),
        )).id();

        let mut grid = PressureGrid::new(20, 20);
        grid.set(9, 10, 1.0);
        grid.set(11, 10, 0.0);
        app.world_mut().insert_resource(grid);

        app.add_systems(bevy_app::Update, super::process_door_venting_system);
        app.update();

        let grid = app.world().resource::<PressureGrid>();
        assert!(grid.get(9, 10) < 1.0, "Interior pressure should drop");
        assert!(grid.get(11, 10) > 0.0, "Exterior pressure should rise");
    }

    #[test]
    fn test_airlock_minimizes_venting() {
        let mut app = bevy_app::App::new();
        app.add_plugins(bevy_app::MinimalPlugins);

        let airlock_entity = app.world_mut().spawn((
            Building { building_type: BuildingType::Airlock },
            DoorControl { state: DoorState::Open },
            GridPosition { x: 10, y: 10 },
            crate::layer1::structure::Structure::default(),
        )).id();

        let mut grid = PressureGrid::new(20, 20);
        grid.set(9, 10, 1.0);
        grid.set(11, 10, 0.0);
        app.world_mut().insert_resource(grid);

        app.add_systems(bevy_app::Update, super::process_door_venting_system);
        app.update();

        let grid = app.world().resource::<PressureGrid>();
        assert!(grid.get(9, 10) > 0.95, "Airlock should preserve most interior pressure");
        assert!(grid.get(11, 10) < 0.05, "Airlock should leak minimal pressure");
    }

    #[test]
    fn test_airlock_slows_movement() {
        let mut app = bevy_app::App::new();
        app.add_plugins(bevy_app::MinimalPlugins);

        app.world_mut().spawn((
            Building { building_type: BuildingType::Airlock },
            GridPosition { x: 10, y: 10 },
            crate::layer1::structure::Structure::default(),
        ));

        let pop_entity = app.world_mut().spawn((
            Pop,
            GridPosition { x: 10, y: 10 },
            Speed { base: 1.0, current: 1.0, accumulator: 0.0 },
        )).id();

        app.add_systems(bevy_app::Update, super::apply_door_movement_penalties_system);
        app.update();

        let stats = app.world().get::<Speed>(pop_entity).unwrap();
        assert!(stats.current < stats.base, "Airlock should slow movement");
    }

    #[test]
    fn test_unsealed_room_suffocates_pops() {
        let mut app = bevy_app::App::new();
        app.add_plugins(bevy_app::MinimalPlugins);

        let mut grid = PressureGrid::new(20, 20);
        grid.set(5, 5, 0.0);
        app.world_mut().insert_resource(grid);

        let pop_entity = app.world_mut().spawn((
            Pop,
            GridPosition { x: 5, y: 5 },
            Health { current: 100.0, max: 100.0 },
        )).id();

        let mut schedule = bevy_ecs::schedule::Schedule::default();
        schedule.add_systems(super::process_suffocation_system);
        schedule.run(app.world_mut());

        let health = app.world().get::<Health>(pop_entity).unwrap();
        assert!(health.current < 100.0, "Pop should take suffocation damage in vacuum");
    }
"""

# Find `mod tests {` and insert our stubs before it, and tests inside it.
new_content = content.replace("mod tests {", stubs + "mod tests {\n" + test_code)
with open('src/layer1/pressure.rs', 'w') as f:
    f.write(new_content)

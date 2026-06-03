with open("src/layer1/graffiti.rs", "r") as f:
    content = f.read()

import re

# Remove the test from inside another test block
# It starts with `    #[test]` and ends with `    }\n` near the end of `fn test_graffiti_observation_affects_mood`
# Wait, I appended the whole thing manually last time.
content = content.replace("""
    #[test]
    fn test_creative_pop_creates_graffiti_under_high_stress() {
        use crate::layer1::psychology::stress::StressTracker;
        use crate::layer1::traits::{Trait, Traits};
        let mut world = World::new();
        world.insert_resource(GraffitiMap::default());

        let mut occupied = OccupiedTiles::default();
        occupied.0.insert((5, 5));
        world.insert_resource(occupied);

        world.spawn((
            Building {
                building_type: BuildingType::Wall,
            },
            GridPosition { x: 5, y: 5 },
        ));

        let mut traits = Traits::default();
        traits.add(Trait::Creative);

        world.spawn((
            Pop,
            GridPosition { x: 5, y: 4 },
            Morale {
                value: 0.5,
                ..Default::default()
            },
            StressTracker { accumulated_stress: 90.0 },
            traits,
        ));

        let mut schedule = Schedule::default();
        schedule.add_systems(graffiti_placement_system);

        let mut placed = false;
        for _ in 0..1000 {
            schedule.run(&mut world);
            if world
                .resource::<GraffitiMap>()
                .markings
                .contains_key(&(5, 5))
            {
                placed = true;
                break;
            }
        }

        assert!(placed, "Propaganda should be placed eventually");

        let map = world.resource::<GraffitiMap>();
        let graffiti = map.markings.get(&(5, 5));
        assert!(graffiti.is_some());
        assert_eq!(graffiti.unwrap().graffiti_type, GraffitiType::Propaganda);
    }
""", "")

content = content.replace("""
#[cfg(test)]
mod additional_tests {
    use super::*;
    use crate::layer1::building::{Building, BuildingType, OccupiedTiles};
    use crate::layer1::map::GridPosition;
    use crate::layer1::morale::Morale;
    use crate::layer1::pop::Pop;
    use crate::layer1::psychology::stress::StressTracker;
    use crate::layer1::traits::{Trait, Traits};

    #[test]
    fn test_creative_pop_creates_graffiti_under_high_stress() {
        let mut world = World::new();
        world.insert_resource(GraffitiMap::default());

        let mut occupied = OccupiedTiles::default();
        occupied.0.insert((5, 5));
        world.insert_resource(occupied);

        world.spawn((
            Building {
                building_type: BuildingType::Wall,
            },
            GridPosition { x: 5, y: 5 },
        ));

        let mut traits = Traits::default();
        traits.add(Trait::Creative);

        world.spawn((
            Pop,
            GridPosition { x: 5, y: 4 },
            Morale {
                value: 0.5,
                ..Default::default()
            },
            StressTracker { accumulated_stress: 90.0 },
            traits,
        ));

        let mut schedule = Schedule::default();
        schedule.add_systems(graffiti_placement_system);

        let mut placed = false;
        for _ in 0..1000 {
            schedule.run(&mut world);
            if world
                .resource::<GraffitiMap>()
                .markings
                .contains_key(&(5, 5))
            {
                placed = true;
                break;
            }
        }

        assert!(placed, "Propaganda should be placed eventually");

        let map = world.resource::<GraffitiMap>();
        let graffiti = map.markings.get(&(5, 5));
        assert!(graffiti.is_some());
        assert_eq!(graffiti.unwrap().graffiti_type, GraffitiType::Propaganda);
    }
}
""", "")

with open("src/layer1/graffiti.rs", "w") as f:
    f.write(content)

with open("src/layer1/graffiti.rs", "a") as f:
    f.write("""
#[cfg(test)]
mod additional_tests {
    use super::*;
    use crate::layer1::building::{Building, BuildingType, OccupiedTiles};
    use crate::layer1::map::GridPosition;
    use crate::layer1::morale::Morale;
    use crate::layer1::pop::Pop;
    use crate::layer1::psychology::stress::StressTracker;
    use crate::layer1::traits::{Trait, Traits};

    #[test]
    fn test_creative_pop_creates_graffiti_under_high_stress() {
        let mut world = World::new();
        world.insert_resource(GraffitiMap::default());

        let mut occupied = OccupiedTiles::default();
        occupied.0.insert((5, 5));
        world.insert_resource(occupied);

        world.spawn((
            Building {
                building_type: BuildingType::Wall,
            },
            GridPosition { x: 5, y: 5 },
        ));

        let mut traits = Traits::default();
        traits.add(Trait::Creative);

        world.spawn((
            Pop,
            GridPosition { x: 5, y: 4 },
            Morale {
                value: 0.5,
                ..Default::default()
            },
            StressTracker { accumulated_stress: 90.0 },
            traits,
        ));

        let mut schedule = Schedule::default();
        schedule.add_systems(graffiti_placement_system);

        let mut placed = false;
        for _ in 0..1000 {
            schedule.run(&mut world);
            if world
                .resource::<GraffitiMap>()
                .markings
                .contains_key(&(5, 5))
            {
                placed = true;
                break;
            }
        }

        assert!(placed, "Propaganda should be placed eventually");

        let map = world.resource::<GraffitiMap>();
        let graffiti = map.markings.get(&(5, 5));
        assert!(graffiti.is_some());
        assert_eq!(graffiti.unwrap().graffiti_type, GraffitiType::Propaganda);
    }
}
""")

with open("src/layer1/graffiti.rs", "r") as f:
    lines = f.readlines()

new_lines = []
for line in lines:
    if "fn test_creative_pop_creates_graffiti_under_high_stress() {" in line:
        new_lines.pop() # remove #[test] that was appended last time, but wait, the inner items warning means I put #[test] inside another function
        pass

with open("src/layer1/graffiti.rs", "r") as f:
    content = f.read()

# Ah, I see I appended it INSIDE another test function!
# Let's fix that.
import re
content = re.sub(r"    #\[test\]\n    fn test_creative_pop_creates_graffiti_under_high_stress\(\) \{.*?\n        assert_eq!\(graffiti\.unwrap\(\)\.graffiti_type, GraffitiType::Propaganda\);\n    \}\n", "", content, flags=re.DOTALL)

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

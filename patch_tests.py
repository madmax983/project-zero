import sys

with open('src/layer2/syzygy.rs', 'r') as f:
    content = f.read()

tests = """
#[cfg(test)]
mod tests {
    use super::*;

    use crate::layer2::alignment::PlanetaryAlignment;
    use crate::layer1::map::GridPosition;
    use crate::layer1::pop::Pop;
    use crate::layer1::chronicle::{Chronicle, EventImportance};

    fn setup_test_world() -> World {
        let mut world = World::new();
        world.insert_resource(Chronicle::default());
        world
    }

    #[test]
    fn test_syzygy_trigger() {
        let mut world = setup_test_world();
        let _alignment = world.spawn(PlanetaryAlignment { days_until: 0 }).id();

        trigger_syzygy(&mut world);

        assert!(world.contains_resource::<SyzygyActive>());
        assert!(world.resource::<Chronicle>().events.iter().any(|e| e.text == "syzygy_begins"));
    }

    #[test]
    fn test_syzygy_hauling_boost() {
        let mut world = setup_test_world();
        world.insert_resource(SyzygyActive { duration: 10 });
        let hauler = world.spawn((crate::layer1::entities::pop::Speed { base: 1.0, current: 1.0, accumulator: 0.0 }, crate::layer1::utility_types::PopAction { current: crate::layer1::utility_types::ActionType::Haul, ..Default::default() })).id();

        apply_syzygy_effects(&mut world);

        let boosted_speed = world.get::<crate::layer1::entities::pop::Speed>(hauler).unwrap().current;
        assert!(boosted_speed > 1.0);
    }

    #[test]
    fn test_syzygy_mutation_chance() {
        let mut world = setup_test_world();
        world.insert_resource(SyzygyActive { duration: 10 });
        let pop = world.spawn((Pop, GridPosition { x: 5, y: 5 })).id();

        // Trigger it many times to ensure the 10% chance procs at least once in tests
        for _ in 0..100 {
            trigger_random_mutation(&mut world);
        }

        assert!(world.get::<MutatedTrait>(pop).is_some());
    }
    use crate::shared::time::SimulationTime;
"""

test_idx = content.find("#[cfg(test)]\nmod tests {\n    use super::*;")
if test_idx != -1:
    idx2 = content.find("use crate::shared::time::SimulationTime;", test_idx)
    content = content[:test_idx] + tests + content[idx2 + len("use crate::shared::time::SimulationTime;"):]
else:
    print("Could not find tests module")

with open('src/layer2/syzygy.rs', 'w') as f:
    f.write(content)

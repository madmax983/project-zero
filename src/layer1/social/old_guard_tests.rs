#[cfg(test)]
mod tests {
    use crate::layer1::chronicle::{AddChronicleEvent, EventImportance};
    use crate::layer1::pop::Pop;
    use crate::layer1::social::old_guard::{
        Arrival, Demographics, FOUNDER_CUTOFF_YEAR, FounderBuff, Generation, MoodModifiers,
        apply_founder_benefits_system, check_generational_friction_system,
    };
    use bevy_ecs::prelude::*;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_arrival_component_defaults() {
        let arrival = Arrival { tick: 100 };
        assert_eq!(arrival.tick, 100);
    }

    #[test]
    fn test_generation_determination() {
        // Founders arrive before Year 5 (5000 ticks)
        let early = Arrival { tick: 100 };
        assert_eq!(early.generation(), Generation::Founder);

        let late = Arrival {
            tick: FOUNDER_CUTOFF_YEAR * crate::layer1::balance::TICKS_PER_YEAR + 1000,
        };
        assert_eq!(late.generation(), Generation::Immigrant);
    }

    #[test]
    fn test_apply_founder_benefits() {
        let mut world = World::new();

        // Spawn a Founder
        let founder = world
            .spawn((
                Pop,
                Arrival { tick: 100 }, // Year 0
            ))
            .id();

        // Spawn an Immigrant
        let immigrant = world
            .spawn((
                Pop,
                Arrival { tick: 6000 }, // Year 6
            ))
            .id();

        // Run system
        let _ = world.run_system_once(apply_founder_benefits_system);

        assert!(world.get::<FounderBuff>(founder).is_some());
        assert!(world.get::<FounderBuff>(immigrant).is_none());

        // Check for mood modifier
        let modifiers = world.get::<MoodModifiers>(founder);
        assert!(modifiers.is_some());
        assert!(
            modifiers
                .unwrap()
                .entries
                .iter()
                .any(|m| m.value == 5.0 && m.source == "Legacy of the First")
        );
    }

    #[test]
    fn test_friction_founders_overwhelmed() {
        let mut world = World::new();
        world.init_resource::<Demographics>(); // Initialize resource to avoid panic if system expects it
        world.init_resource::<Events<AddChronicleEvent>>();

        // 1 Founder
        world.spawn((Pop, Arrival { tick: 100 }, Generation::Founder));

        // 3 Immigrants (Ratio 3:1 > 2:1)
        world.spawn((Pop, Arrival { tick: 6000 }, Generation::Immigrant));
        world.spawn((Pop, Arrival { tick: 6000 }, Generation::Immigrant));
        world.spawn((Pop, Arrival { tick: 6000 }, Generation::Immigrant));

        let _ = world.run_system_once(check_generational_friction_system);

        // Founder should have "Overwhelmed" mood modifier
        let mut query = world.query::<&MoodModifiers>();
        let overwhelmed_count = query
            .iter(&world)
            .flat_map(|m| m.entries.iter())
            .filter(|entry| entry.source == "Overwhelmed by Strangers")
            .count();

        assert_eq!(overwhelmed_count, 1);
    }

    #[test]
    fn test_friction_immigrants_excluded() {
        let mut world = World::new();
        world.init_resource::<Demographics>();
        world.init_resource::<Events<AddChronicleEvent>>();

        // 3 Founders
        world.spawn((Pop, Arrival { tick: 100 }, Generation::Founder));
        world.spawn((Pop, Arrival { tick: 100 }, Generation::Founder));
        world.spawn((Pop, Arrival { tick: 100 }, Generation::Founder));

        // 1 Immigrant
        world.spawn((Pop, Arrival { tick: 6000 }, Generation::Immigrant));

        let _ = world.run_system_once(check_generational_friction_system);

        // Immigrant should have "Excluded" mood modifier
        let mut query = world.query::<&MoodModifiers>();
        let excluded_count = query
            .iter(&world)
            .flat_map(|m| m.entries.iter())
            .filter(|entry| entry.source == "Excluded by Clique")
            .count();

        assert_eq!(excluded_count, 1);
    }

    #[test]
    fn test_demographics_tracking() {
        let mut world = World::new();
        world.insert_resource(Demographics::default());
        world.init_resource::<Events<AddChronicleEvent>>();

        // 2 Founders, 1 Immigrant
        world.spawn((Pop, Generation::Founder));
        world.spawn((Pop, Generation::Founder));
        world.spawn((Pop, Generation::Immigrant));

        let _ = world.run_system_once(check_generational_friction_system);

        let demographics = world.resource::<Demographics>();
        assert_eq!(demographics.founders, 2);
        assert_eq!(demographics.immigrants, 1);
    }

    #[test]
    fn test_chronicle_event_turning_point() {
        let mut world = World::new();
        world.insert_resource(Demographics::default());
        world.init_resource::<Events<AddChronicleEvent>>();

        // Initial state: Majority Founders (3 vs 1)
        world.resource_mut::<Demographics>().founders = 3;
        world.resource_mut::<Demographics>().immigrants = 1;

        // Spawn entities to match demographics for the system query
        world.spawn((Pop, Generation::Founder));
        world.spawn((Pop, Generation::Founder));
        world.spawn((Pop, Generation::Founder));
        world.spawn((Pop, Generation::Immigrant));

        // Run system - should NOT trigger event yet
        let _ = world.run_system_once(check_generational_friction_system);

        {
            let events = world.resource::<Events<AddChronicleEvent>>();
            let mut reader = events.get_cursor();
            assert_eq!(reader.read(events).count(), 0);
        }

        // Add more Immigrants to flip majority (3 vs 4)
        world.spawn((Pop, Generation::Immigrant));
        world.spawn((Pop, Generation::Immigrant));
        world.spawn((Pop, Generation::Immigrant));

        // Run system - SHOULD trigger event
        let _ = world.run_system_once(check_generational_friction_system);

        {
            let events = world.resource::<Events<AddChronicleEvent>>();
            let mut reader = events.get_cursor();
            let events: Vec<_> = reader.read(events).collect();
            assert_eq!(events.len(), 1);
            assert_eq!(events[0].importance, EventImportance::Major);
            assert!(events[0].text.contains("Turning Point"));
        }

        // Run again - SHOULD NOT trigger event (only once)
        let _ = world.run_system_once(check_generational_friction_system);

        {
            // Let's just check the resource flag.
            let demo = world.resource::<Demographics>();
            assert!(demo.has_triggered_turning_point);

            // Check total count of events in the resource (should still be 1)
            let events_res = world.resource::<Events<AddChronicleEvent>>();
            assert_eq!(events_res.len(), 1);
        }
    }
}

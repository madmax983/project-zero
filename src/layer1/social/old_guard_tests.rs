#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::social::old_guard::{
        Arrival, Generation, FounderBuff, MoodModifiers, check_generational_friction_system,
        apply_founder_benefits_system, FOUNDER_CUTOFF_YEAR
    };
    use crate::layer1::balance::TICKS_PER_YEAR;
    use crate::layer1::pop::Pop;
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

        let late = Arrival { tick: FOUNDER_CUTOFF_YEAR * TICKS_PER_YEAR + 1000 };
        assert_eq!(late.generation(), Generation::Immigrant);
    }

    #[test]
    fn test_apply_founder_benefits() {
        let mut world = World::new();

        // Spawn a Founder
        let founder = world.spawn((
            Pop,
            Arrival { tick: 100 }, // Year 0
        )).id();

        // Spawn an Immigrant
        let immigrant = world.spawn((
            Pop,
            Arrival { tick: 6000 }, // Year 6
        )).id();

        // Run system
        let _ = world.run_system_once(apply_founder_benefits_system);

        assert!(world.get::<FounderBuff>(founder).is_some());
        assert!(world.get::<FounderBuff>(immigrant).is_none());

        // Check for mood modifier
        let modifiers = world.get::<MoodModifiers>(founder);
        assert!(modifiers.is_some());
        assert!(modifiers.unwrap().entries.iter().any(|m| m.value == 5.0 && m.source == "Legacy of the First"));
    }

    #[test]
    fn test_friction_founders_overwhelmed() {
        let mut world = World::new();

        // 1 Founder
        world.spawn((Pop, Arrival { tick: 100 }, Generation::Founder));

        // 3 Immigrants (Ratio 3:1 > 2:1)
        world.spawn((Pop, Arrival { tick: 6000 }, Generation::Immigrant));
        world.spawn((Pop, Arrival { tick: 6000 }, Generation::Immigrant));
        world.spawn((Pop, Arrival { tick: 6000 }, Generation::Immigrant));

        let _ = world.run_system_once(check_generational_friction_system);

        // Founder should have "Overwhelmed" mood modifier
        let mut query = world.query::<&MoodModifiers>();
        let overwhelmed_count = query.iter(&world)
            .flat_map(|m| m.entries.iter())
            .filter(|entry| entry.source == "Overwhelmed by Strangers")
            .count();

        assert_eq!(overwhelmed_count, 1);
    }

    #[test]
    fn test_friction_immigrants_excluded() {
        let mut world = World::new();

        // 3 Founders
        world.spawn((Pop, Arrival { tick: 100 }, Generation::Founder));
        world.spawn((Pop, Arrival { tick: 100 }, Generation::Founder));
        world.spawn((Pop, Arrival { tick: 100 }, Generation::Founder));

        // 1 Immigrant
        world.spawn((Pop, Arrival { tick: 6000 }, Generation::Immigrant));

        let _ = world.run_system_once(check_generational_friction_system);

        // Immigrant should have "Excluded" mood modifier
        let mut query = world.query::<&MoodModifiers>();
        let excluded_count = query.iter(&world)
            .flat_map(|m| m.entries.iter())
            .filter(|entry| entry.source == "Excluded by Clique")
            .count();

        assert_eq!(excluded_count, 1);
    }
}

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use scale::shared::attention::{AttentionFocus, DataResolution, query_hunger, query_rest, IsFocused, sync_is_focused_system};
    use scale::layer1::psychology::needs::Needs;

    #[test]
    fn test_focus_limit_enforced() {
        let mut focus = AttentionFocus::new(2);
        let e1 = Entity::from_raw(1);
        let e2 = Entity::from_raw(2);
        let e3 = Entity::from_raw(3);

        focus.focus_on(e1);
        focus.focus_on(e2);
        assert_eq!(focus.focused_entities.len(), 2);
        assert!(focus.is_focused(e1));
        assert!(focus.is_focused(e2));

        // Adding e3 should evict e1
        focus.focus_on(e3);
        assert_eq!(focus.focused_entities.len(), 2);
        assert!(!focus.is_focused(e1));
        assert!(focus.is_focused(e2));
        assert!(focus.is_focused(e3));
    }

    #[test]
    fn test_focused_entity_returns_precise_stats() {
        let mut focus = AttentionFocus::new(5);
        let entity = Entity::from_raw(1);
        focus.focus_on(entity);

        let needs = Needs { hunger: 0.425, rest: 0.5, leisure: 1.0, hygiene: 1.0 };

        let result_hunger = query_hunger(entity, &needs, &focus);
        match result_hunger {
            DataResolution::Precise(val) => assert_eq!(val, 0.425),
            _ => panic!("Expected Precise data"),
        }

        let result_rest = query_rest(entity, &needs, &focus);
        match result_rest {
            DataResolution::Precise(val) => assert_eq!(val, 0.5),
            _ => panic!("Expected Precise data"),
        }
    }

    #[test]
    fn test_unfocused_entity_returns_fuzzy_stats() {
        let focus = AttentionFocus::new(5);
        let entity = Entity::from_raw(2); // Not focused

        let needs = Needs { hunger: 0.425, rest: 0.5, leisure: 1.0, hygiene: 1.0 };

        let result_hunger = query_hunger(entity, &needs, &focus);
        match result_hunger {
            DataResolution::Fuzzy(s) => assert_eq!(s, "Moderate"),
            _ => panic!("Expected Fuzzy data"),
        }

        let needs_high = Needs { hunger: 0.9, rest: 0.9, leisure: 1.0, hygiene: 1.0 };
        match query_hunger(entity, &needs_high, &focus) {
            DataResolution::Fuzzy(s) => assert_eq!(s, "High"),
            _ => panic!("Expected Fuzzy data"),
        }
        match query_rest(entity, &needs_high, &focus) {
            DataResolution::Fuzzy(s) => assert_eq!(s, "Rested"),
            _ => panic!("Expected Fuzzy data"),
        }

        let needs_low = Needs { hunger: 0.1, rest: 0.1, leisure: 1.0, hygiene: 1.0 };
        match query_hunger(entity, &needs_low, &focus) {
            DataResolution::Fuzzy(s) => assert_eq!(s, "Low"),
            _ => panic!("Expected Fuzzy data"),
        }
        match query_rest(entity, &needs_low, &focus) {
            DataResolution::Fuzzy(s) => assert_eq!(s, "Exhausted"),
            _ => panic!("Expected Fuzzy data"),
        }

        let result_rest = query_rest(entity, &needs, &focus);
        match result_rest {
            DataResolution::Fuzzy(s) => assert_eq!(s, "Tired"),
            _ => panic!("Expected Fuzzy data"),
        }
    }

    #[test]
    fn test_sync_is_focused() {
        let mut world = World::new();
        let mut focus = AttentionFocus::new(2);

        let e1 = world.spawn_empty().id();
        let e2 = world.spawn_empty().id();

        focus.focus_on(e1);
        world.insert_resource(focus);

        let mut schedule = Schedule::default();
        schedule.add_systems(sync_is_focused_system);
        schedule.run(&mut world);

        assert!(world.get::<IsFocused>(e1).is_some());
        assert!(world.get::<IsFocused>(e2).is_none());

        world.resource_mut::<AttentionFocus>().focus_on(e2);
        schedule.run(&mut world);

        assert!(world.get::<IsFocused>(e1).is_some());
        assert!(world.get::<IsFocused>(e2).is_some());

        world.resource_mut::<AttentionFocus>().focused_entities.pop_front(); // Remove e1
        schedule.run(&mut world);

        assert!(world.get::<IsFocused>(e1).is_none());
        assert!(world.get::<IsFocused>(e2).is_some());
    }

    #[test]
    fn test_cultural_drift_while_unfocused() {
        use scale::layer1::pop::Pop;
        use scale::layer1::utility_types::UtilityWeights;
        use scale::shared::attention::unobserved_drift_system;

        let mut world = World::new();

        let entity_c = world.spawn((
            Pop,
            UtilityWeights::default(),
        )).id();

        let _initial_weight = world.get::<UtilityWeights>(entity_c).unwrap().availability_weight;

        let mut schedule = Schedule::default();
        schedule.add_systems(unobserved_drift_system);

        for _ in 0..100 {
            schedule.run(&mut world);
        }

        let final_weight = world.get::<UtilityWeights>(entity_c).unwrap().availability_weight;
        // Verify drift has likely occurred
        assert!(final_weight >= 0.0 && final_weight <= 2.0);
        assert_ne!(final_weight, _initial_weight, "Weights should drift when unobserved");
    }
}

#[cfg(test)]
mod tests {
    use crate::layer1::hazards::{
        AccidentSeverity, AmputationEvent, calculate_risk, determine_severity, trigger_accident,
    };
    use crate::layer1::map::GridPosition;
    use crate::layer1::pop::Pop;
    use crate::layer1::skills::{SkillType, Skills};
    use crate::layer1::structure::Structure;
    use crate::layer1::utility_types::ActionType;
    use bevy_ecs::prelude::*;

    #[test]
    fn test_risk_scaling_skill() {
        // Base risk for Work is 0.001 (0.1%)
        let base_risk = ActionType::Work.danger_level();

        // Level 0 skill
        let skills_0 = Skills::default();
        let risk_0 = calculate_risk(
            base_risk,
            &skills_0,
            SkillType::Construction,
            &Structure::default(),
            1.0,
        );

        // Level 10 skill (Efficiency 2.0)
        let mut skills_10 = Skills::default();
        skills_10.add_xp(SkillType::Construction, 10000.0); // Sufficient for high level
        let risk_10 = calculate_risk(
            base_risk,
            &skills_10,
            SkillType::Construction,
            &Structure::default(),
            1.0,
        );

        assert!(
            risk_10 < risk_0,
            "High skill should reduce risk. Risk 0: {}, Risk 10: {}",
            risk_0,
            risk_10
        );
        // Check reduction factor (approx half if skill factor is significant)
        assert!(risk_10 < risk_0 * 0.6);
    }

    #[test]
    fn test_risk_scaling_decay() {
        let base_risk = ActionType::Work.danger_level();
        let skills = Skills::default();

        let pristine = Structure {
            current_hp: 100.0,
            max_hp: 100.0,
            ..Default::default()
        };
        let risk_pristine =
            calculate_risk(base_risk, &skills, SkillType::Construction, &pristine, 1.0);

        let crumbling = Structure {
            current_hp: 10.0,
            max_hp: 100.0,
            ..Default::default()
        };
        let risk_crumbling =
            calculate_risk(base_risk, &skills, SkillType::Construction, &crumbling, 1.0);

        assert!(
            risk_crumbling > risk_pristine,
            "Decay should increase risk. Pristine: {}, Crumbling: {}",
            risk_pristine,
            risk_crumbling
        );
        // Should be at least double (decay factor ~2.0)
        assert!(risk_crumbling > risk_pristine * 2.0);
    }

    #[test]
    fn test_severity_distribution() {
        // Deterministic check or statistical
        // For unit test, we can check the logic of the helper function directly
        // assuming we pass a seed or random value.
        // Let's assume determine_severity takes a float 0.0-1.0

        assert_eq!(determine_severity(0.5), AccidentSeverity::Minor); // 0.0 - 0.8
        assert_eq!(determine_severity(0.85), AccidentSeverity::Major); // 0.8 - 0.95
        assert_eq!(determine_severity(0.96), AccidentSeverity::Critical); // 0.95 - 1.0
    }

    #[test]
    fn test_amputation_event_generation() {
        let mut world = World::new();
        world.insert_resource(Events::<AmputationEvent>::default());
        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 0, y: 0 },
                crate::layer1::health::Health::default(),
            ))
            .id();

        // Trigger critical accident logic
        trigger_accident(&mut world, pop, AccidentSeverity::Critical);

        // Check for AmputationEvent
        let events = world.resource::<Events<AmputationEvent>>();
        let mut reader = events.get_cursor();
        let event = reader.read(events).next();

        assert!(event.is_some(), "AmputationEvent should be fired");
        assert_eq!(event.unwrap().entity, pop);
    }
}

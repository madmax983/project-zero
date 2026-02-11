#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use bevy_ecs::system::RunSystemOnce;
    use scale::layer1::factions::{FactionId, FactionMember, Factions};
    use scale::layer1::integration::apply_faction_mood_system;
    use scale::layer1::needs::Needs;
    use scale::layer1::pop::Pop;

    fn setup_world() -> World {
        let mut world = World::new();
        // Initialize Factions resource
        let mut factions = Factions::default();
        factions.initialize();
        world.insert_resource(factions);
        world
    }

    #[test]
    fn test_low_satisfaction_penalty() {
        let mut world = setup_world();

        // 1. Set MinersGuild satisfaction to Low (0.1)
        {
            let mut factions = world.resource_mut::<Factions>();
            let miners = factions.map.get_mut(&FactionId::MinersGuild).unwrap();
            miners.satisfaction = 0.1;
        }

        // 2. Spawn Pop in MinersGuild
        let pop = world
            .spawn((
                Pop,
                Needs {
                    leisure: 0.5,
                    ..Default::default()
                },
                FactionMember {
                    faction_id: Some(FactionId::MinersGuild),
                },
            ))
            .id();

        // 3. Run System
        world.run_system_once(apply_faction_mood_system).unwrap();

        // 4. Verify Leisure Decayed
        // Expected: 0.5 - 0.002 = 0.498
        let needs = world.get::<Needs>(pop).unwrap();
        assert!(
            needs.leisure < 0.5,
            "Leisure should decay due to low satisfaction"
        );
        assert!(
            (needs.leisure - 0.498).abs() < f32::EPSILON,
            "Expected leisure to be 0.498, got {}",
            needs.leisure
        );
    }

    #[test]
    fn test_high_satisfaction_bonus() {
        let mut world = setup_world();

        // 1. Set MinersGuild satisfaction to High (0.9)
        {
            let mut factions = world.resource_mut::<Factions>();
            let miners = factions.map.get_mut(&FactionId::MinersGuild).unwrap();
            miners.satisfaction = 0.9;
        }

        // 2. Spawn Pop in MinersGuild
        let pop = world
            .spawn((
                Pop,
                Needs {
                    leisure: 0.5,
                    ..Default::default()
                },
                FactionMember {
                    faction_id: Some(FactionId::MinersGuild),
                },
            ))
            .id();

        // 3. Run System
        world.run_system_once(apply_faction_mood_system).unwrap();

        // 4. Verify Leisure Increased
        // Expected: 0.5 + 0.001 = 0.501
        let needs = world.get::<Needs>(pop).unwrap();
        assert!(
            needs.leisure > 0.5,
            "Leisure should increase due to high satisfaction"
        );
        assert!(
            (needs.leisure - 0.501).abs() < f32::EPSILON,
            "Expected leisure to be 0.501, got {}",
            needs.leisure
        );
    }

    #[test]
    fn test_neutral_satisfaction_no_change() {
        let mut world = setup_world();

        // 1. Set MinersGuild satisfaction to Neutral (0.5)
        {
            let mut factions = world.resource_mut::<Factions>();
            let miners = factions.map.get_mut(&FactionId::MinersGuild).unwrap();
            miners.satisfaction = 0.5;
        }

        // 2. Spawn Pop in MinersGuild
        let pop = world
            .spawn((
                Pop,
                Needs {
                    leisure: 0.5,
                    ..Default::default()
                },
                FactionMember {
                    faction_id: Some(FactionId::MinersGuild),
                },
            ))
            .id();

        // 3. Run System
        world.run_system_once(apply_faction_mood_system).unwrap();

        // 4. Verify No Change
        let needs = world.get::<Needs>(pop).unwrap();
        assert!(
            (needs.leisure - 0.5).abs() < f32::EPSILON,
            "Leisure should not change for neutral satisfaction"
        );
    }
}

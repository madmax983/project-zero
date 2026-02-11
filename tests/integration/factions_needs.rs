#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use scale::layer1::factions::{FactionId, FactionMember, Factions};
    use scale::layer1::integration::apply_faction_mood_system;
    use scale::layer1::needs::Needs;
    use scale::layer1::pop::Pop;

    fn setup_world() -> World {
        let mut world = World::new();
        // Insert Factions resource
        let mut factions = Factions::default();
        factions.initialize();
        world.insert_resource(factions);
        world
    }

    #[test]
    fn test_low_satisfaction_decays_leisure() {
        let mut world = setup_world();

        // Setup Faction with low satisfaction
        {
            let mut factions = world.resource_mut::<Factions>();
            let faction = factions.map.get_mut(&FactionId::MinersGuild).unwrap();
            faction.satisfaction = 0.1; // Low
        }

        // Spawn Pop in that faction
        let pop = world
            .spawn((
                Pop,
                FactionMember {
                    faction_id: Some(FactionId::MinersGuild),
                },
                Needs {
                    leisure: 0.5,
                    ..Default::default()
                },
            ))
            .id();

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(apply_faction_mood_system);
        schedule.run(&mut world);

        // Check Leisure
        let needs = world.get::<Needs>(pop).unwrap();
        assert!(needs.leisure < 0.5, "Leisure should decay due to low satisfaction");
    }

    #[test]
    fn test_high_satisfaction_boosts_leisure() {
        let mut world = setup_world();

        // Setup Faction with high satisfaction
        {
            let mut factions = world.resource_mut::<Factions>();
            let faction = factions.map.get_mut(&FactionId::MinersGuild).unwrap();
            faction.satisfaction = 0.9; // High
        }

        // Spawn Pop in that faction
        let pop = world
            .spawn((
                Pop,
                FactionMember {
                    faction_id: Some(FactionId::MinersGuild),
                },
                Needs {
                    leisure: 0.5,
                    ..Default::default()
                },
            ))
            .id();

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(apply_faction_mood_system);
        schedule.run(&mut world);

        // Check Leisure
        let needs = world.get::<Needs>(pop).unwrap();
        assert!(needs.leisure > 0.5, "Leisure should increase due to high satisfaction");
    }

    #[test]
    fn test_neutral_satisfaction_no_change() {
        let mut world = setup_world();

        // Setup Faction with neutral satisfaction
        {
            let mut factions = world.resource_mut::<Factions>();
            let faction = factions.map.get_mut(&FactionId::MinersGuild).unwrap();
            faction.satisfaction = 0.5; // Neutral
        }

        // Spawn Pop in that faction
        let pop = world
            .spawn((
                Pop,
                FactionMember {
                    faction_id: Some(FactionId::MinersGuild),
                },
                Needs {
                    leisure: 0.5,
                    ..Default::default()
                },
            ))
            .id();

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(apply_faction_mood_system);
        schedule.run(&mut world);

        // Check Leisure
        let needs = world.get::<Needs>(pop).unwrap();
        assert!((needs.leisure - 0.5).abs() < f32::EPSILON, "Leisure should not change");
    }

    #[test]
    fn test_no_faction_no_change() {
        let mut world = setup_world();

        // Spawn Pop with no faction
        let pop = world
            .spawn((
                Pop,
                FactionMember {
                    faction_id: None,
                },
                Needs {
                    leisure: 0.5,
                    ..Default::default()
                },
            ))
            .id();

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(apply_faction_mood_system);
        schedule.run(&mut world);

        // Check Leisure
        let needs = world.get::<Needs>(pop).unwrap();
        assert!((needs.leisure - 0.5).abs() < f32::EPSILON, "Leisure should not change");
    }
}

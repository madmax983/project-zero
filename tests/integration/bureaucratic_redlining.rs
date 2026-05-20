#[cfg(test)]
mod integration_tests {
    use bevy::prelude::*;
    use scale::layer1::administration::bureaucratic_redlining::*;
    use scale::layer1::administration::zone::{ZoneGrid, ZoneType};
    use scale::layer1::economy::Wallet;
    use scale::layer1::entities::pop::Pop;
    use scale::layer1::map::GridPosition;
    use scale::layer1::social::factions::{FactionId, FactionMember};

    #[test]
    fn test_redlining_economy_seam() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let mut zone_grid = ZoneGrid::new(10, 10);
        zone_grid.set(5, 5, ZoneType::Dezoned);
        app.insert_resource(zone_grid);

        let pop = app
            .world_mut()
            .spawn((
                Pop,
                Wallet { credits: 0.0 },
                FactionMember {
                    faction_id: Some(FactionId::MinersGuild),
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        // System 1: The builder's system runs and turns them stateless
        app.add_systems(Update, bureaucratic_redlining_system);
        app.update();

        let f = app.world().get::<FactionMember>(pop).unwrap();
        assert_eq!(f.faction_id, Some(FactionId::Stateless));

        // System 2: Seam - stateless pops should not receive wages (economy integration)
        scale::layer1::economy::pay_wage(app.world_mut(), pop, 10.0);

        let w = app.world().get::<Wallet>(pop).unwrap();
        assert_eq!(w.credits, 0.0, "Stateless pops should not receive wages");
    }

    #[test]
    fn test_squatter_combat_seam() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let mut zone_grid = ZoneGrid::new(10, 10);
        zone_grid.set(5, 5, ZoneType::Dezoned);
        zone_grid.set(6, 5, ZoneType::Bedroom);
        app.insert_resource(zone_grid);

        let pop = app
            .world_mut()
            .spawn((
                Pop,
                FactionMember {
                    faction_id: Some(FactionId::Stateless),
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        // System: Squatter raid system from builder
        app.add_systems(Update, stateless_squatter_raid_system);

        // Let's force it to succeed
        let mut drafted = false;
        for _ in 0..1000 {
            app.update();
            if app
                .world()
                .get::<scale::layer1::combat::Drafted>(pop)
                .is_some()
            {
                drafted = true;
                break;
            }
        }

        assert!(
            drafted,
            "Stateless pop should be drafted into combat system"
        );
    }
}

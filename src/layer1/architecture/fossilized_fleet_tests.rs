#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use crate::layer1::architecture::building::{Building, BuildingType};
    use crate::layer1::map::GridPosition;
    use crate::layer1::economy::inventory::Inventory;
    use crate::layer1::economy::resources::ColonyResources;
    use crate::layer1::architecture::fossilized_fleet::{FossilizedShip, fossilized_ship_decay_system};
    use bevy::time::Time;

    #[test]
    fn test_fossilized_ship_creation() {
        let mut app = App::new();

        let entity = app.world_mut().spawn((
            FossilizedShip {
                decay_rate: 1.0,
                maintenance_cost: 10,
                defense_bonus: 50,
                structural_integrity: 100.0,
            },
            GridPosition { x: 5, y: 5 },
            Building { building_type: BuildingType::Housing }, // Treat as housing for MVP
            Inventory::default(),
        )).id();

        let ship = app.world().get::<FossilizedShip>(entity).unwrap();
        assert_eq!(ship.defense_bonus, 50);
        assert_eq!(ship.maintenance_cost, 10);
    }

    #[test]
    fn test_fossilized_ship_decay() {
        let mut app = App::new();
        let mut time: Time<()> = Time::default();
        time.advance_by(std::time::Duration::from_secs(1));
        app.insert_resource(time);

        let resources = ColonyResources {
            scrap: 0.0, // Cannot afford maintenance
            ..Default::default()
        };
        app.insert_resource(resources);

        app.add_systems(Update, fossilized_ship_decay_system);

        let entity = app.world_mut().spawn((
            FossilizedShip {
                decay_rate: 1.0,
                maintenance_cost: 10,
                defense_bonus: 50,
                structural_integrity: 100.0,
            },
        )).id();

        app.update();

        let ship = app.world().get::<FossilizedShip>(entity).unwrap();
        assert!(ship.structural_integrity < 100.0);
    }

    #[test]
    fn test_fossilized_ship_maintenance() {
        let mut app = App::new();
        let mut time: Time<()> = Time::default();
        time.advance_by(std::time::Duration::from_secs(1));
        app.insert_resource(time);

        let resources = ColonyResources {
            scrap: 100.0, // Can afford maintenance
            ..Default::default()
        };
        app.insert_resource(resources);

        app.add_systems(Update, fossilized_ship_decay_system);

        let entity = app.world_mut().spawn((
            FossilizedShip {
                decay_rate: 1.0,
                maintenance_cost: 10,
                defense_bonus: 50,
                structural_integrity: 100.0,
            },
        )).id();

        app.update();

        let ship = app.world().get::<FossilizedShip>(entity).unwrap();
        assert_eq!(ship.structural_integrity, 100.0); // Does not decay
        let res = app.world().resource::<ColonyResources>();
        assert!(res.scrap < 100.0); // Used up resources for maintenance
    }
}

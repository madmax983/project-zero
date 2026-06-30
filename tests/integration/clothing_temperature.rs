
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use scale::layer1::clothing::clothing_wear_system;
    use scale::layer1::health::Health;
    use scale::layer1::items::{Clothing, ClothingType, Equipment, Item};
    use scale::layer1::map::GridPosition;
    use scale::layer1::pop::Pop;
    use scale::layer1::temperature::{thermal_damage_system, TemperatureGrid};

    #[test]
    fn test_clothing_temperature_integration() {
        let mut world = World::new();

        let mut grid = TemperatureGrid::new(10, 10, -50.0);
        grid.set(5, 5, -50.0);
        world.insert_resource(grid);

        let tunic = world
            .spawn((
                Item::default(),
                Clothing {
                    clothing_type: ClothingType::Tunic,
                    insulation: 2.0, // Grants +60 cold tolerance, making tolerance -50.0
                    durability: 0.05, // Will break after 1 wear tick
                    max_durability: 100.0,
                },
            ))
            .id();

        let pop = world
            .spawn((
                Pop,
                Health {
                    current: 100.0,
                    max: 100.0,
                    has_rust_lung: false,
                },
                Equipment {
                    body: Some(tunic),
                    ..Default::default()
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems((clothing_wear_system, apply_deferred, thermal_damage_system).chain());

        schedule.run(&mut world);

        let h1 = world.get::<Health>(pop).unwrap();
        // Since tunic broke and commands applied, pop should take damage
        assert!(h1.current < 100.0, "Pop should take damage after clothing breaks");
    }
}

#[cfg(test)]
mod tests {
    use crate::layer1::actions::evaluate_fetch_clothing;
    use crate::layer1::clothing::clothing_wear_system;
    use crate::layer1::health::Health;
    use crate::layer1::items::{Clothing, ClothingType, Equipment, Item};
    use crate::layer1::map::GridPosition;
    use crate::layer1::pop::Pop;
    use crate::layer1::resources::ColonyResources;
    use crate::layer1::temperature::{TemperatureGrid, thermal_damage_system};
    use crate::layer1::utility_eval_types::ScorableCandidate;
    use bevy_ecs::prelude::*;
    use bevy_ecs::system::RunSystemOnce;

    // 1. Equipment Slots
    #[test]
    fn test_equipment_has_body_slot() {
        let mut world = World::new();
        let entity = world.spawn((Pop, Equipment::default())).id();
        let eq = world.get::<Equipment>(entity).unwrap();

        // Assert new fields exist
        assert!(eq.body.is_none());
        assert!(eq.head.is_none());
    }

    // 2. Clothing Component
    #[test]
    fn test_clothing_component() {
        let mut world = World::new();
        let tunic = world
            .spawn((
                Item::default(),
                Clothing {
                    clothing_type: ClothingType::Tunic,
                    insulation: 1.0,
                    durability: 100.0,
                    max_durability: 100.0,
                },
            ))
            .id();

        let c = world.get::<Clothing>(tunic).unwrap();
        assert_eq!(c.insulation, 1.0);
    }

    // 3. Thermal Damage Logic (Refactored from Hypothermia)
    #[test]
    fn test_thermal_damage_checks_equipment() {
        let mut world = World::new();
        // Setup Grid: -20.0 C (Freezing)
        let mut grid = TemperatureGrid::new(10, 10, -20.0);
        // Explicitly set pos (5,5) to -20.0
        grid.set(5, 5, -20.0);
        world.insert_resource(grid);

        // Pop 1: Naked (Should take damage)
        // Base cold tolerance is 10.0. Temp is -20.0. 10.0 > -20.0 -> Cold!
        let pop1 = world
            .spawn((
                Pop,
                Health {
                    current: 100.0,
                    max: 100.0,
                },
                Equipment::default(), // No body
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        // Pop 2: Clothed (Should be safe)
        // Insulation 1.0 -> Tolerance = 10.0 - (1.0 * 30.0) = -20.0.
        // Temp -20.0 is NOT < -20.0 (it is equal). So strictly speaking safe?
        // Logic: if temp < cold_tolerance { damage }. -20 < -20 is false. Safe.
        let tunic = world
            .spawn(Clothing {
                clothing_type: ClothingType::Tunic,
                insulation: 1.0,
                durability: 100.0,
                max_durability: 100.0,
            })
            .id();

        let pop2 = world
            .spawn((
                Pop,
                Health {
                    current: 100.0,
                    max: 100.0,
                },
                Equipment {
                    body: Some(tunic),
                    ..Default::default()
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        // Run system
        world.run_system_once(thermal_damage_system).unwrap();

        let h1 = world.get::<Health>(pop1).unwrap();
        let h2 = world.get::<Health>(pop2).unwrap();

        assert!(h1.current < 100.0, "Naked pop should freeze");
        assert_eq!(h2.current, 100.0, "Clothed pop should be warm");
    }

    // 4. Wear Logic
    #[test]
    fn test_clothing_degrades_on_wearer() {
        let mut world = World::new();

        let tunic = world
            .spawn(Clothing {
                clothing_type: ClothingType::Tunic,
                insulation: 1.0,
                durability: 10.0,
                max_durability: 100.0,
            })
            .id();

        let _pop = world
            .spawn((
                Pop,
                Equipment {
                    body: Some(tunic),
                    ..Default::default()
                },
            ))
            .id();

        world.run_system_once(clothing_wear_system).unwrap();

        let c = world.get::<Clothing>(tunic).unwrap();
        assert!(c.durability < 10.0);
    }

    // 5. Breakage
    #[test]
    fn test_clothing_breaks() {
        let mut world = World::new();

        let tunic = world
            .spawn(Clothing {
                clothing_type: ClothingType::Tunic,
                insulation: 1.0,
                durability: 0.001, // Almost broken
                max_durability: 100.0,
            })
            .id();

        let pop = world
            .spawn((
                Pop,
                Equipment {
                    body: Some(tunic),
                    ..Default::default()
                },
            ))
            .id();

        world.run_system_once(clothing_wear_system).unwrap();

        // Entity should be despawned
        assert!(world.get_entity(tunic).is_err());

        // Slot should be None
        let eq = world.get::<Equipment>(pop).unwrap();
        assert!(eq.body.is_none());
    }

    // 6. Fetch Clothing (Naked)
    #[test]
    fn test_naked_pop_wants_clothing() {
        let pop_pos = GridPosition { x: 0, y: 0 };
        let equipment = Equipment::default(); // No body
        let resources = ColonyResources {
            clothing: 1.0,
            ..Default::default()
        };
        // Fake entity for stockpile
        let stockpile_entity = Entity::from_raw(1);
        let stockpiles = vec![ScorableCandidate::new(
            stockpile_entity,
            GridPosition { x: 5, y: 0 },
        )];

        let result = evaluate_fetch_clothing(pop_pos, 0.0, &resources, &stockpiles, None);
        assert!(result.is_some(), "Naked pop should want clothing");
    }

    // 7. Fetch Clothing (Upgrade when Freezing)
    #[test]
    fn test_freezing_pop_upgrades_clothing() {
        let pop_pos = GridPosition { x: 0, y: 0 };
        // Has Tunic (Insulation 1.0)
        let insulation = 1.0;

        let resources = ColonyResources {
            clothing: 1.0,
            ..Default::default()
        };
        // Fake entity for stockpile (Nearby to avoid distance penalty lowering score too much)
        let stockpile_entity = Entity::from_raw(1);
        let stockpiles = vec![ScorableCandidate::new(
            stockpile_entity,
            GridPosition { x: 0, y: 0 },
        )];

        // Grid at -50.0 C
        let mut grid = TemperatureGrid::new(10, 10, -50.0);
        grid.set(0, 0, -50.0);

        let result =
            evaluate_fetch_clothing(pop_pos, insulation, &resources, &stockpiles, Some(&grid));

        assert!(result.is_some(), "Freezing pop should want upgrade");
        let (utility, target) = result.unwrap();
        assert_eq!(target, stockpile_entity);
        assert!(
            utility > 0.9,
            "Urgency should be high (Utility: {})",
            utility
        );
    }

    // 8. Handle Fetch Clothing (Upgrade Logic)
    #[test]
    fn test_handle_fetch_clothing_upgrades_to_parka() {
        use crate::layer1::execution::arrival::handle_fetch_clothing;

        let mut world = World::new();
        let mut resources = ColonyResources {
            clothing: 1.0,
            ..Default::default()
        };

        // Pop with Tunic
        let tunic = world
            .spawn(Clothing {
                clothing_type: ClothingType::Tunic,
                insulation: 1.0,
                durability: 100.0,
                max_durability: 100.0,
            })
            .id();

        let pop = world
            .spawn((
                Pop,
                Equipment {
                    body: Some(tunic),
                    ..Default::default()
                },
            ))
            .id();

        world.insert_resource(resources);
        world.init_resource::<bevy_ecs::event::Events<crate::layer1::items::UnequipEvent>>();

        // Run as system to avoid borrow checker issues with Mut<Equipment> vs Commands
        world
            .run_system_once(
                move |mut commands: Commands,
                      mut res: ResMut<ColonyResources>,
                      mut query: Query<&mut Equipment>,
                      mut unequip_events: EventWriter<crate::layer1::items::UnequipEvent>| {
                    let mut equipment_opt = query.get_mut(pop).ok();
                    handle_fetch_clothing(&mut commands, &mut res, pop, &mut equipment_opt, &mut unequip_events);
                },
            )
            .unwrap();

        // Apply commands
        world.flush();

        // Check equipment
        let eq = world.get::<Equipment>(pop).unwrap();
        assert!(eq.body.is_some());
        assert_ne!(eq.body, Some(tunic), "Should have new item");

        let new_clothing = world.get::<Clothing>(eq.body.unwrap()).unwrap();
        assert_eq!(
            new_clothing.clothing_type,
            ClothingType::Parka,
            "Should upgrade to Parka"
        );
        assert_eq!(new_clothing.insulation, 2.0);
    }
}

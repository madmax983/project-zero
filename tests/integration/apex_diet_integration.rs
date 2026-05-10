#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use scale::layer1::balance::FOOD_PER_MEAL;
    use scale::layer1::core::integration::{
        apex_meat_distribution_system, apex_meat_harvest_bridge_system,
    };
    use scale::layer1::economy::apex_diet::{
        process_apex_meat_consumption, ApexMeatStores, ConsumeFoodEvent,
    };
    use scale::layer1::fauna::{Fauna, FaunaType};
    use scale::layer1::needs::Needs;
    use scale::layer1::pop::Pop;
    use scale::layer1::Dead;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_event::<ConsumeFoodEvent>();
        app.insert_resource(ApexMeatStores::default());
        app.add_systems(
            Update,
            (
                apex_meat_harvest_bridge_system,
                apex_meat_distribution_system,
                process_apex_meat_consumption,
            )
                .chain(),
        );
        app
    }

    #[test]
    fn test_megafauna_death_harvests_apex_meat() {
        let mut app = setup_app();

        // Spawn a dead wolf
        app.world_mut().spawn((
            Fauna {
                fauna_type: FaunaType::Wolf,
                ..Default::default()
            },
            Dead,
        ));

        // Also spawn a rat (shouldn't yield meat)
        app.world_mut().spawn((
            Fauna {
                fauna_type: FaunaType::SpaceRat,
                ..Default::default()
            },
            Dead,
        ));

        app.update();

        let stores = app.world().get_resource::<ApexMeatStores>().unwrap();
        assert_eq!(stores.amount, 10.0, "Wolf death should yield 10 apex meat");
    }

    #[test]
    fn test_hungry_pops_consume_apex_meat() {
        let mut app = setup_app();

        // Initialize stores with enough meat for 1 pop
        app.world_mut().resource_mut::<ApexMeatStores>().amount = FOOD_PER_MEAL;

        let pop = app
            .world_mut()
            .spawn((
                Pop,
                Needs {
                    hunger: 0.1,
                    rest: 1.0,
                    hygiene: 1.0,
                    leisure: 1.0,
                }, // Very hungry
                scale::layer1::economy::apex_diet::Morale { current: 50.0 },
                scale::layer1::economy::apex_diet::PhysicalStrength { value: 10.0 },
                scale::layer1::economy::apex_diet::ApexMutationProgress { level: 0.0 },
            ))
            .id();

        // The bridge system will read stores, feed the pop, and emit ConsumeFoodEvent,
        // which process_apex_meat_consumption will pick up in the same frame due to .chain()
        app.update();

        // Verify stores depleted
        let stores = app.world().get_resource::<ApexMeatStores>().unwrap();
        assert!(stores.amount < FOOD_PER_MEAL, "Stores should be depleted");

        // Verify hunger satisfied
        let needs = app.world().get::<Needs>(pop).unwrap();
        assert!(needs.hunger > 0.1, "Hunger should be restored");

        // Verify apex diet effects applied
        let morale = app
            .world()
            .get::<scale::layer1::economy::apex_diet::Morale>(pop)
            .unwrap();
        let strength = app
            .world()
            .get::<scale::layer1::economy::apex_diet::PhysicalStrength>(pop)
            .unwrap();
        assert!(morale.current > 50.0, "Morale should increase");
        assert!(strength.value > 10.0, "Strength should increase");
    }
}

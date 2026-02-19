#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::vermin::{VerminState, VerminTrait, vermin_growth_system};
    use crate::layer1::resources::{ColonyResources, ResourceItem, ResourceType};
    use crate::layer1::edicts::{ColonyPolicies, Policy};
    use crate::layer1::fire::Fire;
    use crate::layer1::map::GridPosition;
    use bevy_ecs::system::RunSystemOnce;

    fn setup_world() -> World {
        let mut world = World::new();
        world.insert_resource(VerminState::default());
        world.insert_resource(ColonyResources::default());
        world.insert_resource(ColonyPolicies::default());
        world
    }

    #[test]
    fn test_gain_toxic_trait_from_waste() {
        let mut world = setup_world();

        // Add lots of Waste
        {
            let mut res = world.resource_mut::<ColonyResources>();
            res.waste = 1000.0;
            res.food = 10.0; // Minimal food so waste dominates
        }

        // Run growth system multiple times to trigger evolution
        for _ in 0..100 {
            world.run_system_once(vermin_growth_system).unwrap();
        }

        let vermin = world.resource::<VerminState>();
        assert!(vermin.traits.contains(&VerminTrait::Toxic), "Vermin should become Toxic from waste");
    }

    #[test]
    fn test_gain_volatile_trait_from_fuel() {
        let mut world = setup_world();

        // Add lots of Fuel
        {
            let mut res = world.resource_mut::<ColonyResources>();
            res.fuel = 1000.0;
            res.food = 10.0;
        }

        for _ in 0..100 {
            world.run_system_once(vermin_growth_system).unwrap();
        }

        let vermin = world.resource::<VerminState>();
        assert!(vermin.traits.contains(&VerminTrait::Volatile), "Vermin should become Volatile from fuel");
    }

    #[test]
    fn test_toxic_vermin_resist_pest_control() {
        let mut world = setup_world();

        // Setup Toxic Vermin
        {
            let mut vermin = world.resource_mut::<VerminState>();
            vermin.severity = 50.0;
            vermin.traits.insert(VerminTrait::Toxic);
        }

        // Enable Pest Control
        {
            let mut policies = world.resource_mut::<ColonyPolicies>();
            policies.toggle(Policy::PestControl);
        }

        // Add Food to stimulate growth
        {
            let mut res = world.resource_mut::<ColonyResources>();
            res.food = 1000.0;
        }

        // Run system
        world.run_system_once(vermin_growth_system).unwrap();

        let vermin = world.resource::<VerminState>();
        // Normal pest control halves growth. Toxic should reduce effectiveness (e.g. only 25% reduction or no reduction).
        // If normal growth is X, with pest control it's 0.5*X.
        // With Toxic, maybe it's 0.8*X or 1.0*X.
        // We just assert it grew MORE than it would have with normal pest control.
        // Calculating expected:
        // Food impact = 1000 * 0.00005 = 0.05
        // Normal Pest Control = 0.025 growth.
        // Toxic Pest Control = > 0.025 growth.

        // Let's rely on specific numbers for the test or logic check
        let growth = vermin.severity - 50.0;
        assert!(growth > 0.026, "Toxic vermin should resist pest control (growth > 0.025)");
    }

    #[test]
    fn test_volatile_vermin_ignite_fuel() {
        let mut world = setup_world();

        // Setup Volatile Vermin
        {
            let mut vermin = world.resource_mut::<VerminState>();
            vermin.traits.insert(VerminTrait::Volatile);
            vermin.severity = 50.0;
        }

        // Spawn Fuel Item
        world.spawn((
            ResourceItem { resource_type: ResourceType::Fuel, amount: 10.0 },
            GridPosition { x: 5, y: 5 }
        ));

        // Run a new system: vermin_effect_system
        // This system handles the active effects of traits
        // Note: vermin_effect_system doesn't exist yet, but we need to reference it.
        // We will reference it from crate::layer1::vermin
        world.run_system_once(crate::layer1::vermin::vermin_effect_system).unwrap();

        // Check for Fire
        // Since it's probabilistic, we might need to mock RNG or force it.
        // For TDD, assume we can force probability or run enough times.
        // Let's loop.

        let mut ignited = false;
        for _ in 0..100 {
             world.run_system_once(crate::layer1::vermin::vermin_effect_system).unwrap();
             if world.query::<&Fire>().iter(&world).count() > 0 {
                 ignited = true;
                 break;
             }
        }

        assert!(ignited, "Volatile vermin should eventually ignite fuel");
    }
}

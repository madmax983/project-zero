#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use bevy_ecs::system::RunSystemOnce;
    use scale::layer1::integration::vermin_item_rot_system;
    use scale::layer1::resources::ColonyResources;
    use scale::layer1::spoilage::{spoilage_system, Perishable};
    use scale::layer1::vermin::VerminState;

    #[test]
    fn test_vermin_accelerates_rot() {
        let mut world = World::new();

        // Setup high vermin severity
        world.insert_resource(VerminState {
            severity: 50.0,
            ..Default::default()
        });
        // Required for spoilage_system
        world.insert_resource(ColonyResources::default());

        // Spawn perishable item
        let item = world
            .spawn(Perishable {
                current_ticks: 0,
                max_ticks: 1000,
            })
            .id();

        // Run systems
        // Spoilage system handles normal decay (+1)
        world.run_system_once(spoilage_system).unwrap();
        // Rot system handles extra decay (Logic pending)
        world.run_system_once(vermin_item_rot_system).unwrap();

        let p = world.get::<Perishable>(item).unwrap();

        // With Stub: +1 (spoilage) + 0 (stub) = 1.
        // With Logic: +1 (spoilage) + ~5 (rot) = ~6.
        assert!(
            p.current_ticks > 2,
            "Vermin should significantly accelerate rot. Current: {}",
            p.current_ticks
        );
    }
}

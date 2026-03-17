#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::geology::tectonic::{TectonicStress, update_stress_system, check_quake_system, MegaQuakeEvent};
    use crate::layer1::resources::MiningEvent;
    use crate::layer1::volatile::ExplosionEvent;

    #[test]
    fn test_mining_increases_stress() {
        let mut world = World::new();
        world.insert_resource(TectonicStress::default());
        world.init_resource::<Events<MiningEvent>>();
        world.init_resource::<Events<ExplosionEvent>>();

        let mut schedule = Schedule::default();
        schedule.add_systems(update_stress_system);

        // Send mining event
        world.send_event(MiningEvent { amount: 10.0 });

        schedule.run(&mut world);

        let stress = world.resource::<TectonicStress>();
        assert!(stress.current > 0.0);
    }

    #[test]
    fn test_stress_dissipation() {
        let mut world = World::new();
        world.insert_resource(TectonicStress { current: 50.0, dissipation_rate: 1.0, ..Default::default() });
        world.init_resource::<Events<MiningEvent>>();
        world.init_resource::<Events<ExplosionEvent>>();

        let mut schedule = Schedule::default();
        schedule.add_systems(update_stress_system);

        schedule.run(&mut world);

        let stress = world.resource::<TectonicStress>();
        assert_eq!(stress.current, 49.0);
    }

    #[test]
    fn test_mega_quake_trigger() {
        let mut world = World::new();
        world.insert_resource(TectonicStress { current: 100.0, threshold: 100.0, ..Default::default() });
        world.init_resource::<Events<MegaQuakeEvent>>();
        world.init_resource::<Events<crate::layer1::geology::GeologicalEvent>>();

        let mut schedule = Schedule::default();
        schedule.add_systems(check_quake_system);

        schedule.run(&mut world);

        let events = world.resource::<Events<MegaQuakeEvent>>();
        assert!(!events.is_empty());

        let geo_events = world.resource::<Events<crate::layer1::geology::GeologicalEvent>>();
        assert!(!geo_events.is_empty());

        let stress = world.resource::<TectonicStress>();
        assert_eq!(stress.current, 0.0); // Reset after quake
    }
}

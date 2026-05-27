#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer2::fleet::{Fleet, FleetOrder, InOrbit};
    use crate::layer2::mining::{FleetCargo, CargoStack};
    use crate::layer2::drop::{DropPod, orbital_drop_system, pod_impact_system};
    use crate::layer1::economy::resources::{ResourceItem, ResourceType};
    use crate::layer1::core::map::GridPosition;

    fn setup_world() -> World {
        World::new()
    }

    #[test]
    fn test_drop_order_removes_cargo() {
        let mut world = setup_world();
        let colony = world.spawn(GridPosition { x: 50, y: 50 }).id();

        let fleet = world.spawn((
            Fleet,
            InOrbit { parent: colony },
            FleetCargo {
                contents: vec![CargoStack {
                    resource_type: ResourceType::Ore,
                    amount: 100.0,
                }],
                capacity: 100.0,
            }
        )).id();

        // Issue Drop Order for 50 Ore
        world.entity_mut(fleet).insert(FleetOrder::Drop(ResourceType::Ore, 50.0));

        // Run Drop System
        let mut schedule = Schedule::default();
        schedule.add_systems(orbital_drop_system);
        schedule.run(&mut world);

        // Verify Cargo Reduced
        let cargo = world.get::<FleetCargo>(fleet).unwrap();
        assert_eq!(cargo.contents[0].amount, 50.0);

        // Verify Order Consumed
        assert!(world.get::<FleetOrder>(fleet).is_none());
    }

    #[test]
    fn test_drop_order_spawns_pod() {
        let mut world = setup_world();
        let colony = world.spawn(GridPosition { x: 50, y: 50 }).id();

        let fleet = world.spawn((
            Fleet,
            InOrbit { parent: colony },
            FleetCargo {
                contents: vec![CargoStack {
                    resource_type: ResourceType::Ore,
                    amount: 50.0,
                }],
                capacity: 100.0,
            }
        )).id();

        world.entity_mut(fleet).insert(FleetOrder::Drop(ResourceType::Ore, 50.0));

        let mut schedule = Schedule::default();
        schedule.add_systems(orbital_drop_system);
        schedule.run(&mut world);

        // Verify DropPod Spawned
        let pod_count = world.query::<&DropPod>().iter(&world).count();
        assert_eq!(pod_count, 1);

        let (_, pod) = world.query::<(Entity, &DropPod)>().single(&world);
        assert_eq!(pod.resource_type, ResourceType::Ore);
        assert_eq!(pod.amount, 50.0);
        assert!(pod.impact_timer > 0.0);
    }

    #[test]
    fn test_pod_impact_spawns_resources() {
        let mut world = setup_world();

        // Spawn a DropPod about to impact
        let pod = world.spawn(DropPod {
            resource_type: ResourceType::Ore,
            amount: 50.0,
            impact_timer: 0.0, // Ready to impact
            target_pos: (10, 10),
            accuracy: 1.0,
            landing_delay: 0.0,
        }).id();

        // Run Impact System
        let mut schedule = Schedule::default();
        schedule.add_systems(pod_impact_system);
        schedule.run(&mut world);

        // Verify Pod Despawned
        assert!(world.get::<DropPod>(pod).is_none());

        // Verify ResourceItem Spawned
        let item_count = world.query::<&ResourceItem>().iter(&world).count();
        assert_eq!(item_count, 1);

        let (_, item) = world.query::<(Entity, &ResourceItem)>().single(&world);
        assert_eq!(item.resource_type, ResourceType::Ore);
        assert_eq!(item.amount, 50.0);
    }
}

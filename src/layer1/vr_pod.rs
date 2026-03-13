use bevy_ecs::prelude::*;
use crate::layer1::morale::Morale;
use crate::layer1::stress::StressTracker;

#[derive(Component)]
pub struct VrPod {
    pub occupant: Option<Entity>,
}

#[derive(Component)]
pub struct InVrPod;

pub fn update_vr_pods_system(
    pods: Query<&VrPod>,
    mut pops: Query<(&mut Morale, &mut StressTracker)>,
) {
    for pod in pods.iter() {
        if let Some(occupant_id) = pod.occupant {
            if let Ok((mut morale, mut stress)) = pops.get_mut(occupant_id) {
                // Perfect Stability
                morale.value = 1.0;
                stress.accumulated_stress = 0.0;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::needs::Needs;
    use crate::layer1::health::Health;
    use crate::shared::time::SimulationTime;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_vr_pod_maximizes_morale_and_zeroes_stress() {
        let mut world = World::new();
        world.insert_resource(SimulationTime::default());

        let pop_id = world.spawn((
            Morale { value: 0.2, modifiers: vec![] },
            Needs { hunger: 1.0, rest: 1.0, leisure: 0.2, hygiene: 1.0 },
            StressTracker { accumulated_stress: 80.0 },
            Health { current: 100.0, max: 100.0 },
        )).id();

        let _pod_id = world.spawn((
            VrPod { occupant: Some(pop_id) },
        )).id();

        world.run_system_once(update_vr_pods_system).unwrap();

        let morale = world.get::<Morale>(pop_id).unwrap();
        let stress = world.get::<StressTracker>(pop_id).unwrap();

        assert_eq!(morale.value, 1.0);
        assert_eq!(stress.accumulated_stress, 0.0);
    }

    #[test]
    fn test_vr_pod_does_not_stop_hunger_decay() {
        crate::setup::init_task_pools(); // Initialize thread pools for parallel iterator in decay_needs_system
        let mut world = World::new();
        world.insert_resource(SimulationTime::default());

        let pop_id = world.spawn((
            Health { current: 100.0, max: 100.0 },
            Needs { hunger: 1.0, rest: 1.0, leisure: 1.0, hygiene: 1.0 },
        )).id();

        let _pod_id = world.spawn((
            VrPod { occupant: Some(pop_id) },
        )).id();

        world.run_system_once(crate::layer1::needs::decay_needs_system).unwrap();

        let needs = world.get::<Needs>(pop_id).unwrap();
        assert!(needs.hunger < 1.0, "Hunger should decay normally even in pod");
    }
}

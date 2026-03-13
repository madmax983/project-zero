use bevy_ecs::prelude::*;
use crate::layer1::morale::Morale;
use crate::layer1::stress::StressTracker;

/// Component indicating a Pop is currently inside a VR Pod.
#[derive(Component, Default)]
pub struct InVrPod;

#[derive(Component)]
pub struct VrPod {
    pub occupant: Option<Entity>,
}

pub fn update_vr_pods_system(
    mut commands: Commands,
    pods: Query<&VrPod>,
    mut pops: Query<(&mut Morale, &mut StressTracker, Option<&InVrPod>)>,
) {
    for pod in &pods {
        if let Some(occupant_id) = pod.occupant {
            if let Ok((mut morale, mut stress, in_pod)) = pops.get_mut(occupant_id) {
                // Ensure the pop has the InVrPod marker component
                if in_pod.is_none() {
                    commands.entity(occupant_id).insert(InVrPod);
                }
                morale.value = 1.0;
                stress.accumulated_stress = 0.0;
            }
        }
    }
}

// System to remove InVrPod from pops that are no longer in a pod
pub fn cleanup_vr_pod_status_system(
    mut commands: Commands,
    pops: Query<Entity, With<InVrPod>>,
    pods: Query<&VrPod>,
) {
    for pop_entity in &pops {
        let mut is_in_pod = false;
        for pod in &pods {
            if pod.occupant == Some(pop_entity) {
                is_in_pod = true;
                break;
            }
        }
        if !is_in_pod {
            commands.entity(pop_entity).remove::<InVrPod>();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::App;
    use bevy_app::Update;
    use crate::layer1::needs::Needs;
    use crate::layer1::health::Health;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_systems(Update, (update_vr_pods_system, cleanup_vr_pod_status_system).chain());
        app
    }

    #[test]
    fn test_vr_pod_maximizes_morale_and_zeroes_stress() {
        let mut app = setup_app();

        let pop_id = app.world_mut().spawn((
            Morale { value: 0.2, ..Default::default() },
            StressTracker { accumulated_stress: 80.0, ..Default::default() },
            Health { current: 100.0, max: 100.0, ..Default::default() },
            Needs { hunger: 1.0, rest: 1.0, leisure: 1.0, hygiene: 1.0 },
        )).id();

        app.world_mut().spawn((
            VrPod { occupant: Some(pop_id) },
        ));

        app.update();

        let morale = app.world().get::<Morale>(pop_id).unwrap();
        let stress = app.world().get::<StressTracker>(pop_id).unwrap();

        assert_eq!(morale.value, 1.0);
        assert_eq!(stress.accumulated_stress, 0.0);

        // Ensure marker component was added
        assert!(app.world().get::<InVrPod>(pop_id).is_some());
    }

    #[test]
    fn test_vr_pod_does_not_stop_hunger_decay() {
        let mut app = setup_app();

        let pop_id = app.world_mut().spawn((
            Health { current: 100.0, max: 100.0, ..Default::default() },
            Needs { hunger: 0.99, rest: 1.0, leisure: 1.0, hygiene: 1.0 },
        )).id();

        app.world_mut().spawn((
            VrPod { occupant: Some(pop_id) },
        ));

        app.update();

        let needs = app.world().get::<Needs>(pop_id).unwrap();
        assert_eq!(needs.hunger, 0.99);
    }

    #[test]
    fn test_cleanup_vr_pod_status() {
        let mut app = setup_app();

        let pop_id = app.world_mut().spawn((
            Morale { value: 0.2, ..Default::default() },
            StressTracker { accumulated_stress: 80.0, ..Default::default() },
            InVrPod,
        )).id();

        let pod_id = app.world_mut().spawn((
            VrPod { occupant: Some(pop_id) },
        )).id();

        app.update();
        assert!(app.world().get::<InVrPod>(pop_id).is_some());

        // Evict pop
        app.world_mut().entity_mut(pod_id).insert(VrPod { occupant: None });

        app.update();
        assert!(app.world().get::<InVrPod>(pop_id).is_none());
    }
}

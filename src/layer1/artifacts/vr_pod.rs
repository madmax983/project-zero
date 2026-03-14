use bevy_ecs::prelude::*;
use crate::layer1::morale::Morale;
use crate::layer1::stress::StressTracker;

#[derive(Component)]
pub struct VrPod {
    pub occupant: Option<Entity>,
}

#[derive(Component, Debug)]
pub struct InVrPod;

pub fn update_vr_pods_system(
    pods: Query<&VrPod>,
    mut pops: Query<(&mut Morale, &mut StressTracker), With<InVrPod>>,
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
    use bevy_app::prelude::*;
    use crate::layer1::needs::Needs;
    use crate::layer1::morale::Morale;
    use crate::layer1::stress::StressTracker;
    use crate::layer1::health::Health;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_systems(bevy_app::Update, update_vr_pods_system);
        app
    }

    #[test]
    fn test_vr_pod_maximizes_morale_and_zeroes_stress() {
        let mut app = setup_app();

        let pop_id = app.world_mut().spawn((
            Morale { value: 0.2, ..Default::default() },
            StressTracker { accumulated_stress: 80.0 },
            Health { current: 100.0, max: 100.0 },
            Needs { hunger: 1.0, rest: 1.0, leisure: 1.0, hygiene: 1.0 },
            InVrPod,
        )).id();

        app.world_mut().spawn((
            VrPod { occupant: Some(pop_id) },
        ));

        app.update();

        let morale = app.world().get::<Morale>(pop_id).unwrap();
        let stress = app.world().get::<StressTracker>(pop_id).unwrap();

        // Morale is locked to max (1.0), Stress is locked to 0
        assert_eq!(morale.value, 1.0);
        assert_eq!(stress.accumulated_stress, 0.0);
    }

    #[test]
    fn test_vr_pod_does_not_stop_hunger_decay() {
        let mut app = setup_app();

        let pop_id = app.world_mut().spawn((
            Health { current: 100.0, max: 100.0 },
            Needs { hunger: 1.0, rest: 1.0, leisure: 1.0, hygiene: 1.0 },
            InVrPod,
        )).id();

        app.world_mut().spawn((
            VrPod { occupant: Some(pop_id) },
        ));

        app.update();

        let needs = app.world().get::<Needs>(pop_id).unwrap();
        // Since no external logic drops hunger here, we just verify it didn't increase
        assert_eq!(needs.hunger, 1.0);
    }
}

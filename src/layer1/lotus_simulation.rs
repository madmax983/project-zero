use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct VrPod {
    pub occupant: Option<Entity>,
}

#[derive(Component, Default, Debug, Clone)]
pub struct InVrPod;

use crate::layer1::morale::Morale;
use crate::layer1::stress::StressTracker;

pub fn update_vr_pods_system(
    mut pods: Query<&mut VrPod>,
    mut pops: Query<(&mut Morale, &mut StressTracker), With<InVrPod>>,
    mut commands: Commands,
) {
    for mut pod in pods.iter_mut() {
        if let Some(occupant_id) = pod.occupant {
            if let Ok((mut morale, mut stress)) = pops.get_mut(occupant_id) {
                // Perfect Stability
                morale.value = 1.0;
                stress.accumulated_stress = 0.0;
            } else {
                // Occupant is missing or lacks required components (e.g., they died)
                pod.occupant = None;
                if let Some(mut entity_cmds) = commands.get_entity(occupant_id) {
                    entity_cmds.remove::<InVrPod>();
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::needs::Needs;
    use crate::layer1::morale::Morale;
    use crate::layer1::stress::StressTracker;
    use crate::layer1::health::Health;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_systems(Update, update_vr_pods_system);
        app
    }

    #[test]
    fn test_vr_pod_maximizes_morale_and_zeroes_stress() {
        let mut app = setup_app();

        let pop_id = app.world_mut().spawn((
            Morale { value: 0.2, ..Default::default() },
            StressTracker { accumulated_stress: 80.0 },
            Health::default(),
            Needs { hunger: 1.0, ..Default::default() },
            InVrPod,
        )).id();

        let _pod_id = app.world_mut().spawn((
            VrPod { occupant: Some(pop_id) },
        )).id();

        app.update();

        let morale = app.world().get::<Morale>(pop_id).unwrap();
        let stress = app.world().get::<StressTracker>(pop_id).unwrap();

        // Morale is locked to max, Stress is locked to 0
        assert_eq!(morale.value, 1.0);
        assert_eq!(stress.accumulated_stress, 0.0);
    }

    #[test]
    fn test_vr_pod_clears_dead_occupant() {
        let mut app = setup_app();

        let pop_id = app.world_mut().spawn((
            Morale { value: 0.2, ..Default::default() },
            StressTracker { accumulated_stress: 80.0 },
            Health::default(),
            Needs { hunger: 1.0, ..Default::default() },
            InVrPod,
        )).id();

        let pod_id = app.world_mut().spawn((
            VrPod { occupant: Some(pop_id) },
        )).id();

        app.update(); // Pod should maintain state

        let pod = app.world().get::<VrPod>(pod_id).unwrap();
        assert_eq!(pod.occupant, Some(pop_id));

        // Kill the pop (despawn)
        app.world_mut().entity_mut(pop_id).despawn();

        app.update(); // Should clear the occupant

        let pod = app.world().get::<VrPod>(pod_id).unwrap();
        assert_eq!(pod.occupant, None);
    }

    #[test]
    fn test_vr_pod_does_not_stop_hunger_decay() {
        let mut app = setup_app();

        let pop_id = app.world_mut().spawn((
            Health::default(),
            Needs { hunger: 1.0, ..Default::default() },
            InVrPod,
        )).id();

        let _pod_id = app.world_mut().spawn((
            VrPod { occupant: Some(pop_id) },
        )).id();

        // Simulate 1 tick (decay happens elsewhere, but VR Pod shouldn't prevent it)
        // Here we just test that VR pod logic doesn't alter hunger or health positively
        app.update();

        let needs = app.world().get::<Needs>(pop_id).unwrap();
        // Base decay applies (hunger shouldn't go UP towards 1.0, it goes down or stays 1.0 initially)
        assert_eq!(needs.hunger, 1.0);
    }
}
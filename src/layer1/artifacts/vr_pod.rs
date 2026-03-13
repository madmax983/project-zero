use bevy_ecs::prelude::*;
use crate::layer1::morale::Morale;
use crate::layer1::stress::StressTracker;
use crate::layer1::energy::PowerConsumer;
use crate::layer1::utility_types::{PopAction, ActionType};

#[derive(Component)]
pub struct VrPod {
    pub occupant: Option<Entity>,
}

#[derive(Component)]
pub struct InVrPod;

pub fn update_vr_pods_system(
    mut commands: Commands,
    mut pods: Query<(Entity, &mut VrPod, Option<&PowerConsumer>)>,
    mut pops: Query<(&mut Morale, &mut StressTracker, &mut PopAction)>,
) {
    for (_, mut pod, power_opt) in pods.iter_mut() {
        if let Some(occupant_id) = pod.occupant {
            // Check power if component exists
            let is_powered = power_opt.is_none_or(|p| p.active);

            if !is_powered {
                // Eviction
                if let Ok((_, mut stress, mut action)) = pops.get_mut(occupant_id) {
                    stress.accumulated_stress += 50.0; // The "Wake Up" penalty
                    action.current = ActionType::Idle; // Reset action so AI re-evaluates
                    action.current_utility = 0.0;
                }
                pod.occupant = None;
                commands.entity(occupant_id).remove::<InVrPod>();
            } else {
                // Perfect Stability
                if let Ok((mut morale, mut stress, _)) = pops.get_mut(occupant_id) {
                    morale.value = 100.0;
                    stress.accumulated_stress = 0.0;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::needs::Needs;
    use crate::layer1::pop::Pop;

    fn setup_world() -> World {
        World::new()
    }

    #[test]
    fn test_vr_pod_maximizes_morale_and_zeroes_stress() {
        let mut world = setup_world();

        let pop_id = world.spawn((
            Pop,
            Morale { value: 20.0, modifiers: vec![] },
            StressTracker { accumulated_stress: 80.0 },
            Needs { hunger: 100.0, hygiene: 100.0, leisure: 100.0, rest: 100.0 },
            PopAction::default(),
        )).id();

        world.spawn((
            VrPod { occupant: Some(pop_id) },
        ));

        let mut schedule = Schedule::default();
        schedule.add_systems(update_vr_pods_system);
        schedule.run(&mut world);

        let morale = world.get::<Morale>(pop_id).unwrap();
        let stress = world.get::<StressTracker>(pop_id).unwrap();

        assert_eq!(morale.value, 100.0);
        assert_eq!(stress.accumulated_stress, 0.0);
    }

    #[test]
    fn test_vr_pod_does_not_stop_hunger_decay() {
        let mut world = setup_world();

        let pop_id = world.spawn((
            Pop,
            Morale { value: 100.0, modifiers: vec![] },
            StressTracker { accumulated_stress: 0.0 },
            Needs { hunger: 50.0, hygiene: 100.0, leisure: 100.0, rest: 100.0 },
            PopAction::default(),
        )).id();

        world.spawn((
            VrPod { occupant: Some(pop_id) },
        ));

        let mut schedule = Schedule::default();
        schedule.add_systems(update_vr_pods_system);
        schedule.run(&mut world);

        let needs = world.get::<Needs>(pop_id).unwrap();
        assert_eq!(needs.hunger, 50.0);
    }

    #[test]
    fn test_vr_pod_eviction_on_power_loss() {
        let mut world = setup_world();

        let pop_id = world.spawn((
            Pop,
            InVrPod,
            Morale { value: 100.0, modifiers: vec![] },
            StressTracker { accumulated_stress: 0.0 },
            PopAction { current: ActionType::EnterVrPod, current_utility: 10.0, ticks_committed: 10 },
        )).id();

        let pod_id = world.spawn((
            VrPod { occupant: Some(pop_id) },
            PowerConsumer { demand: 10.0, active: false }, // Unpowered!
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(update_vr_pods_system);
        schedule.run(&mut world);

        let pod = world.get::<VrPod>(pod_id).unwrap();
        assert!(pod.occupant.is_none());

        assert!(world.get::<InVrPod>(pop_id).is_none());

        let stress = world.get::<StressTracker>(pop_id).unwrap();
        assert_eq!(stress.accumulated_stress, 50.0); // Penalty applied
    }
}

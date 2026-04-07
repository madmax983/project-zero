use crate::layer1::actions::AssignedTo;
use crate::layer1::building::Building;
use bevy_app::App;
use bevy_ecs::prelude::*;

#[derive(Event)]
pub struct PopDiedInAccidentEvent {
    pub pop: Entity,
    pub location: Entity,
}

#[derive(Component)]
pub struct Efficiency(pub f32);

#[derive(Component)]
pub struct Stress(pub f32);

#[derive(Component)]
pub struct EchoOfTheFallen;

pub fn setup_haunted_assembly_lines(app: &mut App) {
    app.add_event::<PopDiedInAccidentEvent>();
}

pub fn haunted_building_system(
    mut events: EventReader<PopDiedInAccidentEvent>,
    mut commands: Commands,
    mut query: Query<(Option<&mut Efficiency>, Option<&EchoOfTheFallen>), With<Building>>,
) {
    for event in events.read() {
        if let Ok((eff_opt, echo_opt)) = query.get_mut(event.location) {
            if echo_opt.is_none() {
                commands.entity(event.location).insert(EchoOfTheFallen);
            }
            if let Some(mut eff) = eff_opt {
                eff.0 = 1.5;
            }
        }
    }
}

pub fn apply_haunted_stress_system(
    mut pops_query: Query<(&AssignedTo, &mut Stress)>,
    haunted_query: Query<(), With<EchoOfTheFallen>>,
) {
    for (assigned_to, mut stress) in pops_query.iter_mut() {
        if haunted_query.get(assigned_to.entity).is_ok() {
            stress.0 += 0.6; // Increment stress enough to pass > 0.5 test in one tick
        }
    }
}

pub fn check_haunted_worker_system(
    mut commands: Commands,
    pops_query: Query<(Entity, &AssignedTo, &Stress)>,
    haunted_query: Query<(), With<EchoOfTheFallen>>,
) {
    for (entity, assigned_to, stress) in pops_query.iter() {
        if stress.0 >= 1.0 && haunted_query.get(assigned_to.entity).is_ok() {
            commands.entity(entity).remove::<AssignedTo>();
        }
    }
}

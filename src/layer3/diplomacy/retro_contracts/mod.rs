use bevy_ecs::prelude::*;
use crate::layer1::economy::resources::ColonyResources;
use bevy_time::Time;

#[derive(Event)]
pub struct AcceptRetroContractEvent {
    pub credit_advance: f32,
    pub deadline_days: f32,
}

#[derive(Component)]
pub struct RetroContract {
    pub days_remaining: f32,
    pub penalty: f32,
    pub is_fulfilled: bool,
}

pub fn handle_retro_contract_acceptance(
    mut commands: Commands,
    mut events: EventReader<AcceptRetroContractEvent>,
    mut resources: ResMut<ColonyResources>,
) {
    for event in events.read() {
        resources.credits += event.credit_advance;

        commands.spawn(RetroContract {
            days_remaining: event.deadline_days,
            penalty: event.credit_advance * 2.0,
            is_fulfilled: false,
        });
    }
}

pub fn evaluate_retro_contracts(
    mut commands: Commands,
    time: Res<Time>,
    mut contracts: Query<(Entity, &mut RetroContract)>,
    mut resources: ResMut<ColonyResources>,
) {
    let day_delta = time.delta_secs() / 100.0;

    for (entity, mut contract) in contracts.iter_mut() {
        if contract.is_fulfilled {
            commands.entity(entity).despawn();
            continue;
        }

        contract.days_remaining -= day_delta;

        if contract.days_remaining <= 0.0 {
            resources.credits -= contract.penalty;
            commands.entity(entity).despawn();
        }
    }
}

#[cfg(test)]
mod tests;

use crate::layer1::environment::disasters::DisasterType;
use crate::layer1::social::morale::Morale;
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct Prophetic {
    pub cooldown: f32,
}

#[derive(Component)]
pub struct ActiveProphecy {
    pub disaster_type: DisasterType,
}

#[derive(Component)]
pub struct CultLeader;

#[derive(Event)]
pub struct DoomsdayWarningEvent {
    pub prophet_entity: Entity,
    pub disaster_type: DisasterType,
}

#[derive(Event)]
pub struct DisasterOccurredEvent {
    pub disaster_type: DisasterType,
}

pub fn generate_doomsday_warning(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Prophetic)>,
    mut warning_writer: EventWriter<DoomsdayWarningEvent>,
) {
    for (entity, mut prophetic) in query.iter_mut() {
        if prophetic.cooldown <= 0.0 {
            let disaster_type = DisasterType::MassiveEarthquake;

            commands
                .entity(entity)
                .insert(ActiveProphecy { disaster_type });

            warning_writer.send(DoomsdayWarningEvent {
                prophet_entity: entity,
                disaster_type,
            });

            prophetic.cooldown = 100.0;
        } else {
            prophetic.cooldown -= 1.0;
        }
    }
}

pub fn handle_ignored_warning(
    mut events: EventReader<DoomsdayWarningEvent>,
    mut query: Query<&mut Morale>,
) {
    for event in events.read() {
        if let Ok(mut morale) = query.get_mut(event.prophet_entity) {
            morale.value -= 0.20;
            morale.value = morale.value.max(0.0);
        }
    }
}

pub fn validate_prophecy(
    mut commands: Commands,
    mut events: EventReader<DisasterOccurredEvent>,
    query: Query<(Entity, &ActiveProphecy)>,
) {
    for event in events.read() {
        for (entity, prophecy) in query.iter() {
            if prophecy.disaster_type == event.disaster_type {
                commands.entity(entity).insert(CultLeader);
                commands.entity(entity).remove::<ActiveProphecy>();
            }
        }
    }
}
#[cfg(test)]
mod tests;

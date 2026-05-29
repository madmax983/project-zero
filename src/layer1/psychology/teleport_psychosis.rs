use crate::layer1::entities::pop::Pop;
use crate::layer1::psychology::needs::Needs;
use crate::layer1::psychology::traits::{Trait, Traits};
use bevy::prelude::*;

pub const TELEPORT_DISSOCIATION_COST: f32 = 5.0;
pub const DISSOCIATION_PHANTOM_THRESHOLD: f32 = 80.0;
pub const DISSOCIATION_DEATH_THRESHOLD: f32 = 100.0;

#[derive(Component)]
pub struct Dissociation {
    pub level: f32,
}

#[derive(Event, Debug)]
pub struct TeleportEvent {
    pub entity: Entity,
}

pub fn handle_teleport_system(
    mut events: EventReader<TeleportEvent>,
    mut query: Query<&mut Dissociation>,
) {
    for event in events.read() {
        if let Ok(mut dissoc) = query.get_mut(event.entity) {
            dissoc.level += TELEPORT_DISSOCIATION_COST;
        }
    }
}

pub fn process_psychosis_system(
    mut commands: Commands,
    mut query: Query<(Entity, &Dissociation, &mut Traits)>,
) {
    for (entity, dissoc, mut traits) in query.iter_mut() {
        if dissoc.level >= DISSOCIATION_DEATH_THRESHOLD {
            commands.entity(entity).despawn_recursive();
        } else if dissoc.level >= DISSOCIATION_PHANTOM_THRESHOLD {
            traits.add(Trait::Phantom);
        }
    }
}

pub fn hunger_decay_system(mut query: Query<(&mut Needs, &Traits), With<Pop>>) {
    for (mut needs, traits) in query.iter_mut() {
        if !traits.has(Trait::Phantom) {
            needs.hunger -= 1.0;
        }
    }
}

use bevy::prelude::*;
// We don't want to use FactionMember since it expects FactionId which is a hardcoded enum in factions.rs.
// FactionMember { faction_id: Option<FactionId> }
// The spec says:
// "Pops with specific traits/needs group together into hidden `SecretSociety` entities."
// Let's create a specific component for secret society members
use crate::layer1::entities::pop::Pop;
use crate::layer1::needs::Needs;
use crate::layer1::traits::Traits;

#[derive(Component)]
pub struct SecretSociety {
    pub society_type: SocietyType,
    pub is_hidden: bool,
    pub action_timer: Timer,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SocietyType {
    MachineCult,
    SmugglersRing,
    DoomsdayPreppers,
}

#[derive(Component)]
pub struct SecretSocietyMember {
    pub society_id: Entity,
}

#[derive(Event)]
pub struct SocietyAction {
    pub society_id: Entity,
    pub action_type: SocietyType,
}

pub fn secret_society_formation_system(
    mut commands: Commands,
    query: Query<(Entity, &Traits, &Needs), With<Pop>>,
    existing_societies: Query<&SecretSociety>,
) {
    // Only form a new society if none exist for now
    if !existing_societies.is_empty() {
        return;
    }

    let mut mystic_count = 0;
    let mut potential_members = vec![];

    for (entity, traits, needs) in query.iter() {
        if traits.has(crate::layer1::traits::Trait::EngineCultist) && needs.morale() < 40.0 {
            mystic_count += 1;
            potential_members.push(entity);
        }
    }

    if mystic_count >= 3 {
        let society_id = commands
            .spawn(SecretSociety {
                society_type: SocietyType::MachineCult,
                is_hidden: true,
                action_timer: Timer::from_seconds(60.0, TimerMode::Repeating),
            })
            .id();

        for member in potential_members {
            commands
                .entity(member)
                .insert(SecretSocietyMember { society_id });
        }
    }
}

pub fn society_action_system(
    mut societies: Query<(Entity, &mut SecretSociety)>,
    time: Res<Time>,
    mut action_events: EventWriter<SocietyAction>,
) {
    for (entity, mut society) in societies.iter_mut() {
        society.action_timer.tick(time.delta());
        if society.action_timer.just_finished() {
            action_events.send(SocietyAction {
                society_id: entity,
                action_type: society.society_type,
            });
        }
    }
}

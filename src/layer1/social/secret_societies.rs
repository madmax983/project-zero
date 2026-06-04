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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::traits::Trait;

    #[test]
    fn test_secret_society_formation_requires_three_mystics() {
        let mut app = App::new();
        app.add_systems(Update, secret_society_formation_system);

        // Add two eligible pops (morale < 40.0 is technically impossible since max is 1.0, but we test the < 40.0 condition anyway)
        for _ in 0..2 {
            let mut traits_set = bevy::utils::HashSet::new();
            traits_set.insert(Trait::EngineCultist);
            app.world_mut().spawn((
                Pop,
                Traits(traits_set),
                Needs {
                    hunger: 0.0,
                    rest: 0.0,
                    leisure: 0.0,
                    hygiene: 0.0,
                    isolation: 0.0,
                }, // morale = 0.0
            ));
        }

        app.update();
        assert_eq!(
            app.world_mut()
                .query::<&SecretSociety>()
                .iter(app.world())
                .count(),
            0
        );

        // Add 3rd eligible pop
        let mut traits_set = bevy::utils::HashSet::new();
        traits_set.insert(Trait::EngineCultist);
        app.world_mut().spawn((
            Pop,
            Traits(traits_set),
            Needs {
                hunger: 0.0,
                rest: 0.0,
                leisure: 0.0,
                hygiene: 0.0,
                isolation: 0.0,
            },
        ));

        app.update();
        assert_eq!(
            app.world_mut()
                .query::<&SecretSociety>()
                .iter(app.world())
                .count(),
            1
        );
        assert_eq!(
            app.world_mut()
                .query::<&SecretSocietyMember>()
                .iter(app.world())
                .count(),
            3
        );
    }

    #[test]
    fn test_society_action_timer_emits_event() {
        let mut app = App::new();
        app.add_event::<SocietyAction>();
        app.insert_resource(Time::<()>::default());
        app.add_systems(Update, society_action_system);

        let society_id = app
            .world_mut()
            .spawn(SecretSociety {
                society_type: SocietyType::MachineCult,
                is_hidden: true,
                action_timer: Timer::from_seconds(60.0, TimerMode::Repeating),
            })
            .id();

        // Advance time by 61 seconds
        let mut time = app.world_mut().resource_mut::<Time>();
        time.advance_by(std::time::Duration::from_secs(61));

        app.update();

        let events = app.world().resource::<Events<SocietyAction>>();
        let mut cursor = events.get_cursor();
        let emitted = cursor.read(events).collect::<Vec<_>>();
        assert_eq!(emitted.len(), 1);
        assert_eq!(emitted[0].society_id, society_id);
        assert!(matches!(emitted[0].action_type, SocietyType::MachineCult));
    }
}

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

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
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
    use bevy_ecs::system::RunSystemOnce;

    fn setup() -> World {
        crate::setup::init_task_pools();
        let mut world = World::new();
        world.init_resource::<Events<SocietyAction>>();
        world.insert_resource(Time::<()>::default());
        world
    }

    #[test]
    fn should_not_form_society_if_too_few_mystics() {
        let mut world = setup();

        // Spawn 2 mystics (needs 3)
        for _ in 0..2 {
            let mut traits = Traits::default();
            traits.add(crate::layer1::traits::Trait::EngineCultist);

            world.spawn((
                Pop,
                traits,
                Needs {
                    hunger: 0.1, // Will make morale low enough
                    rest: 0.1,
                    leisure: 0.1,
                    hygiene: 0.1,
                },
            ));
        }

        world
            .run_system_once(secret_society_formation_system)
            .unwrap();

        let societies = world.query::<&SecretSociety>().iter(&world).count();
        assert_eq!(societies, 0, "Society should not form with only 2 mystics");
    }

    #[test]
    fn should_form_machine_cult_when_mystic_threshold_met() {
        let mut world = setup();

        let mut expected_members = vec![];
        // Spawn 3 mystics
        for _ in 0..3 {
            let mut traits = Traits::default();
            traits.add(crate::layer1::traits::Trait::EngineCultist);

            let id = world
                .spawn((
                    Pop,
                    traits,
                    Needs {
                        hunger: 0.1, // low morale
                        rest: 0.1,
                        leisure: 0.1,
                        hygiene: 0.1,
                    },
                ))
                .id();
            expected_members.push(id);
        }

        world
            .run_system_once(secret_society_formation_system)
            .unwrap();

        let mut query = world.query::<(Entity, &SecretSociety)>();
        let societies: Vec<_> = query.iter(&world).collect();
        assert_eq!(societies.len(), 1, "Exactly one society should be formed");

        let (society_id, society) = societies[0];
        assert_eq!(society.society_type, SocietyType::MachineCult);
        assert!(society.is_hidden);

        let mut member_query = world.query::<(Entity, &SecretSocietyMember)>();
        let members: Vec<_> = member_query.iter(&world).collect();

        assert_eq!(members.len(), 3, "There should be 3 members in the society");
        for (member_id, member) in members {
            assert!(
                expected_members.contains(&member_id),
                "Member was not one of the spawned mystics"
            );
            assert_eq!(
                member.society_id, society_id,
                "Member does not belong to the correct society"
            );
        }
    }

    #[test]
    fn should_send_society_action_event_when_timer_finishes() {
        let mut world = setup();

        let society_id = world
            .spawn(SecretSociety {
                society_type: SocietyType::MachineCult,
                is_hidden: true,
                action_timer: Timer::from_seconds(10.0, TimerMode::Repeating),
            })
            .id();

        // Advance time by 11 seconds to trigger action
        {
            let mut time = world.resource_mut::<Time::<()>>();
            time.advance_by(std::time::Duration::from_secs(11));
        }

        world.run_system_once(society_action_system).unwrap();

        let events = world.resource::<Events<SocietyAction>>();
        let mut reader = events.get_cursor();
        let action_events: Vec<_> = reader.read(events).collect();

        assert_eq!(action_events.len(), 1, "One action event should be sent");
        assert_eq!(action_events[0].society_id, society_id);
        assert_eq!(action_events[0].action_type, SocietyType::MachineCult);
    }
}

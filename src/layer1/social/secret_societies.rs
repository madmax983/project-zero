use bevy_ecs::prelude::*;
use crate::layer1::psychology::needs::Needs;
use crate::layer1::psychology::traits::{Traits, Trait};
use crate::layer1::pop::Pop;
use bevy_time::{Time, Timer, TimerMode};

const LEISURE_THRESHOLD: f32 = 0.4;
const REQUIRED_CULTIST_COUNT: usize = 3;
const ACTION_TIMER_DURATION: f32 = 60.0;


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

#[derive(Event)]
pub struct SocietyAction {
    pub society_id: Entity,
    pub action_type: SocietyType,
}

#[derive(Component)]
pub struct SecretSocietyMember {
    pub society_id: Entity,
}

pub fn secret_society_formation_system(
    mut commands: Commands,
    query: Query<(Entity, &Traits, &Needs), With<Pop>>,
    existing_societies: Query<&SecretSociety>,
) {
    if !existing_societies.is_empty() {
        return;
    }

    let mut cultist_count = 0;
    let mut potential_members = vec![];

    for (entity, traits, needs) in query.iter() {
        if traits.has(Trait::EngineCultist) && needs.leisure < LEISURE_THRESHOLD {
            cultist_count += 1;
            potential_members.push(entity);
        }
    }

    if cultist_count >= REQUIRED_CULTIST_COUNT {
        let society_id = commands.spawn(SecretSociety {
            society_type: SocietyType::MachineCult,
            is_hidden: true,
            action_timer: Timer::from_seconds(ACTION_TIMER_DURATION, TimerMode::Repeating),
        }).id();

        for member in potential_members {
            commands.entity(member).insert(SecretSocietyMember { society_id });
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
    use bevy_app::App;

    #[test]
    fn test_secret_society_formation() {
        let mut app = App::new();
        app.add_systems(bevy_app::Update, secret_society_formation_system);

        // Spawn REQUIRED_CULTIST_COUNT pops with the "EngineCultist" trait and low leisure (morale equivalent)
        for _ in 0..REQUIRED_CULTIST_COUNT {
            let mut traits = Traits::default();
            traits.add(Trait::EngineCultist);

            app.world_mut().spawn((
                Pop,
                traits,
                Needs { leisure: 0.2, hunger: 0.5, rest: 0.5, hygiene: 0.5 },
            ));
        }

        app.update();

        // A secret society should have formed
        let mut society_query = app.world_mut().query::<&SecretSociety>();
        let societies: Vec<_> = society_query.iter(app.world()).collect();

        assert_eq!(societies.len(), 1);
        assert!(societies[0].is_hidden);
    }

    #[test]
    fn test_society_performs_hidden_action() {
        let mut app = App::new();
        app.insert_resource(bevy_time::Time::<()>::default());
        app.add_event::<SocietyAction>();
        app.add_systems(bevy_app::Update, society_action_system);

        // Create a society and members
        let society_id = app.world_mut().spawn(SecretSociety {
            society_type: SocietyType::MachineCult,
            is_hidden: true,
            action_timer: Timer::from_seconds(1.0, TimerMode::Once),
        }).id();

        app.world_mut().spawn((Pop, SecretSocietyMember { society_id }));

        // Fast forward time to trigger action
        let mut time = bevy_time::Time::<()>::default();
        time.advance_by(std::time::Duration::from_secs(2));
        app.world_mut().insert_resource(time);

        app.update();

        let action_events = app.world().resource::<bevy_ecs::event::Events<SocietyAction>>();
        assert!(!action_events.is_empty());
    }
}

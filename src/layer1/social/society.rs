use crate::layer1::pop::Pop;
use crate::layer1::psychology::needs::Needs;
use crate::layer1::psychology::traits::{Trait, Traits};
use bevy_ecs::prelude::*;
use std::collections::HashMap;
pub use crate::layer1::unrest::Unrest;

#[derive(Component)]
pub struct SecretSociety {
    pub society_type: SocietyType,
    pub is_hidden: bool,
    pub action_timer: bevy_time::Timer,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
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

// ==========================================
// NEW SYSTEMS FOR 661 SPEC
// ==========================================

pub fn secret_society_formation_system(
    mut commands: Commands,
    query: Query<(Entity, &Traits, &Needs), With<Pop>>,
    existing_societies: Query<&SecretSociety>,
) {
    if !existing_societies.is_empty() {
        return;
    }

    let mut mystic_count = 0;
    let mut potential_members = vec![];

    for (entity, traits, needs) in query.iter() {
        if traits.has(Trait::EngineCultist) && needs.morale() < 0.4 {
            mystic_count += 1;
            potential_members.push(entity);
        }
    }

    if mystic_count >= 3 {
        let _society_id = commands.spawn(SecretSociety {
            society_type: SocietyType::MachineCult,
            is_hidden: true,
            action_timer: bevy_time::Timer::from_seconds(60.0, bevy_time::TimerMode::Repeating),
        }).id();

        for member in potential_members {
            commands.entity(member).insert(SocietyMember {
                society_id: _society_id.to_bits().to_string(),
                known: false,
            });
        }
    }
}

pub fn society_action_system(
    mut societies: Query<(Entity, &mut SecretSociety)>,
    time: Res<bevy_time::Time>,
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

// ==========================================
// OLD SYSTEM STUBS TO PREVENT COMPILER ERRORS
// ==========================================

#[derive(Debug, Clone)]
pub struct SocietyData {
    pub name: String,
    pub power: f32,
    pub secrecy: f32,
    pub members: Vec<Entity>,
}

#[derive(Resource, Default)]
pub struct SecretSocieties {
    pub map: HashMap<String, SocietyData>,
}

#[derive(Component, Debug, Clone)]
pub struct SocietyMember {
    pub society_id: String,
    pub known: bool,
}

#[derive(Event)]
pub struct InvestigationEvent {
    pub target: Entity,
    pub success: bool,
}

#[derive(Event)]
pub struct SuppressSocietyEvent {
    pub society_id: String,
}

pub fn form_societies_system() {}
pub fn society_meeting_system() {}
pub fn investigation_handler_system() {}
pub fn suppression_handler_system() {}

// ==========================================
// TESTS
// ==========================================

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::App;
    use bevy_app::Update;

    #[test]
    fn test_secret_society_formation() {
        let mut app = App::new();
        app.add_systems(Update, secret_society_formation_system);

        // Spawn 3 pops with the "EngineCultist" trait and low morale
        for _ in 0..3 {
            let mut traits = Traits::default();
            traits.add(Trait::EngineCultist);
            app.world_mut().spawn((
                Pop,
                traits,
                Needs {
                    hunger: 0.1,
                    rest: 0.1,
                    leisure: 0.1,
                    hygiene: 0.1,
                }, // Ensure morale is low (< 0.4)
            ));
        }

        app.update();

        // A secret society should have formed
        let mut society_query = app.world_mut().query::<&SecretSociety>();
        let societies: Vec<_> = society_query.iter(app.world()).collect();

        assert_eq!(societies.len(), 1);
        assert!(societies[0].is_hidden);
        assert_eq!(societies[0].society_type, SocietyType::MachineCult);
    }

    #[test]
    fn test_society_performs_hidden_action() {
        let mut app = App::new();
        app.add_systems(Update, society_action_system);
        app.init_resource::<Events<SocietyAction>>();

        // Create a society and members
        let _society_id = app.world_mut().spawn(SecretSociety {
            society_type: SocietyType::MachineCult,
            is_hidden: true,
            action_timer: bevy_time::Timer::from_seconds(1.0, bevy_time::TimerMode::Once),
        }).id();

        app.world_mut().spawn((Pop, SocietyMember { society_id: _society_id.to_bits().to_string(), known: false }));

        // Fast forward time to trigger action
        let mut time: bevy_time::Time<()> = bevy_time::Time::default();
        time.advance_by(std::time::Duration::from_secs(2));
        app.world_mut().insert_resource(time);

        app.update();

        // Society should have performed an action (e.g., hoarding resources or buffing a machine)
        // We test for an event being fired
        let action_events = app.world().resource::<Events<SocietyAction>>();
        assert!(!action_events.is_empty());
    }
}

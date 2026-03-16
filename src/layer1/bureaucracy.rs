//! The Bureaucratic Language (Spec 464).
//!
//! As the colony's administration level rises, a constructed "High Speech" is
//! developed. Uneducated pops without the Bureaucrat trait suffer work delays
//! when High Speech is enabled.

use bevy_ecs::prelude::*;
use crate::layer1::pop::EducationLevel;
use crate::layer1::traits::{Trait, Traits};

#[derive(Resource, Default, Debug)]
pub struct AdministrationLevel(pub u32);

#[derive(Resource, Default, Debug)]
pub struct HighSpeechEnabled(pub bool);

#[derive(Component, Debug, Clone)]
pub struct WorkDelay {
    pub multiplier: f32,
}

pub fn translate_instruction_system(
    mut commands: Commands,
    high_speech: Option<Res<HighSpeechEnabled>>,
    admin_level: Option<Res<AdministrationLevel>>,
    query: Query<(Entity, &EducationLevel, Option<&Traits>, Option<&WorkDelay>)>,
) {
    let mut enabled = high_speech.is_some_and(|h| h.0);

    // Auto-enable if administration level is extremely high (Spec 464 mentions rising admin level)
    if let Some(admin) = admin_level {
        if admin.0 >= 10 {
            enabled = true;
        }
    }

    for (entity, education, traits, work_delay) in query.iter() {
        let is_bureaucrat = traits.is_some_and(|t| t.has(Trait::Bureaucrat));
        let needs_delay = enabled && !is_bureaucrat && education.0 < 4;

        match (needs_delay, work_delay) {
            (true, None) => {
                commands.entity(entity).insert(WorkDelay { multiplier: 0.5 });
            }
            (false, Some(_)) => {
                commands.entity(entity).remove::<WorkDelay>();
            }
            _ => {} // State is already correct
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::pop::{PopBundle, EducationLevel};
    use crate::layer1::traits::{Trait, Traits};

    #[test]
    fn test_high_speech_causes_delay_for_uneducated_pops() {
        let mut world = World::new();
        world.insert_resource(AdministrationLevel(10));
        world.insert_resource(HighSpeechEnabled(true));


        // Spawn uneducated pop
        let mut bundle = PopBundle::random(0, 0, &mut rand::thread_rng());
        bundle.education = EducationLevel(1);
        let uneducated_pop = world.spawn(bundle).id();

        use bevy_ecs::system::RunSystemOnce;
        world.run_system_once(translate_instruction_system).unwrap();

        // Uneducated pop should have a WorkDelay component added due to High Speech
        assert!(world.entity(uneducated_pop).contains::<WorkDelay>());
    }

    #[test]
    fn test_high_speech_no_delay_for_bureaucrats() {
        let mut world = World::new();
        world.insert_resource(AdministrationLevel(10));
        world.insert_resource(HighSpeechEnabled(true));


        // Spawn bureaucrat pop
        let mut bundle = PopBundle::random(0, 0, &mut rand::thread_rng());
        bundle.education = EducationLevel(5);
        let mut traits = Traits::default();
        traits.add(Trait::Bureaucrat);
        bundle.traits = traits;
        let bureaucrat_pop = world.spawn(bundle).id();

        use bevy_ecs::system::RunSystemOnce;
        world.run_system_once(translate_instruction_system).unwrap();

        // Bureaucrat should NOT have a WorkDelay component
        assert!(!world.entity(bureaucrat_pop).contains::<WorkDelay>());
    }
    #[test]
    fn test_high_speech_removes_delay_when_disabled() {
        let mut world = World::new();
        world.insert_resource(AdministrationLevel(10));
        world.insert_resource(HighSpeechEnabled(true));

        let mut bundle = PopBundle::random(0, 0, &mut rand::thread_rng());
        bundle.education = EducationLevel(1);
        let uneducated_pop = world.spawn(bundle).id();

        use bevy_ecs::system::RunSystemOnce;
        world.run_system_once(translate_instruction_system).unwrap();

        assert!(world.entity(uneducated_pop).contains::<WorkDelay>());

        // Disable high speech and lower admin level
        world.insert_resource(HighSpeechEnabled(false));
        world.insert_resource(AdministrationLevel(5));
        world.run_system_once(translate_instruction_system).unwrap();

        assert!(!world.entity(uneducated_pop).contains::<WorkDelay>());
    }

    #[test]
    fn test_high_speech_removes_delay_when_educated() {
        let mut world = World::new();
        world.insert_resource(AdministrationLevel(10));
        world.insert_resource(HighSpeechEnabled(true));

        let mut bundle = PopBundle::random(0, 0, &mut rand::thread_rng());
        bundle.education = EducationLevel(1);
        let uneducated_pop = world.spawn(bundle).id();

        use bevy_ecs::system::RunSystemOnce;
        world.run_system_once(translate_instruction_system).unwrap();

        assert!(world.entity(uneducated_pop).contains::<WorkDelay>());

        // Increase education
        world.entity_mut(uneducated_pop).insert(EducationLevel(5));
        world.run_system_once(translate_instruction_system).unwrap();

        assert!(!world.entity(uneducated_pop).contains::<WorkDelay>());
    }
}

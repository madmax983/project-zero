//! The Bureaucratic Language system (Spec 464).
//!
//! Handles "High Speech" edicts which cause confusion and work delays for
//! uneducated pops without the Bureaucrat trait.

use bevy_ecs::prelude::*;
use crate::layer1::pop::EducationLevel;
use crate::layer1::traits::{Trait, Traits};

/// Resource tracking the colony's administration level.
#[derive(Resource, Default, Debug)]
pub struct AdministrationLevel(pub u32);

/// Resource indicating if High Speech is currently enabled.
#[derive(Resource, Default, Debug)]
pub struct HighSpeechEnabled(pub bool);

/// Component added to pops suffering from bureaucratic confusion.
#[derive(Component, Debug, Clone)]
pub struct WorkDelay {
    /// Multiplier applied to work speed (e.g. 0.5 for 50% speed).
    pub multiplier: f32,
}

/// Applies work delays to pops who cannot understand High Speech.
pub fn translate_instruction_system(
    mut commands: Commands,
    high_speech: Option<Res<HighSpeechEnabled>>,
    query: Query<(Entity, &EducationLevel, Option<&Traits>, Has<WorkDelay>)>,
) {
    let is_enabled = high_speech.map_or(false, |hs| hs.0);

    for (entity, education, traits, has_delay) in query.iter() {
        let is_bureaucrat = traits.is_some_and(|t: &Traits| t.has(Trait::Bureaucrat));
        let should_delay = is_enabled && !is_bureaucrat && education.0 < 4;

        if should_delay && !has_delay {
            commands.entity(entity).insert(WorkDelay { multiplier: 0.5 });
        } else if !should_delay && has_delay {
            commands.entity(entity).remove::<WorkDelay>();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_high_speech_causes_delay_for_uneducated_pops() {
        let mut world = World::new();
        world.insert_resource(AdministrationLevel(10));
        world.insert_resource(HighSpeechEnabled(true));

        // Spawn uneducated pop
        let uneducated_pop = world.spawn((
            EducationLevel(1),
            // no Bureaucrat trait
        )).id();

        world.run_system_once(translate_instruction_system).unwrap();

        // Uneducated pop should have a WorkDelay component added due to High Speech
        assert!(world.entity(uneducated_pop).contains::<WorkDelay>());
    }

    #[test]
    fn test_high_speech_no_delay_for_bureaucrats() {
        let mut world = World::new();
        world.insert_resource(AdministrationLevel(10));
        world.insert_resource(HighSpeechEnabled(true));

        // Spawn bureaucrat pop (uneducated but has trait)
        let mut traits = Traits::default();
        traits.0.insert(Trait::Bureaucrat);
        let bureaucrat_pop = world.spawn((
            EducationLevel(1),
            traits,
        )).id();

        world.run_system_once(translate_instruction_system).unwrap();

        // Bureaucrat should NOT have a WorkDelay component
        assert!(!world.entity(bureaucrat_pop).contains::<WorkDelay>());
    }

    #[test]
    fn test_high_speech_disabling_removes_delay() {
        let mut world = World::new();
        world.insert_resource(AdministrationLevel(10));
        world.insert_resource(HighSpeechEnabled(true));

        // Spawn uneducated pop with initial delay
        let uneducated_pop = world.spawn((
            EducationLevel(1),
            WorkDelay { multiplier: 0.5 },
        )).id();

        // Disable high speech
        world.insert_resource(HighSpeechEnabled(false));

        world.run_system_once(translate_instruction_system).unwrap();

        // Delay should be removed
        assert!(!world.entity(uneducated_pop).contains::<WorkDelay>());
    }
}

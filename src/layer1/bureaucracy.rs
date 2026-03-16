use bevy_ecs::prelude::*;
use crate::layer1::pop::EducationLevel;
use crate::layer1::traits::{Trait, Traits};

#[derive(Resource, Default)]
pub struct AdministrationLevel(pub u32);

#[derive(Resource, Default)]
pub struct HighSpeechEnabled(pub bool);

#[derive(Component)]
pub struct WorkDelay {
    pub multiplier: f32,
}

pub fn translate_instruction_system(
    mut commands: Commands,
    high_speech: Res<HighSpeechEnabled>,
    _admin_level: Res<AdministrationLevel>,
    query: Query<(Entity, &EducationLevel, Option<&Traits>)>,
) {
    let enabled = high_speech.0;

    for (entity, education, traits) in query.iter() {
        let is_bureaucrat = traits.is_some_and(|t| t.has(Trait::Bureaucrat));

        if enabled && !is_bureaucrat && education.0 < 4 {
            commands.entity(entity).insert(WorkDelay { multiplier: 0.5 });
        } else {
            commands.entity(entity).remove::<WorkDelay>();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::{App, Update};
    use crate::layer1::pop::PopBundle;

    #[test]
    fn test_high_speech_causes_delay_for_uneducated_pops() {
        let mut app = App::new();
        app.insert_resource(AdministrationLevel(10));
        app.insert_resource(HighSpeechEnabled(true));
        app.add_systems(Update, translate_instruction_system);

        // Spawn uneducated pop
        let uneducated_pop = app.world_mut().spawn(
            PopBundle { education: EducationLevel(1), ..PopBundle::random(0, 0, &mut rand::thread_rng()) }
        ).id();

        app.update();

        // Uneducated pop should have a WorkDelay component added due to High Speech
        assert!(app.world().entity(uneducated_pop).contains::<WorkDelay>());
    }

    #[test]
    fn test_high_speech_no_delay_for_bureaucrats() {
        let mut app = App::new();
        app.insert_resource(AdministrationLevel(10));
        app.insert_resource(HighSpeechEnabled(true));
        app.add_systems(Update, translate_instruction_system);

        // Spawn bureaucrat pop
        let mut traits = Traits::default();
        traits.add(Trait::Bureaucrat);
        let bureaucrat_pop = app.world_mut().spawn(
            PopBundle { education: EducationLevel(5), traits, ..PopBundle::random(0, 0, &mut rand::thread_rng()) }
        ).id();

        app.update();

        // Bureaucrat should NOT have a WorkDelay component
        assert!(!app.world().entity(bureaucrat_pop).contains::<WorkDelay>());
    }

    #[test]
    fn test_high_speech_removed_when_disabled() {
        let mut app = App::new();
        app.insert_resource(AdministrationLevel(10));
        app.insert_resource(HighSpeechEnabled(true));
        app.add_systems(Update, translate_instruction_system);

        let pop = app.world_mut().spawn(
            PopBundle { education: EducationLevel(1), ..PopBundle::random(0, 0, &mut rand::thread_rng()) }
        ).id();

        app.update();
        assert!(app.world().entity(pop).contains::<WorkDelay>());

        // Disable high speech
        app.world_mut().resource_mut::<HighSpeechEnabled>().0 = false;
        app.update();

        // Delay should be removed
        assert!(!app.world().entity(pop).contains::<WorkDelay>());
    }
}

use crate::layer1::pop::EducationLevel;
use crate::layer1::traits::{Trait, Traits};
use bevy_ecs::prelude::*;

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
    high_speech: Option<Res<HighSpeechEnabled>>,
    query: Query<(Entity, &EducationLevel, Option<&Traits>, Has<WorkDelay>)>,
) {
    let high_speech_enabled = high_speech.is_some_and(|hs| hs.0);

    for (entity, education, traits, has_delay) in query.iter() {
        let is_bureaucrat = traits.is_some_and(|t: &Traits| t.has(Trait::Bureaucrat));

        let should_be_delayed = high_speech_enabled && !is_bureaucrat && education.0 < 4;

        if should_be_delayed && !has_delay {
            commands
                .entity(entity)
                .insert(WorkDelay { multiplier: 0.5 });
        } else if !should_be_delayed && has_delay {
            commands.entity(entity).remove::<WorkDelay>();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::pop::{EducationLevel, PopBundle};
    use crate::layer1::traits::{Trait, Traits};
    use bevy_app::App;

    #[test]
    fn test_high_speech_causes_delay_for_uneducated_pops() {
        let mut app = App::new();
        app.insert_resource(AdministrationLevel(10));
        app.insert_resource(HighSpeechEnabled(true));
        app.add_systems(bevy_app::Update, translate_instruction_system);

        // Spawn uneducated pop
        let uneducated_pop = app
            .world_mut()
            .spawn(PopBundle {
                education: EducationLevel(1),
                ..PopBundle::random(0, 0, &mut rand::thread_rng())
            })
            .id();

        app.update();

        // Uneducated pop should have a WorkDelay component added due to High Speech
        assert!(app.world().entity(uneducated_pop).contains::<WorkDelay>());
    }

    #[test]
    fn test_high_speech_no_delay_for_bureaucrats() {
        let mut app = App::new();
        app.insert_resource(AdministrationLevel(10));
        app.insert_resource(HighSpeechEnabled(true));
        app.add_systems(bevy_app::Update, translate_instruction_system);

        // Spawn bureaucrat pop
        let mut traits = Traits::default();
        traits.add(Trait::Bureaucrat);
        let bureaucrat_pop = app
            .world_mut()
            .spawn(PopBundle {
                education: EducationLevel(5),
                traits,
                ..PopBundle::random(0, 0, &mut rand::thread_rng())
            })
            .id();

        app.update();

        // Bureaucrat should NOT have a WorkDelay component
        assert!(!app.world().entity(bureaucrat_pop).contains::<WorkDelay>());
    }

    #[test]
    fn test_high_speech_delay_removed_when_disabled() {
        let mut app = App::new();
        app.insert_resource(AdministrationLevel(10));
        app.insert_resource(HighSpeechEnabled(true));
        app.add_systems(bevy_app::Update, translate_instruction_system);

        let pop = app
            .world_mut()
            .spawn(PopBundle {
                education: EducationLevel(1),
                ..PopBundle::random(0, 0, &mut rand::thread_rng())
            })
            .id();

        app.update();
        assert!(app.world().entity(pop).contains::<WorkDelay>());

        // Disable High Speech
        app.insert_resource(HighSpeechEnabled(false));
        app.update();

        assert!(!app.world().entity(pop).contains::<WorkDelay>());
    }

    #[test]
    fn test_high_speech_delay_removed_when_education_increases() {
        let mut app = App::new();
        app.insert_resource(AdministrationLevel(10));
        app.insert_resource(HighSpeechEnabled(true));
        app.add_systems(bevy_app::Update, translate_instruction_system);

        let pop = app
            .world_mut()
            .spawn(PopBundle {
                education: EducationLevel(1),
                ..PopBundle::random(0, 0, &mut rand::thread_rng())
            })
            .id();

        app.update();
        assert!(app.world().entity(pop).contains::<WorkDelay>());

        // Increase education
        app.world_mut().entity_mut(pop).insert(EducationLevel(5));
        app.update();

        assert!(!app.world().entity(pop).contains::<WorkDelay>());
    }
}

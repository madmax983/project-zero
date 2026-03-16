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

#[allow(clippy::type_complexity)]
pub fn translate_instruction_system(
    mut commands: Commands,
    high_speech: Option<Res<HighSpeechEnabled>>,
    query: Query<(Entity, Option<&EducationLevel>, Option<&Traits>), (With<crate::layer1::pop::Pop>, Without<WorkDelay>)>,
) {
    if !high_speech.is_some_and(|hs| hs.0) {
        return;
    }

    for (entity, education, traits) in query.iter() {
        let is_bureaucrat = traits.is_some_and(|t| t.has(Trait::Bureaucrat));
        let ed_level = education.map_or(0, |e| e.0);

        if !is_bureaucrat && ed_level < 4 {
            commands.entity(entity).insert(WorkDelay { multiplier: 0.5 });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::pop::PopBundle;

    #[test]
    fn test_high_speech_causes_delay_for_uneducated_pops() {
        let mut world = World::new();
        world.insert_resource(AdministrationLevel(10));
        world.insert_resource(HighSpeechEnabled(true));

        let mut schedule = Schedule::default();
        schedule.add_systems(translate_instruction_system);

        // Spawn uneducated pop
        let mut bundle = PopBundle::random(0, 0, &mut rand::thread_rng());
        bundle.education = EducationLevel(1);
        bundle.traits.remove(Trait::Bureaucrat);
        let uneducated_pop = world.spawn(bundle).id();

        schedule.run(&mut world);

        // Uneducated pop should have a WorkDelay component added due to High Speech
        assert!(world.entity(uneducated_pop).contains::<WorkDelay>());
    }

    #[test]
    fn test_high_speech_no_delay_for_bureaucrats() {
        let mut world = World::new();
        world.insert_resource(AdministrationLevel(10));
        world.insert_resource(HighSpeechEnabled(true));

        let mut schedule = Schedule::default();
        schedule.add_systems(translate_instruction_system);

        // Spawn bureaucrat pop
        let mut bundle = PopBundle::random(0, 0, &mut rand::thread_rng());
        bundle.education = EducationLevel(5);
        bundle.traits.add(Trait::Bureaucrat);
        let bureaucrat_pop = world.spawn(bundle).id();

        schedule.run(&mut world);

        // Bureaucrat should NOT have a WorkDelay component
        assert!(!world.entity(bureaucrat_pop).contains::<WorkDelay>());
    }
}

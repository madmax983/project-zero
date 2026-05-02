use bevy_ecs::prelude::*;

use crate::layer1::beauty::BeautySource;
use crate::layer1::biology::health::DamageResistance;
use crate::layer1::culture::art::Art;
use crate::layer1::culture::artifacts::Artifact;
use crate::layer1::execution::components::MovementTarget;
use crate::layer1::needs::Needs;
use crate::layer1::pop::{Pop, PopName, Speed};
use crate::layer1::utility_types::ActionType;

#[derive(Component)]
pub struct ExoticDeepCrust;

#[derive(Component)]
pub struct PetrificationSickness {
    pub stage: u32,
    pub max_stage: u32,
}

#[allow(clippy::type_complexity)]
pub fn petrification_exposure_system(
    mut commands: Commands,
    query: Query<(Entity, &MovementTarget), (With<Pop>, Without<PetrificationSickness>)>,
    target_query: Query<&ExoticDeepCrust>,
) {
    for (entity, target) in query.iter() {
        if target.for_action == ActionType::Work && target_query.get(target.target_entity).is_ok() {
            commands.entity(entity).insert((
                PetrificationSickness {
                    stage: 1,
                    max_stage: 100,
                },
                DamageResistance::default(),
            ));
        }
    }
}

pub fn petrification_progression_system(
    mut query: Query<(
        &mut PetrificationSickness,
        &mut Speed,
        Option<&mut DamageResistance>,
    )>,
) {
    for (mut sickness, mut speed, resistance_opt) in query.iter_mut() {
        if sickness.stage < sickness.max_stage {
            sickness.stage += 1;

            let progress = sickness.stage as f32 / sickness.max_stage as f32;
            speed.current = speed.base * (1.0 - (0.8 * progress));
            if let Some(mut r) = resistance_opt {
                r.physical = 50.0 * progress;
            }
        }
    }
}

pub fn petrification_transformation_system(
    mut commands: Commands,
    query: Query<(Entity, &PetrificationSickness, Option<&PopName>), With<Pop>>,
) {
    for (entity, sickness, pop_name_opt) in query.iter() {
        if sickness.stage >= sickness.max_stage {
            let name = pop_name_opt.map_or("Unknown Colonist".to_string(), |n| n.0.clone());
            let description = format!("Petrified Colonist: {}", name);

            commands
                .entity(entity)
                .remove::<Pop>()
                .remove::<Speed>()
                .remove::<Needs>()
                .insert((
                    Artifact,
                    Art { description },
                    BeautySource {
                        value: 75.0,
                        radius: 0.0,
                    },
                ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::pop::Pop;
    use bevy_app::{App, Update};

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_systems(
            Update,
            (
                petrification_exposure_system,
                petrification_progression_system,
                petrification_transformation_system,
            ),
        );
        app
    }

    #[test]
    fn test_mining_exotic_ore_causes_petrification_exposure() {
        let mut app = setup_app();

        let exotic_mine = app.world_mut().spawn((ExoticDeepCrust,)).id();

        let pop = app
            .world_mut()
            .spawn((
                Pop,
                MovementTarget {
                    target_entity: exotic_mine,
                    target_position: crate::layer1::map::GridPosition { x: 0, y: 0 },
                    for_action: ActionType::Work,
                },
            ))
            .id();

        app.update();

        let exposure = app.world().get::<PetrificationSickness>(pop);
        assert!(exposure.is_some());
        assert_eq!(exposure.unwrap().stage, 1);
    }

    #[test]
    fn test_petrification_progresses_and_alters_stats() {
        let mut app = setup_app();

        let pop = app
            .world_mut()
            .spawn((
                Pop,
                Speed {
                    base: 1.0,
                    current: 1.0,
                    accumulator: 0.0,
                },
                Needs {
                    hunger: 100.0,
                    rest: 100.0,
                    ..Default::default()
                },
                DamageResistance {
                    physical: 0.0,
                },
                PetrificationSickness {
                    stage: 50,
                    max_stage: 100,
                },
            ))
            .id();

        app.update();

        let sickness = app.world().get::<PetrificationSickness>(pop).unwrap();
        assert!(sickness.stage > 50);

        let speed = app.world().get::<Speed>(pop).unwrap();
        assert!(
            speed.current < 1.0,
            "Movement speed should decrease as pop petrifies"
        );

        let resistance = app.world().get::<DamageResistance>(pop).unwrap();
        assert!(
            resistance.physical > 0.0,
            "Physical resistance should increase as pop turns to stone"
        );
    }

    #[test]
    fn test_full_petrification_transforms_pop_into_artifact() {
        let mut app = setup_app();

        let pop = app
            .world_mut()
            .spawn((
                Pop,
                PopName("Jim".to_string()),
                PetrificationSickness {
                    stage: 100, // Should trigger at >= max_stage
                    max_stage: 100,
                },
            ))
            .id();

        app.update();

        // Pop should no longer have the Pop component
        assert!(app.world().get::<Pop>(pop).is_none());

        // Pop should now have the Artifact component with high value
        let artifact = app.world().get::<Artifact>(pop);
        assert!(artifact.is_some());

        let art = app.world().get::<Art>(pop);
        assert!(art.is_some());
        assert_eq!(art.unwrap().description, "Petrified Colonist: Jim");

        let beauty = app.world().get::<BeautySource>(pop);
        assert!(beauty.is_some());
        assert_eq!(beauty.unwrap().value, 75.0);
    }
}

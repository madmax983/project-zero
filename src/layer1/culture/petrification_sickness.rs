use bevy_ecs::prelude::*;
use crate::layer1::pop::{Pop, Speed, PopName};
use crate::layer1::needs::Needs;
use crate::layer1::beauty::BeautySource;
use crate::layer1::artifacts::Artifact;
use crate::layer1::deep_crust_resonance::ExcavationEvent;
use crate::layer1::map::GridPosition;


use crate::layer1::biology::health::DamageResistance;


#[derive(Component)]
pub struct PetrificationSickness {
    pub stage: u32,
    pub max_stage: u32,
}

pub fn petrification_exposure_system(
    mut commands: Commands,
    mut events: EventReader<ExcavationEvent>,
    query: Query<(), (With<Pop>, Without<PetrificationSickness>)>,
) {
    for event in events.read() {
        if event.discovery_type == "ResonantOre" && query.contains(event.miner) {
            commands.entity(event.miner).insert((
                PetrificationSickness {
                    stage: 1,
                    max_stage: 100,
                },
                DamageResistance::default()
            ));
        }
    }
}

pub fn petrification_progression_system(
    mut query: Query<(
        &mut PetrificationSickness,
        Option<&mut Speed>,
        Option<&mut DamageResistance>
    )>
) {
    for (mut sickness, mut speed_opt, mut resistance_opt) in query.iter_mut() {
        if sickness.stage < sickness.max_stage {
            sickness.stage += 1;

            let progress = sickness.stage as f32 / sickness.max_stage as f32;

            if let Some(ref mut speed) = speed_opt {
                speed.current = speed.base * (1.0 - (0.8 * progress));
            }

            if let Some(ref mut resistance) = resistance_opt {
                resistance.physical = 50.0 * progress;
            }
        }
    }
}

pub fn petrification_transformation_system(
    mut commands: Commands,
    query: Query<(Entity, &PetrificationSickness, Option<&GridPosition>), With<Pop>>
) {
    for (entity, sickness, pos_opt) in query.iter() {
        if sickness.stage >= sickness.max_stage {
            let mut entity_cmds = commands.entity(entity);
            entity_cmds
                .remove::<Pop>()
                .remove::<Speed>()
                .remove::<Needs>()
                .insert(Artifact)
                .insert(PopName("Petrified Colonist".to_string()))
                .insert(BeautySource {
                    value: 75.0,
                    radius: 2.0,
                });

            if pos_opt.is_none() {
                 entity_cmds.insert(GridPosition { x: 0, y: 0 }); // Fallback for artifact rendering if needed
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::time::SimulationTime;
    use crate::layer1::map::GridPosition;

    fn setup_world() -> World {
        let mut world = World::new();
        world.insert_resource(Events::<ExcavationEvent>::default());
        world.insert_resource(SimulationTime::default());
        world
    }

    #[test]
    fn test_mining_resonant_ore_causes_petrification_exposure() {
        let mut world = setup_world();

        let miner = world.spawn((Pop, )).id();

        world.resource_mut::<Events<ExcavationEvent>>().send(ExcavationEvent {
            colony: Entity::PLACEHOLDER,
            miner,
            discovery_type: "ResonantOre".to_string(),
            target: Entity::PLACEHOLDER,
        });

        let mut schedule = bevy_ecs::schedule::Schedule::default();
        schedule.add_systems(petrification_exposure_system);
        schedule.run(&mut world);

        let exposure = world.get::<PetrificationSickness>(miner);
        assert!(exposure.is_some());
        assert_eq!(exposure.unwrap().stage, 1);

        assert!(world.get::<DamageResistance>(miner).is_some());
    }

    #[test]
    fn test_petrification_progresses_and_alters_stats() {
        let mut world = setup_world();

        let pop = world.spawn((
            Pop,
            Speed { base: 1.0, current: 1.0, accumulator: 0.0 },
            Needs::default(),
            DamageResistance::default(),
            PetrificationSickness { stage: 50, max_stage: 100 }
        )).id();

        let mut schedule = bevy_ecs::schedule::Schedule::default();
        schedule.add_systems(petrification_progression_system);
        schedule.run(&mut world);

        let sickness = world.get::<PetrificationSickness>(pop).unwrap();
        assert!(sickness.stage > 50);

        let speed = world.get::<Speed>(pop).unwrap();
        assert!(speed.current < 1.0, "Movement speed should decrease as pop petrifies");

        let resistance = world.get::<DamageResistance>(pop).unwrap();
        assert!(resistance.physical > 0.0, "Physical resistance should increase as pop turns to stone");
    }

    #[test]
    fn test_full_petrification_transforms_pop_into_artifact() {
        let mut world = setup_world();

        let pop = world.spawn((
            Pop,
            GridPosition { x: 5, y: 5 },
            PetrificationSickness { stage: 99, max_stage: 100 }
        )).id();

        let mut schedule = bevy_ecs::schedule::Schedule::default();
        schedule.add_systems((
            petrification_progression_system,
            petrification_transformation_system
        ).chain());
        schedule.run(&mut world);

        assert!(world.get::<Pop>(pop).is_none());

        let artifact = world.get::<Artifact>(pop);
        assert!(artifact.is_some());

        let name = world.get::<PopName>(pop);
        assert!(name.is_some());
        assert_eq!(name.unwrap().0, "Petrified Colonist");

        let beauty = world.get::<BeautySource>(pop);
        assert!(beauty.is_some());
        assert!(beauty.unwrap().value > 50.0);
    }
}

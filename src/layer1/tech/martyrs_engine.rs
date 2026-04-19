// src/layer1/tech/martyrs_engine.rs

use crate::layer1::chronicle::{AddChronicleEvent, EventImportance};
use crate::layer1::energy::PowerSource;
use crate::layer1::lighting::LightSource;
use crate::layer1::pop::PopName;
use crate::layer1::traits::{Trait, Traits};
use crate::layer1::StressTracker;
use crate::layer2::shielding::OrbitalShield;
use bevy::prelude::DespawnRecursiveExt;
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct MartyrsEngine {
    pub ticks_remaining: u32,
}

#[derive(Component)]
pub struct AttuneEngineAction {
    pub target_engine: Entity,
    pub pop: Entity,
}

// Constants
const ATTUNE_DURATION_TICKS: u32 = 1000; // E.g., one year
const ENGINE_POWER_OUTPUT: f32 = 10000.0;
const ENGINE_SHIELD_CAPACITY: f32 = 5000.0;
const SACRIFICE_STRESS_PENALTY: f32 = 40.0;

pub fn attune_engine_system(
    mut commands: Commands,
    action_query: Query<(Entity, &AttuneEngineAction)>,
    mut engine_query: Query<&mut MartyrsEngine>,
    mut stress_query: Query<(&mut StressTracker, Option<&Traits>)>,
    pop_name_query: Query<&PopName>,
    mut chronicle_events: EventWriter<AddChronicleEvent>,
) {
    for (action_entity, action) in action_query.iter() {
        // Activate Engine
        if let Ok(mut engine) = engine_query.get_mut(action.target_engine) {
            engine.ticks_remaining = ATTUNE_DURATION_TICKS;
        }

        let name = if let Ok(pop_name) = pop_name_query.get(action.pop) {
            pop_name.0.clone()
        } else {
            "An unknown colonist".to_string()
        };

        chronicle_events.send(AddChronicleEvent {
            text: format!("{} was sacrificed to the Martyr's Engine.", name),
            importance: EventImportance::Legendary,
        });

        // Consume Pop
        commands.entity(action.pop).despawn_recursive();

        // Apply Stress to everyone else, modified by traits
        for (mut stress, traits_opt) in stress_query.iter_mut() {
            let mut penalty = SACRIFICE_STRESS_PENALTY;
            if let Some(traits) = traits_opt {
                if traits.has(Trait::Cannibal) || traits.has(Trait::Outsider) {
                    penalty *= 0.1; // Much less stress
                }
                if traits.has(Trait::EmpathicLink) || traits.has(Trait::Compassionate) {
                    penalty *= 1.5; // More stress
                }
            }
            stress.accumulated_stress += penalty;
        }

        // Cleanup Action
        commands.entity(action_entity).despawn();
    }
}

pub fn process_martyrs_engine(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &mut MartyrsEngine,
        &mut PowerSource,
        &mut OrbitalShield,
        Option<&LightSource>,
    )>,
) {
    for (entity, mut engine, mut power, mut shield, light_opt) in query.iter_mut() {
        if engine.ticks_remaining > 0 {
            engine.ticks_remaining -= 1;
            power.output = ENGINE_POWER_OUTPUT;
            shield.capacity = ENGINE_SHIELD_CAPACITY;

            // Add or update eerie glow
            if light_opt.is_none() {
                commands.entity(entity).insert(LightSource {
                    radius: 20.0,
                    intensity: 1.5,
                    color: (255, 50, 50), // Eerie red glow
                    is_outdoor: false,
                });
            }
        } else {
            power.output = 0.0;
            shield.capacity = 0.0;
            if light_opt.is_some() {
                commands.entity(entity).remove::<LightSource>();
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use crate::layer1::chronicle::AddChronicleEvent;
    use crate::layer1::energy::PowerSource;
    use crate::layer1::pop::PopBundle;
    use crate::layer1::tech::martyrs_engine::{
        attune_engine_system, process_martyrs_engine, AttuneEngineAction, MartyrsEngine,
    };
    use crate::layer1::StressTracker;
    use crate::layer2::shielding::OrbitalShield;
    use bevy_ecs::prelude::*;
    use rand::thread_rng;

    #[test]
    fn test_martyrs_engine_inactive_produces_nothing() {
        let mut world = World::new();
        let engine = world
            .spawn((
                MartyrsEngine { ticks_remaining: 0 },
                PowerSource {
                    output: 0.0,
                    ..Default::default()
                },
                OrbitalShield {
                    capacity: 0.0,
                    ..Default::default()
                },
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(process_martyrs_engine);
        schedule.run(&mut world);

        let power = world.get::<PowerSource>(engine).expect("Component should exist or System should run");
        assert_eq!(power.output, 0.0);
    }

    #[test]
    fn test_attuning_engine_consumes_pop_and_activates() {
        let mut world = World::new();
        world.insert_resource(Events::<AddChronicleEvent>::default());
        let engine = world
            .spawn((
                MartyrsEngine { ticks_remaining: 0 },
                PowerSource {
                    output: 0.0,
                    ..Default::default()
                },
                OrbitalShield {
                    capacity: 0.0,
                    ..Default::default()
                },
            ))
            .id();

        let mut rng = thread_rng();
        let pop = world.spawn(PopBundle::random(0, 0, &mut rng)).id();

        world.spawn(AttuneEngineAction {
            target_engine: engine,
            pop,
        });

        let mut schedule = Schedule::default();
        schedule.add_systems(attune_engine_system);
        schedule.run(&mut world);

        // Pop is consumed
        assert!(world.get_entity(pop).is_err());

        // Engine is active
        let engine_comp = world.get::<MartyrsEngine>(engine).expect("Component should exist or System should run");
        assert!(engine_comp.ticks_remaining > 0);
    }

    #[test]
    fn test_active_engine_produces_power_and_decays() {
        let mut world = World::new();
        let engine = world
            .spawn((
                MartyrsEngine {
                    ticks_remaining: 10,
                },
                PowerSource {
                    output: 0.0,
                    ..Default::default()
                },
                OrbitalShield {
                    capacity: 0.0,
                    ..Default::default()
                },
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(process_martyrs_engine);
        schedule.run(&mut world);

        let engine_comp = world.get::<MartyrsEngine>(engine).expect("Component should exist or System should run");
        assert_eq!(engine_comp.ticks_remaining, 9); // Decays

        let power = world.get::<PowerSource>(engine).expect("Component should exist or System should run");
        assert!(power.output > 0.0); // Producing power
    }

    #[test]
    fn test_attuning_causes_colony_stress() {
        let mut world = World::new();
        world.insert_resource(Events::<AddChronicleEvent>::default());
        let engine = world.spawn(MartyrsEngine { ticks_remaining: 0 }).id();
        let mut rng = thread_rng();
        let pop = world.spawn(PopBundle::random(0, 0, &mut rng)).id();
        let bystander = world
            .spawn(StressTracker {
                accumulated_stress: 0.0,
            })
            .id();

        world.spawn(AttuneEngineAction {
            target_engine: engine,
            pop,
        });

        let mut schedule = Schedule::default();
        schedule.add_systems(attune_engine_system);
        schedule.run(&mut world);

        let bystander_stress = world.get::<StressTracker>(bystander).expect("Component should exist or System should run");
        assert!(bystander_stress.accumulated_stress > 0.0);
    }
}

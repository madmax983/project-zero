//! Fading Homeworld
//!
//! Tracks the decay of the original homeworld's influence over time.
//! As colonies grow autonomous, the homeworld's edicts carry less weight, leading to inevitable independence.

use crate::layer1::economy::resources::ColonyResources;
use crate::layer3::diplomacy_reflection::DiplomaticRelations;
use bevy::prelude::*;

#[derive(Component)]
pub struct CoreWorld {
    pub stability: f32,
    pub decay_rate: f32,
}

#[derive(Event, Clone)]
pub struct CoreWorldDemandEvent {
    pub demand: CoreWorldDemand,
}

#[derive(Clone)]
pub struct CoreWorldDemand {
    pub amount: f32,
    pub penalty: f32,
    pub faction: Entity,
}

pub fn update_core_world_decay_system(mut query: Query<&mut CoreWorld>) {
    for mut core_world in query.iter_mut() {
        core_world.stability -= core_world.decay_rate;
        if core_world.stability < 0.0 {
            core_world.stability = 0.0;
        }
    }
}

#[must_use]
pub fn calculate_demand_severity(stability: f32) -> f32 {
    if stability >= 100.0 {
        1.0
    } else {
        1.0 + ((100.0 - stability) / 100.0) * 2.0
    }
}

pub fn fulfill_demand(world: &mut World, demand: CoreWorldDemand) {
    let mut pool = world.resource_mut::<ColonyResources>();
    if pool.food >= demand.amount {
        pool.food -= demand.amount;
    }
}

pub fn refuse_demand(world: &mut World, demand: CoreWorldDemand) {
    if let Some(mut relation) = world.get_mut::<DiplomaticRelations>(demand.faction) {
        for rel in relation.relations.iter_mut() {
            rel.standing -= demand.penalty;
        }
    }
}

pub fn generate_core_world_demand_system(
    mut events: EventWriter<CoreWorldDemandEvent>,
    query: Query<(Entity, &CoreWorld)>,
) {
    for (entity, core_world) in query.iter() {
        if core_world.stability < 20.0 {
            events.send(CoreWorldDemandEvent {
                demand: CoreWorldDemand {
                    amount: 200.0,
                    penalty: 20.0,
                    faction: entity,
                },
            });
        }
    }
}

#[derive(Event, Clone)]
pub struct PlayerDemandResponse {
    pub demand: CoreWorldDemand,
    pub accept: bool,
}

pub fn handle_core_world_demands_system(world: &mut World) {
    let mut response_events = world.resource_mut::<Events<PlayerDemandResponse>>();
    let events: Vec<_> = response_events.drain().collect();

    for response in events {
        if response.accept {
            fulfill_demand(world, response.demand);
        } else {
            refuse_demand(world, response.demand);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::layer1::economy::resources::ColonyResources;
    use crate::layer3::diplomacy_reflection::{
        Civilization, DiplomaticRelations, DiplomaticStanding,
    };
    use crate::shared::time::SimulationTime;

    #[test]
    fn test_core_world_decay_increases_demand_severity() {
        let mut app = App::new();
        app.init_resource::<SimulationTime>()
            .init_resource::<Events<CoreWorldDemandEvent>>();

        app.add_systems(Update, update_core_world_decay_system);

        let core_faction_id = app
            .world_mut()
            .spawn(CoreWorld {
                stability: 100.0,
                decay_rate: 1.0,
            })
            .id();

        for _ in 0..10 {
            app.world_mut().resource_mut::<SimulationTime>().tick += 1;
            app.update();
        }

        let core_world = app.world().get::<CoreWorld>(core_faction_id).unwrap();
        assert!(core_world.stability < 100.0);

        let demand_severity = calculate_demand_severity(core_world.stability);
        assert!(demand_severity > 1.0);
    }

    #[test]
    fn test_fulfilling_demand_depletes_resources_and_maintains_standing() {
        let mut app = App::new();
        app.insert_resource(ColonyResources {
            food: 500.0,
            ..Default::default()
        });

        let core_faction = app
            .world_mut()
            .spawn((
                Civilization {
                    id: "Motherland".to_string(),
                },
                DiplomaticRelations {
                    relations: vec![DiplomaticStanding {
                        target_id: "Colony".to_string(),
                        standing: 50.0,
                        sanctioned: false,
                    }],
                },
            ))
            .id();

        app.add_systems(Update, handle_core_world_demands_system);

        let demand = CoreWorldDemand {
            amount: 200.0,
            penalty: 20.0,
            faction: core_faction,
        };

        fulfill_demand(app.world_mut(), demand);

        let pool = app.world().resource::<ColonyResources>();
        assert!((pool.food - 300.0).abs() < f32::EPSILON);

        let relation = app
            .world()
            .get::<DiplomaticRelations>(core_faction)
            .unwrap();
        assert!((relation.relations[0].standing - 50.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_refusing_demand_decreases_standing() {
        let mut app = App::new();
        let core_faction = app
            .world_mut()
            .spawn((
                Civilization {
                    id: "Motherland".to_string(),
                },
                DiplomaticRelations {
                    relations: vec![DiplomaticStanding {
                        target_id: "Colony".to_string(),
                        standing: 50.0,
                        sanctioned: false,
                    }],
                },
            ))
            .id();

        let demand = CoreWorldDemand {
            amount: 200.0,
            penalty: 20.0,
            faction: core_faction,
        };

        refuse_demand(app.world_mut(), demand);

        let relation = app
            .world()
            .get::<DiplomaticRelations>(core_faction)
            .unwrap();
        assert!((relation.relations[0].standing - 30.0).abs() < f32::EPSILON);
    }
}

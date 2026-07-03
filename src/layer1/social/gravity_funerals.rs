use bevy_ecs::prelude::*;
use bevy_app::prelude::*;
use crate::layer1::entities::pop::{PopDied, StatusLevel};
use crate::layer1::economy::resources::ColonyResources;
use crate::layer1::social::inherited_grudges::Lineage;
use crate::layer1::social::unrest::Unrest;

#[derive(Event, Clone)]
pub struct SunBurialRequestEvent {
    pub deceased: Entity,
    pub fuel_cost: f32,
}


#[derive(Resource, Default)]

pub struct GravityFuneralsPlugin;

impl Plugin for GravityFuneralsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SunBurialRequestEvent>();
        app.init_resource::<SunBurialQueue>();
        app.add_systems(
            Update,
            (
                process_high_status_deaths_system,
                evaluate_sun_burial_requests_system,
            ),
        );
    }
}

#[derive(Resource, Default)]
pub struct SunBurialQueue {
    pub requests: Vec<SunBurialRequestEvent>,
}

pub fn process_high_status_deaths_system(
    query: Query<&StatusLevel>,
    mut death_events: EventReader<PopDied>,
    mut burial_requests: EventWriter<SunBurialRequestEvent>,
) {
    for event in death_events.read() {
        if let Ok(status) = query.get(event.entity) {
            if *status == StatusLevel::Elite {
                burial_requests.send(SunBurialRequestEvent {
                    deceased: event.entity,
                    fuel_cost: 500.0,
                });
            }
        }
    }
}

pub fn evaluate_sun_burial_requests_system(
    mut burial_requests: EventReader<SunBurialRequestEvent>,
    mut queue: Option<ResMut<SunBurialQueue>>,
    mut resources: ResMut<ColonyResources>,
    lineage_query: Query<(Entity, &Lineage)>,
    mut global_unrest: ResMut<Unrest>,
) {
    let mut new_requests = Vec::new();
    for request in burial_requests.read() {
        new_requests.push(SunBurialRequestEvent {
            deceased: request.deceased,
            fuel_cost: request.fuel_cost,
        });
    }

    if let Some(ref mut q) = queue {
        q.requests.append(&mut new_requests);
    }

    let mut remaining_requests = Vec::new();

    if let Some(ref mut q) = queue {
        for request in q.requests.drain(..) {
            if resources.fuel >= request.fuel_cost {
                resources.fuel -= request.fuel_cost;
            } else {
                remaining_requests.push(request);
            }
        }
        q.requests = remaining_requests.clone();
    } else {
        for request in new_requests.drain(..) {
            if resources.fuel >= request.fuel_cost {
                resources.fuel -= request.fuel_cost;
            } else {
                remaining_requests.push(request);
            }
        }
    }

    // Apply slow unrest build up for backlogged requests using a recursive lineage tree
    for request in remaining_requests {
        let mut to_process = vec![request.deceased];

        while let Some(current_ancestor) = to_process.pop() {
            for (entity, lineage) in lineage_query.iter() {
                if lineage.parent_entity == Some(current_ancestor) {
                    global_unrest.level += 0.05; // Slow buildup over time while in queue
                    to_process.push(entity);
                }
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::prelude::*;
    use crate::layer1::entities::pop::{PopDied, StatusLevel};
    use crate::layer1::economy::resources::ColonyResources;
    use crate::layer1::social::inherited_grudges::Lineage;
    use crate::layer1::social::unrest::Unrest;

    #[test]
    fn test_high_status_death_triggers_sun_burial_request() {
        let mut app = App::new();
        app.add_event::<PopDied>();
        app.add_event::<SunBurialRequestEvent>();


        app.add_systems(Update, process_high_status_deaths_system);

        let high_status_pop = app.world_mut().spawn((
            Pop,
            StatusLevel::Elite,
        )).id();

        app.world_mut().resource_mut::<Events<PopDied>>().send(PopDied {
            entity: high_status_pop,
            name: "Bob".to_string(),
            tick: 0,
            reason: "Old Age".to_string(),
        });

        app.update();

        let requests = app.world().resource::<Events<SunBurialRequestEvent>>();
        let mut reader = requests.get_cursor();
        let mut found = false;
        for event in reader.read(requests) {
            if event.deceased == high_status_pop {
                found = true;
            }
        }

        assert!(found, "A high-status pop death should trigger a Sun Burial request.");
    }


    #[test]
    fn test_denied_sun_burial_causes_lineage_unrest() {
        let mut app = App::new();
        app.add_event::<SunBurialRequestEvent>();
        let mut res = ColonyResources::default();
        res.fuel = 0.0;
        app.insert_resource(res);
        app.insert_resource(Unrest::default());
        app.init_resource::<SunBurialQueue>();

        app.add_systems(Update, evaluate_sun_burial_requests_system);

        let deceased = app.world_mut().spawn(Pop).id();

        let _child1 = app.world_mut().spawn((Pop, Lineage { parent_entity: Some(deceased) })).id();
        let _child2 = app.world_mut().spawn((Pop, Lineage { parent_entity: Some(deceased) })).id();
        let _grandchild = app.world_mut().spawn((Pop, Lineage { parent_entity: Some(_child1) })).id();

        app.world_mut().resource_mut::<Events<SunBurialRequestEvent>>().send(SunBurialRequestEvent {
            deceased,
            fuel_cost: 500.0,
        });

        app.update();

        assert!(app.world().resource::<Unrest>().level > 0.0, "Denied sun burial should increase global unrest due to surviving lineage.");

        let q = app.world().resource::<SunBurialQueue>();
        assert_eq!(q.requests.len(), 1, "Should backlog");
    }

    #[test]
    fn test_fulfilled_sun_burial_consumes_fuel() {
        let mut app = App::new();
        app.add_event::<SunBurialRequestEvent>();
        let mut res = ColonyResources::default();
        res.fuel = 1000.0;
        app.insert_resource(res);
        app.insert_resource(Unrest::default());


        app.add_systems(Update, evaluate_sun_burial_requests_system);

        let deceased = app.world_mut().spawn(Pop).id();

        app.world_mut().resource_mut::<Events<SunBurialRequestEvent>>().send(SunBurialRequestEvent {
            deceased,
            fuel_cost: 500.0,
        });

        app.update();

        let res = app.world().resource::<ColonyResources>();
        assert_eq!(res.fuel, 500.0, "Fulfilled sun burial should consume fuel.");
    }
}

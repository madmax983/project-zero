use crate::layer1::economy::resources::ColonyResources;
use crate::layer1::entities::pop::PopDied;
use crate::layer1::social::inherited_grudges::Lineage;
use crate::layer1::social::social_stratification::SocialClass;
use crate::layer1::social::unrest::{Unrest, UnrestModifier};
use bevy::prelude::*;

#[derive(Event)]
pub struct SunBurialRequestEvent {
    pub deceased: Entity,
    pub fuel_cost: f32,
}

pub fn process_high_status_deaths_system(
    query: Query<&SocialClass>,
    mut death_events: EventReader<PopDied>,
    mut burial_requests: EventWriter<SunBurialRequestEvent>,
) {
    for event in death_events.read() {
        if let Ok(status) = query.get(event.entity) {
            if *status == SocialClass::Elite {
                burial_requests.send(SunBurialRequestEvent {
                    deceased: event.entity,
                    fuel_cost: 500.0, // MVP cost
                });
            }
        }
    }
}

pub fn evaluate_sun_burial_requests_system(
    mut burial_requests: EventReader<SunBurialRequestEvent>,
    mut resources: ResMut<ColonyResources>,
    mut unrest: ResMut<Unrest>,
    lineage_query: Query<&Lineage>,
) {
    for request in burial_requests.read() {
        if resources.fuel >= request.fuel_cost {
            // Fulfill
            resources.fuel -= request.fuel_cost;
        } else {
            // Deny: Punish the lineage by adding global unrest proportional to affected lineage
            let mut affected_family_members = 0;
            for lineage in lineage_query.iter() {
                if lineage.parent_entity == Some(request.deceased) {
                    affected_family_members += 1;
                }
            }
            if affected_family_members > 0 {
                unrest.modifiers.push(UnrestModifier {
                    value: 0.1 * affected_family_members as f32, // Massive penalty scaled by family size
                    duration: 100,                               // Lingers for a while
                    label: "Denied Sun Burial".to_string(),
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::economy::resources::ColonyResources;
    use crate::layer1::entities::pop::{Pop, PopDied};
    use crate::layer1::social::inherited_grudges::Lineage;
    use crate::layer1::social::social_stratification::SocialClass;
    use crate::layer1::social::unrest::Unrest;

    #[test]
    fn test_high_status_death_triggers_sun_burial_request() {
        let mut app = App::new();
        app.add_event::<PopDied>();
        app.add_event::<SunBurialRequestEvent>();
        app.add_systems(Update, process_high_status_deaths_system);

        let high_status_pop = app.world_mut().spawn((Pop, SocialClass::Elite)).id();

        app.world_mut()
            .resource_mut::<Events<PopDied>>()
            .send(PopDied {
                entity: high_status_pop,
                name: "Test Elite".to_string(),
                tick: 0,
                reason: "Old Age".to_string(),
            });

        app.update();

        // Verify request generated
        let requests = app.world().resource::<Events<SunBurialRequestEvent>>();
        let mut reader = requests.get_cursor();
        let mut found = false;
        for event in reader.read(requests) {
            if event.deceased == high_status_pop {
                found = true;
            }
        }

        assert!(
            found,
            "A high-status pop death should trigger a Sun Burial request."
        );
    }

    #[test]
    fn test_denied_sun_burial_causes_lineage_unrest() {
        let mut app = App::new();
        app.add_event::<SunBurialRequestEvent>();

        let mut res = ColonyResources::default();
        res.fuel = 0.0;
        app.insert_resource(res); // No fuel available
        app.insert_resource(Unrest::default());
        app.add_systems(Update, evaluate_sun_burial_requests_system);

        let deceased = app.world_mut().spawn(Pop).id();

        // Spawn family members
        app.world_mut().spawn((
            Pop,
            Lineage {
                parent_entity: Some(deceased),
            },
        ));
        app.world_mut().spawn((
            Pop,
            Lineage {
                parent_entity: Some(deceased),
            },
        ));

        app.world_mut()
            .resource_mut::<Events<SunBurialRequestEvent>>()
            .send(SunBurialRequestEvent {
                deceased,
                fuel_cost: 500.0,
            });

        app.update();

        // Fuel is 0, so the request is denied. Check global unrest modifier.
        let unrest = app.world().resource::<Unrest>();
        let has_modifier = unrest
            .modifiers
            .iter()
            .any(|m| m.label == "Denied Sun Burial" && m.value > 0.0);
        assert!(
            has_modifier,
            "Denied sun burial should increase global unrest based on surviving lineage."
        );
    }

    #[test]
    fn test_fulfilled_sun_burial_consumes_fuel() {
        let mut app = App::new();
        app.add_event::<SunBurialRequestEvent>();
        let mut res = ColonyResources::default();
        res.fuel = 1000.0;
        app.insert_resource(res); // Plenty of fuel
        app.insert_resource(Unrest::default());
        app.add_systems(Update, evaluate_sun_burial_requests_system);

        let deceased = app.world_mut().spawn(Pop).id();

        app.world_mut()
            .resource_mut::<Events<SunBurialRequestEvent>>()
            .send(SunBurialRequestEvent {
                deceased,
                fuel_cost: 500.0,
            });

        app.update();

        let resources = app.world().resource::<ColonyResources>();
        assert_eq!(
            resources.fuel, 500.0,
            "Fulfilled sun burial should consume fuel."
        );
    }
}

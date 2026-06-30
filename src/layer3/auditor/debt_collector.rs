use crate::layer1::architecture::building::{Building, BuildingType};
use crate::layer1::architecture::structure::Structure;
use crate::layer1::resources::{ColonyResources, ResourceType};
use bevy_ecs::prelude::*;

#[derive(Event, Clone, Debug)]
pub struct AuditorArrivalEvent;

#[derive(Event, Clone, Debug)]
pub struct AuditorDemandEvent {
    pub amount: f32,
    pub resource: ResourceType,
}

#[derive(Event, Clone, Debug)]
pub struct AuditorRefusalEvent;

pub fn process_auditor_demand_system(
    mut events: EventReader<AuditorDemandEvent>,
    mut resources: ResMut<ColonyResources>,
    mut refusal_writer: EventWriter<AuditorRefusalEvent>,
) {
    for ev in events.read() {
        if resources.get_amount(ev.resource) >= ev.amount {
            resources.try_consume(ev.resource, ev.amount);
            // Paid successfully
        } else {
            refusal_writer.send(AuditorRefusalEvent);
        }
    }
}

pub fn execute_auditor_strike_system(
    mut events: EventReader<AuditorRefusalEvent>,
    mut query: Query<(&Building, &mut Structure)>,
) {
    for _ in events.read() {
        for (building, mut structure) in query.iter_mut() {
            if matches!(
                building.building_type,
                BuildingType::Tavern | BuildingType::Statue | BuildingType::FlowerBed
            ) {
                structure.current_hp = 0.0;
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::App;

    fn setup_app() -> App {
        let mut app = App::new();
        let mut res = ColonyResources::zeroed();
        res.max_scrap = 10000.0; // Needs space to add scrap
        app.insert_resource(res);
        app.add_event::<AuditorArrivalEvent>();
        app.add_event::<AuditorDemandEvent>();
        app.add_event::<AuditorRefusalEvent>();
        app.add_systems(
            bevy_app::Update,
            (process_auditor_demand_system, execute_auditor_strike_system),
        );
        app
    }

    #[test]
    fn test_auditor_demand_payment_success() {
        let mut app = setup_app();

        let mut resources = app.world_mut().resource_mut::<ColonyResources>();
        resources.add_scrap(5000.0);

        app.world_mut()
            .resource_mut::<Events<AuditorDemandEvent>>()
            .send(AuditorDemandEvent {
                amount: 5000.0,
                resource: ResourceType::Scrap,
            });

        app.update();

        let resources = app.world().resource::<ColonyResources>();
        assert_eq!(resources.scrap, 0.0);
    }

    #[test]
    fn test_auditor_demand_payment_failure_triggers_strike() {
        let mut app = setup_app();

        let mut resources = app.world_mut().resource_mut::<ColonyResources>();
        resources.add_scrap(1000.0);

        app.world_mut()
            .resource_mut::<Events<AuditorDemandEvent>>()
            .send(AuditorDemandEvent {
                amount: 5000.0,
                resource: ResourceType::Scrap,
            });

        app.update();

        let resources = app.world().resource::<ColonyResources>();
        assert_eq!(resources.scrap, 1000.0);

        let refusal_events = app.world().resource::<Events<AuditorRefusalEvent>>();
        assert_eq!(refusal_events.len(), 1);
    }

    #[test]
    fn test_auditor_strike_destroys_luxury_building() {
        let mut app = setup_app();

        let tavern = app
            .world_mut()
            .spawn((
                Building {
                    building_type: BuildingType::Tavern,
                },
                Structure {
                    current_hp: 100.0,
                    max_hp: 100.0,
                },
            ))
            .id();

        app.world_mut()
            .resource_mut::<Events<AuditorRefusalEvent>>()
            .send(AuditorRefusalEvent);

        app.update();

        let structure = app.world().get::<Structure>(tavern).unwrap();
        assert!(structure.current_hp <= 0.0);
    }

    #[test]
    fn test_auditor_strike_ignores_military_building() {
        let mut app = setup_app();

        let hospital = app
            .world_mut()
            .spawn((
                Building {
                    building_type: BuildingType::Hospital,
                },
                Structure {
                    current_hp: 100.0,
                    max_hp: 100.0,
                },
            ))
            .id();

        app.world_mut()
            .resource_mut::<Events<AuditorRefusalEvent>>()
            .send(AuditorRefusalEvent);

        app.update();

        let structure = app.world().get::<Structure>(hospital).unwrap();
        assert_eq!(structure.current_hp, 100.0);
    }
}

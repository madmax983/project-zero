use crate::layer1::architecture::building::{Building, BuildingType};
use crate::layer1::architecture::structure::Structure;
use crate::layer1::economy::resources::{ColonyResources, ResourceType};
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
            // Deduct exact resource amount using safe sub
            let success = resources.try_consume(ev.resource, ev.amount);
            if !success {
                refusal_writer.send(AuditorRefusalEvent);
            }
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
                BuildingType::Tavern | BuildingType::Statue | BuildingType::Library
            ) {
                structure.current_hp = 0.0;
                break; // One precision strike per refusal
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::architecture::building::{Building, BuildingType};
    use crate::layer1::architecture::structure::Structure;
    use crate::layer1::economy::resources::{ColonyResources, ResourceType};
    use bevy_ecs::schedule::Schedule;

    fn setup_world() -> World {
        let mut world = World::new();
        world.insert_resource(ColonyResources::zeroed());
        world.init_resource::<Events<AuditorArrivalEvent>>();
        world.init_resource::<Events<AuditorDemandEvent>>();
        world.init_resource::<Events<AuditorRefusalEvent>>();
        world
    }

    #[test]
    fn test_auditor_demand_payment_success() {
        let mut world = setup_world();

        let mut resources = world.resource_mut::<ColonyResources>();
        resources.max_scrap = 50000.0;
        resources.add_scrap(5000.0);

        world
            .resource_mut::<Events<AuditorDemandEvent>>()
            .send(AuditorDemandEvent {
                amount: 5000.0,
                resource: ResourceType::Scrap,
            });

        let mut schedule = Schedule::default();
        schedule.add_systems(process_auditor_demand_system);
        schedule.run(&mut world);

        let resources = world.resource::<ColonyResources>();
        assert_eq!(resources.get_amount(ResourceType::Scrap), 0.0);
    }

    #[test]
    fn test_auditor_demand_payment_failure_triggers_strike() {
        let mut world = setup_world();

        let mut resources = world.resource_mut::<ColonyResources>();
        resources.max_scrap = 50000.0;
        resources.add_scrap(1000.0); // Not enough

        world
            .resource_mut::<Events<AuditorDemandEvent>>()
            .send(AuditorDemandEvent {
                amount: 5000.0,
                resource: ResourceType::Scrap,
            });

        let mut schedule = Schedule::default();
        schedule.add_systems(process_auditor_demand_system);
        schedule.run(&mut world);

        // Resources unchanged
        let resources = world.resource::<ColonyResources>();
        assert_eq!(resources.get_amount(ResourceType::Scrap), 1000.0);

        // AuditorRefusalEvent should be sent
        let refusal_events = world.resource::<Events<AuditorRefusalEvent>>();
        assert_eq!(refusal_events.len(), 1);
    }

    #[test]
    fn test_auditor_strike_destroys_luxury_building() {
        let mut world = setup_world();

        let tavern = world
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

        world
            .resource_mut::<Events<AuditorRefusalEvent>>()
            .send(AuditorRefusalEvent);

        let mut schedule = Schedule::default();
        schedule.add_systems(execute_auditor_strike_system);
        schedule.run(&mut world);

        let health = world.get::<Structure>(tavern).unwrap();
        assert!(health.current_hp <= 0.0);
    }

    #[test]
    fn test_auditor_strike_ignores_military_building() {
        let mut world = setup_world();

        let turret = world
            .spawn((
                Building {
                    building_type: BuildingType::Office,
                },
                Structure {
                    current_hp: 100.0,
                    max_hp: 100.0,
                },
            ))
            .id();

        world
            .resource_mut::<Events<AuditorRefusalEvent>>()
            .send(AuditorRefusalEvent);

        let mut schedule = Schedule::default();
        schedule.add_systems(execute_auditor_strike_system);
        schedule.run(&mut world);

        let health = world.get::<Structure>(turret).unwrap();
        assert_eq!(health.current_hp, 100.0);
    }
}

use crate::layer1::economy::resources::ColonyResources;
use crate::layer1::environment::disasters::DisasterEvent;
use crate::layer1::social::unrest::Unrest;
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct Colony;

#[derive(Component)]
pub struct DisasterWarning {
    pub active: bool,
}

#[derive(Component)]
pub struct CassandraProtocolActive;

#[derive(Component)]
pub struct ResourceHoarding;

#[derive(Component, PartialEq, Debug)]
pub enum CapitalStatus {
    Loyal,
    Rebellious,
}

pub fn activate_cassandra_protocol(
    mut commands: Commands,
    mut query: Query<(Entity, &mut CapitalStatus), With<CassandraProtocolActive>>,
) {
    for (entity, mut status) in query.iter_mut() {
        *status = CapitalStatus::Rebellious;
        commands.entity(entity).insert(ResourceHoarding);
    }
}

pub fn handle_disaster_strike(
    mut events: EventReader<DisasterEvent>,
    mut resources: ResMut<ColonyResources>,
    mut unrest: ResMut<Unrest>,
    query: Query<&CassandraProtocolActive>,
) {
    for _event in events.read() {
        if query.is_empty() {
            unrest.level = (unrest.level + 0.5).min(1.0);
            resources.food = (resources.food - 80.0).max(0.0);
            resources.metal = (resources.metal - 80.0).max(0.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cassandra_protocol_triggers_hoarding_and_rebellion() {
        let mut app = bevy_ecs::world::World::new();
        app.insert_resource(ColonyResources {
            food: 50.0,
            ..Default::default()
        });

        let colony = app
            .spawn((
                Colony,
                DisasterWarning { active: true },
                CapitalStatus::Loyal,
            ))
            .id();

        app.entity_mut(colony).insert(CassandraProtocolActive);

        let mut schedule = Schedule::default();
        schedule.add_systems(activate_cassandra_protocol);
        schedule.run(&mut app);

        assert_eq!(
            app.get::<CapitalStatus>(colony).unwrap(),
            &CapitalStatus::Rebellious
        );
        assert!(app.get::<ResourceHoarding>(colony).is_some());
    }

    #[test]
    fn test_disaster_strikes_without_protocol() {
        let mut app = bevy_ecs::world::World::new();
        app.init_resource::<Events<DisasterEvent>>();
        app.insert_resource(ColonyResources {
            food: 100.0,
            ..Default::default()
        });
        app.insert_resource(Unrest {
            level: 0.0,
            modifiers: vec![],
        });

        let _colony = app.spawn((Colony, DisasterWarning { active: true })).id();

        app.resource_mut::<Events<DisasterEvent>>()
            .send(DisasterEvent {
                disaster_type:
                    crate::layer1::environment::disasters::DisasterType::MassiveEarthquake,
                location: crate::layer1::map::GridPosition { x: 0, y: 0 },
                severity: 1.0,
            });

        let mut schedule = Schedule::default();
        schedule.add_systems(handle_disaster_strike);
        schedule.run(&mut app);

        assert!(app.resource::<Unrest>().level > 0.4);
        assert!(app.resource::<ColonyResources>().food < 50.0);
    }
}

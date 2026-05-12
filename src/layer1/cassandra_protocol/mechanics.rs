use crate::layer1::economy::resources::ColonyResources;
use bevy::prelude::*;

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

#[derive(Component)]
pub struct UnrestLevel(pub u32);

#[derive(Event)]
pub struct DisasterStrikeEvent(pub Entity);

pub fn activate_cassandra_protocol(
    mut commands: Commands,
    mut query: Query<(Entity, &mut CapitalStatus), With<CassandraProtocolActive>>,
) {
    for (entity, mut status) in query.iter_mut() {
        if *status != CapitalStatus::Rebellious {
            *status = CapitalStatus::Rebellious;
            commands.entity(entity).insert(ResourceHoarding);
        }
    }
}

pub fn handle_disaster_strike(
    mut events: EventReader<DisasterStrikeEvent>,
    mut query: Query<(Entity, &mut UnrestLevel, Option<&CassandraProtocolActive>)>,
    mut resources: ResMut<ColonyResources>,
) {
    for event in events.read() {
        if let Ok((_, mut unrest, protocol)) = query.get_mut(event.0) {
            if protocol.is_none() {
                unrest.0 += 100;
                resources.food = (resources.food - 80.0).max(0.0);
                resources.metal = (resources.metal - 80.0).max(0.0);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cassandra_protocol_triggers_hoarding_and_rebellion() {
        let mut app = App::new();
        app.add_systems(Update, activate_cassandra_protocol);

        let colony = app
            .world_mut()
            .spawn((
                Colony,
                DisasterWarning { active: true },
                CapitalStatus::Loyal,
            ))
            .id();

        // Player triggers protocol
        app.world_mut()
            .entity_mut(colony)
            .insert(CassandraProtocolActive);
        app.update();

        // Capital should now consider them a rebellion, and hoarding is active
        assert_eq!(
            app.world().get::<CapitalStatus>(colony).unwrap(),
            &CapitalStatus::Rebellious
        );
        assert!(app.world().get::<ResourceHoarding>(colony).is_some());
    }

    #[test]
    fn test_disaster_strikes_without_protocol() {
        let mut app = App::new();
        app.init_resource::<Events<DisasterStrikeEvent>>();
        app.insert_resource(ColonyResources {
            food: 100.0,
            metal: 100.0,
            ..Default::default()
        });
        app.add_systems(Update, handle_disaster_strike);

        let colony = app
            .world_mut()
            .spawn((Colony, DisasterWarning { active: true }, UnrestLevel(0)))
            .id();

        // Trigger disaster
        app.world_mut()
            .resource_mut::<Events<DisasterStrikeEvent>>()
            .send(DisasterStrikeEvent(colony));
        app.update();

        // Massive unrest and resource destruction because protocol wasn't active
        assert!(app.world().get::<UnrestLevel>(colony).unwrap().0 > 50);
        assert!(app.world().resource::<ColonyResources>().food < 50.0);
    }

    #[test]
    fn test_disaster_strikes_with_protocol() {
        let mut app = App::new();
        app.init_resource::<Events<DisasterStrikeEvent>>();
        app.insert_resource(ColonyResources {
            food: 100.0,
            metal: 100.0,
            ..Default::default()
        });
        app.add_systems(Update, handle_disaster_strike);

        let colony = app
            .world_mut()
            .spawn((
                Colony,
                DisasterWarning { active: true },
                CassandraProtocolActive,
                UnrestLevel(0),
            ))
            .id();

        app.world_mut()
            .resource_mut::<Events<DisasterStrikeEvent>>()
            .send(DisasterStrikeEvent(colony));
        app.update();

        // Because protocol is active, no massive unrest or resource destruction
        assert_eq!(app.world().get::<UnrestLevel>(colony).unwrap().0, 0);
        assert_eq!(app.world().resource::<ColonyResources>().food, 100.0);
    }
}

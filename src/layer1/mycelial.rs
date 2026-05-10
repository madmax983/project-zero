
use bevy_ecs::prelude::*;
use crate::layer1::map::GridPosition;

#[derive(Component)]
pub struct SensorShroom {
    pub network_id: u32,
}

#[derive(Component)]
pub struct SporeTurret {
    pub network_id: u32,
    pub is_active: bool,
    pub target: Option<Entity>,
}

#[derive(Event)]
pub struct MycelialTripwireEvent {
    pub network_id: u32,
    pub triggering_entity: Entity,
}

#[allow(clippy::type_complexity)]
pub fn detect_tripwire_step(
    pops: Query<(Entity, &GridPosition), (With<crate::layer1::pop::Pop>, Without<SensorShroom>)>,
    sensors: Query<(&SensorShroom, &GridPosition)>,
    mut events: EventWriter<MycelialTripwireEvent>,
) {
    for (pop_entity, pop_pos) in pops.iter() {
        for (sensor, sensor_pos) in sensors.iter() {
            if pop_pos == sensor_pos {
                events.send(MycelialTripwireEvent {
                    network_id: sensor.network_id,
                    triggering_entity: pop_entity,
                });
            }
        }
    }
}

pub fn aggro_network_entities(
    mut events: EventReader<MycelialTripwireEvent>,
    mut turrets: Query<&mut SporeTurret>,
) {
    for event in events.read() {
        for mut turret in turrets.iter_mut() {
            if turret.network_id == event.network_id {
                turret.is_active = true;
                turret.target = Some(event.triggering_entity);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::{App, Update};

    #[test]
    fn test_stepping_on_sensor_triggers_network_alert() {
        let mut app = App::new();
        app.add_event::<MycelialTripwireEvent>();
        app.add_systems(Update, detect_tripwire_step);

        let pop = app.world_mut().spawn((crate::layer1::pop::Pop, GridPosition { x: 5, y: 5 })).id();
        let _sensor = app.world_mut().spawn((
            SensorShroom { network_id: 1 },
            GridPosition { x: 5, y: 5 },
        )).id();

        app.update();

        let events = app.world().resource::<Events<MycelialTripwireEvent>>();
        let mut reader = events.get_cursor();
        let triggered = reader.read(events).next().unwrap();

        assert_eq!(triggered.network_id, 1, "Network 1 should be alerted.");
        assert_eq!(triggered.triggering_entity, pop, "Pop entity should be identified as trigger.");
    }

    #[test]
    fn test_spore_turret_aggro_on_alert() {
        let mut app = App::new();
        app.add_event::<MycelialTripwireEvent>();
        app.add_systems(Update, aggro_network_entities);

        let pop = app.world_mut().spawn(GridPosition { x: 10, y: 10 }).id();

        let turret = app.world_mut().spawn((
            SporeTurret { network_id: 1, is_active: false, target: None },
        )).id();

        app.world_mut().send_event(MycelialTripwireEvent {
            network_id: 1,
            triggering_entity: pop,
        });

        app.update();

        let turret_comp = app.world().get::<SporeTurret>(turret).unwrap();
        assert!(turret_comp.is_active, "Turret should become active on network alert.");
        assert_eq!(turret_comp.target, Some(pop), "Turret should target the triggering entity.");
    }
}

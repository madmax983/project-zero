use crate::layer1::social::morale::Morale;
use crate::layer1::environment::disasters::DisasterType;
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct Prophetic {
    pub cooldown: f32,
}

#[derive(Component)]
pub struct ActiveProphecy {
    pub disaster_type: DisasterType,
}

#[derive(Component)]
pub struct CultLeader;

#[derive(Event)]
pub struct DoomsdayWarningEvent {
    pub prophet_entity: Entity,
    pub disaster_type: DisasterType,
}

#[derive(Event)]
pub struct DisasterOccurredEvent {
    pub disaster_type: DisasterType,
}

pub fn generate_doomsday_warning(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Prophetic)>,
    mut warning_writer: EventWriter<DoomsdayWarningEvent>,
) {
    for (entity, mut prophetic) in query.iter_mut() {
        if prophetic.cooldown <= 0.0 {
            let disaster_type = DisasterType::MassiveEarthquake;

            commands
                .entity(entity)
                .insert(ActiveProphecy { disaster_type });

            warning_writer.send(DoomsdayWarningEvent {
                prophet_entity: entity,
                disaster_type,
            });

            prophetic.cooldown = 100.0;
        } else {
            prophetic.cooldown -= 1.0;
        }
    }
}

pub fn handle_ignored_warning(
    mut events: EventReader<DoomsdayWarningEvent>,
    mut query: Query<&mut Morale>,
) {
    for event in events.read() {
        if let Ok(mut morale) = query.get_mut(event.prophet_entity) {
            morale.value -= 0.20;
            morale.value = morale.value.max(0.0);
        }
    }
}

pub fn validate_prophecy(
    mut commands: Commands,
    mut events: EventReader<DisasterOccurredEvent>,
    query: Query<(Entity, &ActiveProphecy)>,
) {
    for event in events.read() {
        for (entity, prophecy) in query.iter() {
            if prophecy.disaster_type == event.disaster_type {
                commands.entity(entity).insert(CultLeader);
                commands.entity(entity).remove::<ActiveProphecy>();
            }
        }
    }
}





#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::entities::pop::Pop;
    use bevy_app::{App, Update};
#[test]
fn test_prophetic_pop_generates_warning() {
    // Arrange
    let mut app = App::new();
    app.add_systems(Update, generate_doomsday_warning);
    app.add_event::<DoomsdayWarningEvent>();

    // Spawn a pop with the Prophetic trait
    let pop_id = app
        .world_mut()
        .spawn((Pop, Prophetic { cooldown: 0.0 }))
        .id();

    // Act
    app.update();

    // Assert
    let warning_events = app.world().resource::<Events<DoomsdayWarningEvent>>();
    let mut reader = warning_events.get_cursor();
    let events: Vec<_> = reader.read(warning_events).collect();
    assert_eq!(events.len(), 1, "Prophet should generate a warning");
    assert_eq!(events[0].prophet_entity, pop_id);
}

#[test]
fn test_ignored_warning_drops_prophet_morale() {
    // Arrange
    let mut app = App::new();
    app.add_event::<DoomsdayWarningEvent>();
    app.add_systems(Update, handle_ignored_warning);

    let prophet = app
        .world_mut()
        .spawn((
            Pop,
            Morale {
                value: 1.0,
                ..Default::default()
            },
            Prophetic { cooldown: 10.0 },
        ))
        .id();

    let warning = DoomsdayWarningEvent {
        prophet_entity: prophet,
        disaster_type: DisasterType::MassiveEarthquake,
    };

    // Act
    app.world_mut()
        .resource_mut::<Events<DoomsdayWarningEvent>>()
        .send(warning);
    app.update();

    // Assert
    let morale = app.world().get::<Morale>(prophet).unwrap();
    assert!(
        morale.value < 1.0,
        "Ignored warning should drop prophet's morale"
    );
}

#[test]
fn test_disaster_occurrence_spawns_cult() {
    // Arrange
    let mut app = App::new();
    app.add_event::<DisasterOccurredEvent>();
    app.add_systems(Update, validate_prophecy);

    let _prophet = app
        .world_mut()
        .spawn((
            Pop,
            Prophetic { cooldown: 10.0 },
            ActiveProphecy {
                disaster_type: DisasterType::MassiveEarthquake,
            },
        ))
        .id();

    let disaster = DisasterOccurredEvent {
        disaster_type: DisasterType::MassiveEarthquake,
    };

    // Act
    app.world_mut()
        .resource_mut::<Events<DisasterOccurredEvent>>()
        .send(disaster);
    app.update();

    // Assert
    let query = app
        .world_mut()
        .query::<&CultLeader>()
        .get_single(app.world());
    assert!(
        query.is_ok(),
        "True prophecy should make the prophet a cult leader"
    );
}
}

use bevy_ecs::prelude::*;
use crate::layer1::law::justice::{CrimeCommittedEvent, CrimeRecord};

#[derive(Component)]
pub struct DiplomaticImmunity {
    pub faction_id: u32,
}

#[derive(Event)]
pub struct ArrestEvent {
    pub target: Entity,
}

#[derive(Event)]
pub struct DiplomaticIncidentEvent {
    pub faction_id: u32,
    pub reason: String,
}

pub fn evaluate_diplomatic_crime_system(
    mut crime_events: EventReader<CrimeCommittedEvent>,
    mut query: Query<(&mut CrimeRecord, Option<&DiplomaticImmunity>)>,
) {
    for event in crime_events.read() {
        if let Ok((mut record, immunity)) = query.get_mut(event.perpetrator) {
            if immunity.is_some() {
                record.wanted = false;
            }
        }
    }
}

pub fn process_diplomatic_arrest_system(
    mut arrest_events: EventReader<ArrestEvent>,
    mut incident_writer: EventWriter<DiplomaticIncidentEvent>,
    query: Query<&DiplomaticImmunity>,
) {
    for event in arrest_events.read() {
        if let Ok(immunity) = query.get(event.target) {
            incident_writer.send(DiplomaticIncidentEvent {
                faction_id: immunity.faction_id,
                reason: "Violation of Diplomatic Immunity".to_string(),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::pop::Pop;
    use crate::layer1::law::justice::{CrimeType, CrimeRecord, process_crimes_system};

    fn setup_world() -> World {
        let mut world = World::new();
        world.init_resource::<Events<CrimeCommittedEvent>>();
        world.init_resource::<Events<ArrestEvent>>();
        world.init_resource::<Events<DiplomaticIncidentEvent>>();
        world
    }

    #[test]
    fn test_diplomat_commits_crime_ignored_by_sheriff() {
        let mut world = setup_world();

        let diplomat = world.spawn((
            Pop,
            DiplomaticImmunity { faction_id: 2 },
            CrimeRecord::default()
        )).id();

        world.send_event(CrimeCommittedEvent {
            perpetrator: diplomat,
            crime_type: CrimeType::Theft,
        });

        let mut schedule = Schedule::default();
        schedule.add_systems((process_crimes_system, evaluate_diplomatic_crime_system).chain());
        schedule.run(&mut world);

        let record = world.get::<CrimeRecord>(diplomat).unwrap();
        assert!(!record.is_wanted());
    }

    #[test]
    fn test_player_forced_arrest_triggers_diplomatic_incident() {
        let mut world = setup_world();

        let diplomat = world.spawn((
            Pop,
            DiplomaticImmunity { faction_id: 2 },
        )).id();

        world.send_event(ArrestEvent {
            target: diplomat,
        });

        let mut schedule = Schedule::default();
        schedule.add_systems(process_diplomatic_arrest_system);
        schedule.run(&mut world);

        let incidents = world.resource::<Events<DiplomaticIncidentEvent>>();
        let mut reader = incidents.get_cursor();
        let iter: Vec<_> = reader.read(incidents).collect();
        assert_eq!(iter.len(), 1);
        assert_eq!(iter[0].faction_id, 2);
    }
}

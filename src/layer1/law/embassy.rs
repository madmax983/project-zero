use bevy_ecs::prelude::*;
use crate::layer1::law::justice::{CrimeCommittedEvent, CrimeRecord, ArrestEvent};
use crate::layer1::diplomacy::{DiplomaticImmunity, DiplomaticIncidentEvent};

pub fn evaluate_diplomatic_crime_system(
    mut crime_events: EventReader<CrimeCommittedEvent>,
    mut query: Query<(&mut CrimeRecord, Option<&DiplomaticImmunity>)>,
) {
    for event in crime_events.read() {
        if let Ok((mut record, immunity)) = query.get_mut(event.perpetrator) {
            // If they have immunity, the local justice system ignores them
            if immunity.is_some() {
                record.wanted = false;
                // Note: The crime still happens (victim dies, item stolen),
                // but the system doesn't generate a bounty.
            } else {
                record.wanted = true;
                record.severity_type = event.severity.clone();
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
            // An arrest was forced on a diplomat!
            incident_writer.send(DiplomaticIncidentEvent {
                faction_id: immunity.faction_id,
                reason: "Violation of Diplomatic Immunity".to_string(),
            });
        }
    }
}

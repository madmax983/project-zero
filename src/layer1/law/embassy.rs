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
                record.crime_severity = event.severity.clone();
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

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::pop::Pop;
    use crate::layer1::law::justice::{CrimeCommittedEvent, CrimeRecord, ArrestEvent, CrimeSeverity};
    use crate::layer1::diplomacy::{DiplomaticImmunity, DiplomaticIncidentEvent};
    use super::{evaluate_diplomatic_crime_system, process_diplomatic_arrest_system};

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

        // Spawn a diplomat pop
        use crate::layer1::social::factions::FactionId;
        let diplomat = world.spawn((
            Pop,
            DiplomaticImmunity { faction_id: FactionId::FarmersGuild }, // Assuming FactionId exists
            CrimeRecord::default()
        )).id();

        // Trigger a crime
        world.send_event(CrimeCommittedEvent {
            perpetrator: diplomat,
            crime_type: crate::layer1::law::justice::CrimeType::Theft,
            severity: CrimeSeverity::Major, // e.g., Murder
        });

        let mut schedule = Schedule::default();
        schedule.add_systems(evaluate_diplomatic_crime_system);
        schedule.run(&mut world);

        // A diplomat committing a crime should NOT get a Wanted token
        // that triggers Sheriff tasks automatically, to prevent AI sheriffs from starting wars.
        let record = world.get::<CrimeRecord>(diplomat).unwrap();
        assert!(!record.is_wanted());
    }

    #[test]
    fn test_player_forced_arrest_triggers_diplomatic_incident() {
        let mut world = setup_world();

        use crate::layer1::social::factions::FactionId;
        let diplomat = world.spawn((
            Pop,
            DiplomaticImmunity { faction_id: FactionId::FarmersGuild },
        )).id();

        // The player manually orders an arrest despite immunity
        world.send_event(ArrestEvent {
            target: diplomat,
        });

        let mut schedule = Schedule::default();
        schedule.add_systems(process_diplomatic_arrest_system);
        schedule.run(&mut world);

        // This must trigger a Layer 3 incident
        let incidents = world.resource::<Events<DiplomaticIncidentEvent>>();
        let mut reader = incidents.get_cursor();
        let iter: Vec<_> = reader.read(incidents).collect();
        assert_eq!(iter.len(), 1);
        assert_eq!(iter[0].faction_id, FactionId::FarmersGuild);
    }
}

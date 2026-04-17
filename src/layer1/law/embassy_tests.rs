#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::pop::Pop;
    use crate::layer1::law::justice::{CrimeCommittedEvent, CrimeRecord, ArrestEvent, CrimeSeverity};
    use crate::layer1::diplomacy::{DiplomaticImmunity, DiplomaticIncidentEvent};
    use crate::layer1::law::embassy::{evaluate_diplomatic_crime_system, process_diplomatic_arrest_system};

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
        let diplomat = world.spawn((
            Pop,
            DiplomaticImmunity { faction_id: 2 },
            CrimeRecord::default()
        )).id();

        // Trigger a crime
        world.send_event(CrimeCommittedEvent {
            perpetrator: diplomat,
            crime_type: crate::layer1::law::justice::CrimeType::Assault,
            severity: CrimeSeverity::Major, // e.g., Murder
        });

        let mut schedule = Schedule::default();
        schedule.add_systems((
            crate::layer1::law::justice::process_crimes_system,
            evaluate_diplomatic_crime_system,
        ).chain());
        schedule.run(&mut world);

        // A diplomat committing a crime should NOT get a Wanted token
        // that triggers Sheriff tasks automatically, to prevent AI sheriffs from starting wars.
        let record = world.get::<CrimeRecord>(diplomat).unwrap();
        // The implementation doesn't exist yet, so test should fail here
        assert!(!record.is_wanted(), "Diplomat became wanted despite immunity");
    }

    #[test]
    fn test_player_forced_arrest_triggers_diplomatic_incident() {
        let mut world = setup_world();

        let diplomat = world.spawn((
            Pop,
            DiplomaticImmunity { faction_id: 2 },
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
        assert_eq!(iter[0].faction_id, 2);
    }
}

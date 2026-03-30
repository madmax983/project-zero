# 764: The Embassy Sector

## 1. Overview
"Welcome to our home. Under no circumstances are you to enforce our laws here."

A slice of alien culture in your backyard. Players can designate a specific Layer 1 Zone as "Extraterritorial". Alien dignitaries, diplomats, and high-value tourists live there under their own laws. If the player violates their laws—for example, by arresting an alien diplomat who just committed murder in the colony—a galactic war or severe trade embargo starts.

This introduces a maddening tension: allowing high-level crime to happen unpunished causes massive local Unrest and Justice penalties, but enforcing the law guarantees annihilation from a superior Layer 3 empire.

## 2. Dependencies
- `056` Designated Zones (Zone creation and management)
- `562` The Justice System (Crimes, Wanted levels, Arrests)
- `068` Pop Factions (Layer 3 entities and relations)

## 3. RED Phase: Tests First
Write these tests in `src/layer1/justice/embassy_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::pop::{Pop, Trait};
    use crate::layer1::justice::{CrimeCommittedEvent, CrimeRecord, SheriffTask, ArrestEvent, CrimeSeverity};
    use crate::layer1::zones::{Zone, ZoneType};
    use crate::layer1::diplomacy::{DiplomaticImmunity, DiplomaticIncidentEvent};
    use crate::layer1::justice::embassy::{evaluate_diplomatic_crime_system, process_diplomatic_arrest_system};

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
            criminal: diplomat,
            severity: CrimeSeverity::Major, // e.g., Murder
        });

        let mut schedule = Schedule::default();
        schedule.add_systems(evaluate_diplomatic_crime_system);
        schedule.run(&mut world);

        // A diplomat committing a crime should NOT get a Wanted token
        // that triggers Sheriff tasks automatically, to prevent AI sheriffs from starting wars.
        let record = world.get::<CrimeRecord>(diplomat).unwrap();
        assert!(!record.is_wanted);
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
        let mut reader = incidents.get_reader();
        let iter: Vec<_> = reader.read(incidents).collect();
        assert_eq!(iter.len(), 1);
        assert_eq!(iter[0].faction_id, 2);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/layer1/justice/embassy.rs

use bevy_ecs::prelude::*;
use crate::layer1::justice::{CrimeCommittedEvent, CrimeRecord, ArrestEvent, CrimeSeverity};
use crate::layer1::diplomacy::{DiplomaticImmunity, DiplomaticIncidentEvent};
use crate::layer1::zones::{Zone, ZoneType};

pub fn evaluate_diplomatic_crime_system(
    mut crime_events: EventReader<CrimeCommittedEvent>,
    mut query: Query<(&mut CrimeRecord, Option<&DiplomaticImmunity>)>,
) {
    for event in crime_events.read() {
        if let Ok((mut record, immunity)) = query.get_mut(event.criminal) {
            // If they have immunity, the local justice system ignores them
            if immunity.is_some() {
                record.is_wanted = false;
                // Note: The crime still happens (victim dies, item stolen),
                // but the system doesn't generate a bounty.
            } else {
                record.is_wanted = true;
                record.severity = event.severity.clone();
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
```

## 5. REFACTOR Phase: Quality & Design

- **Zone Restrictions**: Add logic so that Pops without `DiplomaticImmunity` who enter an `ExtraterritorialZone` are subjected to the *Alien's* laws, potentially being arrested by Alien guards.
- **Unrest Generation**: A `Diplomat` committing a `CrimeSeverity::Major` (like murder) in the colony should generate a massive, spreading "Injustice" Unrest modifier among normal Pops.
- **Eviction**: Players should have a diplomatic option to "Expel" a diplomat rather than "Arrest" them. Expulsion damages relations but avoids war.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures for new code.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for `src/layer1/justice/embassy.rs`.
- [ ] Pops with `DiplomaticImmunity` do not become `Wanted` when committing crimes.
- [ ] Manually arresting an immune Pop fires a `DiplomaticIncidentEvent`.

## 7. Technical Guidance
- Integrate `DiplomaticImmunity` deeply with the Utility AI so that Sheriff Pops explicitly filter out immune targets during their target-selection queries.
- Connect `DiplomaticIncidentEvent` to the Layer 3 diplomacy system, where it will immediately plunge `faction_id` relations to Hostile/War state.

## 8. Questions
*Builder: add questions here if spec is unclear.*

# The Justice System

## 1. Overview
Law and order on the frontier. Pops who commit crimes (such as Vandalism from Unrest, Theft, or Assault) generate "Wanted" tokens. "Sheriff" jobs track and arrest them, moving them to designated "Jail" zones for a duration. The player can choose to "Pardon" them, causing corruption or angering victims, but returning the Pop to utility. This system forces the player to balance strict justice (stability) vs. pragmatic leniency (utility).

## 2. Dependencies
- `016-utility-ai-system` (Action evaluation)
- `036-pop-memory` (Recording crimes and pardons)
- `056-designated-zones` (Jail zones)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_crime_generates_wanted_token() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let criminal_entity = app.world.spawn((
            Pop,
            CrimeRecord::default(),
        )).id();

        // Simulating a crime event
        app.world.send_event(CrimeCommittedEvent {
            perpetrator: criminal_entity,
            crime_type: CrimeType::Theft,
        });

        app.add_systems(Update, process_crimes_system);
        app.update();

        let record = app.world.get::<CrimeRecord>(criminal_entity).unwrap();
        assert!(record.is_wanted(), "Pop should have a wanted token after a crime");
        assert_eq!(record.severity, 50, "Theft should generate severity 50");
    }

    #[test]
    fn test_sheriff_arrests_wanted_pop() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let jail_zone = app.world.spawn((
            Zone { zone_type: ZoneType::Jail },
            Position { x: 5, y: 5 },
        )).id();

        let criminal_entity = app.world.spawn((
            Pop,
            Position { x: 10, y: 10 },
            CrimeRecord { wanted: true, severity: 50 },
        )).id();

        let sheriff_entity = app.world.spawn((
            Pop,
            Job { role: JobRole::Sheriff },
            Position { x: 9, y: 10 },
        )).id();

        // Simulating sheriff arresting pop
        app.add_systems(Update, sheriff_arrest_system);
        app.update();

        // Assert: Criminal should be moved to jail zone and marked as arrested
        let record = app.world.get::<CrimeRecord>(criminal_entity).unwrap();
        assert!(record.is_arrested, "Criminal should be marked arrested");
        assert_eq!(app.world.get::<Position>(criminal_entity).unwrap().x, 5, "Criminal moved to jail X");
        assert_eq!(app.world.get::<Position>(criminal_entity).unwrap().y, 5, "Criminal moved to jail Y");
    }

    #[test]
    fn test_pardon_removes_wanted_but_adds_corruption() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        app.world.insert_resource(ColonyStats { corruption: 0 });

        let criminal_entity = app.world.spawn((
            Pop,
            CrimeRecord { wanted: true, severity: 50, is_arrested: true },
        )).id();

        // Player issues a pardon
        app.world.send_event(PardonIssuedEvent { target: criminal_entity });

        app.add_systems(Update, process_pardons_system);
        app.update();

        let record = app.world.get::<CrimeRecord>(criminal_entity).unwrap();
        assert!(!record.is_wanted(), "Pardon should remove wanted status");
        assert!(!record.is_arrested, "Pardon should release from jail");

        let stats = app.world.resource::<ColonyStats>();
        assert!(stats.corruption > 0, "Pardoning should increase corruption");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component, Default)]
pub struct CrimeRecord {
    pub wanted: bool,
    pub severity: u32,
    pub is_arrested: bool,
}

impl CrimeRecord {
    pub fn is_wanted(&self) -> bool {
        self.wanted
    }
}

pub enum CrimeType {
    Theft,
    Assault,
    Vandalism,
}

#[derive(Event)]
pub struct CrimeCommittedEvent {
    pub perpetrator: Entity,
    pub crime_type: CrimeType,
}

#[derive(Event)]
pub struct PardonIssuedEvent {
    pub target: Entity,
}

// Updates CrimeRecord when a CrimeCommittedEvent is received
pub fn process_crimes_system(
    mut events: EventReader<CrimeCommittedEvent>,
    mut query: Query<&mut CrimeRecord>,
) {
    for ev in events.read() {
        if let Ok(mut record) = query.get_mut(ev.perpetrator) {
            record.wanted = true;
            record.severity += match ev.crime_type {
                CrimeType::Theft => 50,
                CrimeType::Assault => 80,
                CrimeType::Vandalism => 30,
            };
        }
    }
}

// Finds wanted pops near sheriffs and moves them to jail
pub fn sheriff_arrest_system(
    mut criminal_query: Query<(Entity, &mut Position, &mut CrimeRecord), Without<Job>>,
    sheriff_query: Query<&Position, With<Job>>,
    jail_query: Query<&Position, With<Zone>>,
) {
    if let Some(jail_pos) = jail_query.iter().next() {
        for sheriff_pos in sheriff_query.iter() {
            for (ent, mut crim_pos, mut record) in criminal_query.iter_mut() {
                if record.wanted && !record.is_arrested {
                    // Check adjacency (simplified distance)
                    let dist = (sheriff_pos.x - crim_pos.x).abs() + (sheriff_pos.y - crim_pos.y).abs();
                    if dist <= 1 {
                        record.is_arrested = true;
                        record.wanted = false;
                        crim_pos.x = jail_pos.x;
                        crim_pos.y = jail_pos.y;
                    }
                }
            }
        }
    }
}

// Processes PardonIssuedEvents, clearing record and adding corruption
pub fn process_pardons_system(
    mut events: EventReader<PardonIssuedEvent>,
    mut query: Query<&mut CrimeRecord>,
    mut stats: ResMut<ColonyStats>,
) {
    for ev in events.read() {
        if let Ok(mut record) = query.get_mut(ev.target) {
            record.wanted = false;
            record.is_arrested = false;
            record.severity = 0; // Cleared
            stats.corruption += 5; // Flat penalty
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Refactor 1:** `sheriff_arrest_system` currently teleports criminals instantly to jail. It should generate a `HaulAction` or `ArrestAction` where the Sheriff physically escorts them.
- **Refactor 2:** Multiple jails are not handled well (it takes the first one found). Need a `find_closest_jail` utility function.
- **Refactor 3:** Pardons should generate a `ChronicleEvent` and potentially lower the morale of the crime's victim (if it was a targeted crime).

## 6. Acceptance Criteria
- [ ] `cargo test` passes all RED phase tests.
- [ ] `cargo clippy -- -D warnings` returns 0 warnings.
- [ ] Test coverage hits at least 85% for `src/layer1/justice.rs`.
- [ ] Committing crimes successfully generates "Wanted" tokens with appropriate severity.
- [ ] Pops with the Sheriff job can successfully arrest wanted Pops and move them to a designated Jail zone.
- [ ] Arrested Pops cannot perform normal work actions.
- [ ] Pardoning an arrested Pop successfully removes their wanted/arrested status but increases colony corruption.

## 7. Technical Guidance
- Integrate with `016-utility-ai-system` so arrested Pops cannot select typical work tasks while their `is_arrested` flag is true.
- Pardons should be triggered from the UI `Inspector` view.
- Consider adding a `ReleaseTimer` so Pops eventually serve their sentence and are released automatically.

## 8. Questions
*Builder: add questions here if spec is unclear. Architect will address.*

# 725 The Orphaned Megastructure

## 1. Overview
A massive, unidentifiable precursor megastructure drifts into the system. You can establish a specialized Layer 1 colony directly on its surface to mine it for unparalleled, endgame materials. However, the structure occasionally activates dormant sub-routines, altering the map's terrain, changing the atmosphere, or spawning hostile, ancient automated defenders.

## 2. Dependencies
- 002 Basic Map
- 018 Mining and Resources
- 095 System Generation

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_megastructure_subroutine_activation() {
        let mut world = World::new();
        // Setup megastructure state
        let megastructure = world.spawn(OrphanedMegastructure {
            dormancy_timer: Timer::from_seconds(10.0, TimerMode::Repeating),
            subroutines: vec![Subroutine::SpawnDefenders],
        }).id();

        let mut time = Time::default();
        time.advance_by(std::time::Duration::from_secs(11));
        world.insert_resource(time);

        let mut schedule = Schedule::default();
        schedule.add_systems(process_megastructure_subroutines);
        schedule.run(&mut world);

        // Assert defenders were spawned or event emitted
        let events = world.resource::<Events<SubroutineActivatedEvent>>();
        let mut reader = events.get_reader();
        let activated_events: Vec<_> = reader.read(events).collect();

        assert_eq!(activated_events.len(), 1, "A subroutine should activate after timer finishes");
        assert_eq!(activated_events[0].megastructure, megastructure);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct OrphanedMegastructure {
    pub dormancy_timer: Timer,
    pub subroutines: Vec<Subroutine>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Subroutine {
    SpawnDefenders,
    AlterTerrain,
    ChangeAtmosphere,
}

#[derive(Event)]
pub struct SubroutineActivatedEvent {
    pub megastructure: Entity,
    pub subroutine: Subroutine,
}

pub fn process_megastructure_subroutines(
    time: Res<Time>,
    mut query: Query<(Entity, &mut OrphanedMegastructure)>,
    mut events: EventWriter<SubroutineActivatedEvent>,
) {
    for (entity, mut mega) in query.iter_mut() {
        mega.dormancy_timer.tick(time.delta());
        if mega.dormancy_timer.just_finished() {
            if let Some(&subroutine) = mega.subroutines.first() {
                events.send(SubroutineActivatedEvent {
                    megastructure: entity,
                    subroutine,
                });
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- The subroutines should be selected randomly or based on colony actions (mining depth, population).
- Subroutines need their own individual resolution systems.

## 6. Acceptance Criteria
- [ ] `process_megastructure_subroutines` fires events when the timer elapses.
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.

## 7. Technical Guidance
- The timer duration should probably be influenced by player activity on the megastructure.
- Ensure event listening logic is properly hooked up to handle `SubroutineActivatedEvent`.

## 8. Questions
*Builder: add questions here if spec is unclear.*

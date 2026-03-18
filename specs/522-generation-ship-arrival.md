# 522. Generation Ship Arrival

## 1. Overview
**Layer:** Cross-layer
**Fantasy:** Meeting your ancestors, but they are strangers. A collision of eras.
**Mechanic:** A massive, sub-light ship from the distant past arrives. It contains thousands of Pops with "Archaic" traits and low tech. They demand settlement rights.
**Emergence:** You let them land. Suddenly your population triples, but they refuse to use neural interfaces and demand "Paper". A culture war erupts between the Spacers and the Ancients.
**Tension:** Turn them away (they die/attack) or Integrate them (chaos/population boom)?

## 2. Dependencies
- `003` Population Basics
- `068` Pop Factions
- `084` Pop Traits
- `234` The Visitor (implied arriving ships framework)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::pop::{Pop, Trait, Faction};
    use crate::layer1::needs::Needs;
    use crate::layer2::events::GenerationShipArrivalEvent;

    #[test]
    fn test_generation_ship_arrival_adds_pops() {
        let mut app = App::new();
        app.add_event::<GenerationShipArrivalEvent>();

        let initial_pop_count = app.world().query::<&Pop>().iter(app.world()).count();

        app.world_mut().send_event(GenerationShipArrivalEvent { pop_count: 1000 });
        app.update();

        let final_pop_count = app.world().query::<&Pop>().iter(app.world()).count();
        assert_eq!(final_pop_count, initial_pop_count + 1000, "Generation ship should add the specified number of pops.");
    }

    #[test]
    fn test_generation_ship_pops_have_archaic_trait_and_faction() {
        let mut app = App::new();
        app.add_event::<GenerationShipArrivalEvent>();

        app.world_mut().send_event(GenerationShipArrivalEvent { pop_count: 10 });
        app.update();

        let mut query = app.world_mut().query::<(&Pop, &Trait, &Faction)>();
        let mut archaic_count = 0;

        for (_, pop_trait, faction) in query.iter(app.world()) {
            if pop_trait == &Trait::Archaic && faction == &Faction::Ancients {
                archaic_count += 1;
            }
        }

        assert_eq!(archaic_count, 10, "All new generation ship pops should have Archaic trait and Ancients faction.");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// The SIMPLEST code that makes tests pass
// In src/layer2/events/generation_ship.rs

#[derive(Event)]
pub struct GenerationShipArrivalEvent {
    pub pop_count: usize,
}

pub fn handle_generation_ship_arrival_system(
    mut commands: Commands,
    mut events: EventReader<GenerationShipArrivalEvent>,
) {
    for event in events.read() {
        for _ in 0..event.pop_count {
            commands.spawn((
                Pop,
                Trait::Archaic,
                Faction::Ancients,
                Needs::default(),
            ));
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Code Smells:** Spawning potentially thousands of entities in a single frame (`event.pop_count: 1000`) might cause a severe performance spike.
- **Performance:** Consider staggering the spawning process over several frames or using instanced rendering if the population gets too large. Group the pops into "Arrival Batches" to reduce overhead.
- **Feature Completeness:** The current minimal implementation doesn't give the player a choice (Turn them away vs Integrate). This requires a UI prompt and decision logic. We also need to define the negative effects of the `Trait::Archaic` (e.g., lower tech efficiency, higher demand for basic goods).

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >= 85% for new code
- [ ] Arrival event correctly spawns the specified number of Pops.
- [ ] New Pops correctly have the `Archaic` trait and belong to the `Ancients` faction.

## 7. Technical Guidance
- **Integration Points:** Connect the `GenerationShipArrivalEvent` to the UI notification system so the player is prompted to make a decision (Accept or Reject). Ensure the `Trait::Archaic` actually modifies work efficiency in the `job_system`.
- **Gotchas:** A massive influx of Pops will immediately drain food and housing. Players will likely fail if they accept a 1000-pop ship without preparation. Make sure the event gives them fair warning or time to build infrastructure.

## 8. Questions
*Builder: add questions here if spec is unclear.*

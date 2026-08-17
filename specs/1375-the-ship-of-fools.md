# 1375: The 'Ship of Fools'

## 1. Overview
**Layer:** 2 -> 1

**Fantasy:** Not every ship is a threat; some are just tragedies.

**Mechanic:** A ship arrives with no crew, only passengers who have no skills (e.g., "Tourists", "Bureaucrats"). They demand luxury but produce nothing. They are refugees from a pleasure-cruiser accident.

**Emergence:** You save them. They complain about the food. They start a faction demanded "Better Curtains". You realize they are more dangerous than the pirates because they are eating your surplus and doing nothing.

**Tension:** Altruism vs. Parasitism.

## 2. Dependencies
- Base ECS system
- Refugee/Visitor arrival system
- Pop skill/trait generation
- Faction/Demand system

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_event::<ShipOfFoolsArrivalEvent>();
        app.add_systems(Update, process_ship_of_fools_arrival_system);
        app
    }

    #[test]
    fn test_ship_of_fools_spawns_useless_pops() {
        let mut app = setup_app();

        app.world_mut().send_event(ShipOfFoolsArrivalEvent { count: 5 });
        app.update();

        // Check spawned pops
        let mut query = app.world_mut().query::<(&Pop, &Skills, &Trait)>();
        let mut count = 0;

        for (_, skills, pop_trait) in query.iter(app.world()) {
            count += 1;
            assert_eq!(skills.total_level, 0, "Fools should have zero skills");
            assert_eq!(*pop_trait, Trait::Entitled, "Fools should have Entitled trait");
        }

        assert_eq!(count, 5, "Should spawn exact number of fools");
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Pop;

#[derive(Component)]
pub struct Skills {
    pub total_level: u32,
}

#[derive(Component, PartialEq, Eq, Debug)]
pub enum Trait {
    Normal,
    Entitled,
}

#[derive(Event)]
pub struct ShipOfFoolsArrivalEvent {
    pub count: u32,
}

pub fn process_ship_of_fools_arrival_system(
    mut commands: Commands,
    mut events: EventReader<ShipOfFoolsArrivalEvent>,
) {
    for ev in events.read() {
        for _ in 0..ev.count {
            commands.spawn((
                Pop,
                Skills { total_level: 0 },
                Trait::Entitled,
            ));
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Expand the `Skills` component to zero out all individual skill categories (Mining, Farming, etc.).
- Ensure `Trait::Entitled` hooks into the needs system, causing them to demand higher-tier `Food` and `Housing` than standard pops.
- Allow them to generate unique, annoying faction demands via the `FactionSystem`.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >=85% for new code
- [ ] Processing `ShipOfFoolsArrivalEvent` spawns `Pop` entities with zero skills and the `Entitled` trait.

## 7. Technical Guidance
- Integrate with the existing `ArrivalEvent` or `ShipLanding` systems if possible to reuse animation and UI logic.
- They should still consume basic resources even if their entitlement demands aren't met, presenting the core tension.

## 8. Questions
*Builder: add questions here if spec is unclear.*

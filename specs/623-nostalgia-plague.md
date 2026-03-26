# Spec 623: The Nostalgia Plague

## 1. Overview
A psychological epidemic where the colony becomes paralyzed by an obsession with a romanticized past. Severe morale drops or traumatic events trigger "Nostalgia" in older Pops. These Pops spend their time replicating obsolete tools and hoarding old artifacts, proselytizing to younger Pops through the rumor web, eventually crashing productivity if not managed.

## 2. Dependencies
- `src/layer1/pop.rs` (Pop traits, Age)
- `src/layer1/morale.rs` (Morale drops)
- `src/layer1/memory.rs` (Rumors, Memories)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::pop::{Pop, Age};
    use crate::layer1::morale::Morale;
    use crate::layer1::memory::{Memory, Rumor};

    #[test]
    fn test_low_morale_triggers_nostalgia_in_old_pops() {
        let mut app = App::new();
        app.add_systems(Update, nostalgia_trigger_system);

        let pop_entity = app.world_mut().spawn((
            Pop,
            Age(65),
            Morale { value: 10.0 }, // Severe morale drop
        )).id();

        app.update();

        assert!(app.world().get::<Nostalgia>(pop_entity).is_some(), "Old pop with low morale should contract Nostalgia");
    }

    #[test]
    fn test_nostalgia_spreads_via_rumors() {
        let mut app = App::new();
        app.add_event::<RumorSpreadEvent>();
        app.add_systems(Update, nostalgia_spread_system);

        let infected = app.world_mut().spawn((Pop, Nostalgia)).id();
        let target = app.world_mut().spawn((Pop, Age(30))).id();

        app.world_mut().send_event(RumorSpreadEvent { source: infected, target, rumor: Rumor::PastGlory });

        app.update();

        assert!(app.world().get::<Nostalgia>(target).is_some(), "Nostalgia should spread to target via rumors");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer1::pop::{Pop, Age};
use crate::layer1::morale::Morale;

#[derive(Component)]
pub struct Nostalgia;

#[derive(Event)]
pub struct RumorSpreadEvent {
    pub source: Entity,
    pub target: Entity,
    pub rumor: Rumor,
}

pub enum Rumor {
    PastGlory,
    // other rumors
}

pub fn nostalgia_trigger_system(
    mut commands: Commands,
    query: Query<(Entity, &Age, &Morale), (With<Pop>, Without<Nostalgia>)>
) {
    for (entity, age, morale) in query.iter() {
        if age.0 >= 60 && morale.value <= 20.0 {
            commands.entity(entity).insert(Nostalgia);
        }
    }
}

pub fn nostalgia_spread_system(
    mut commands: Commands,
    mut events: EventReader<RumorSpreadEvent>,
    query: Query<&Nostalgia>,
) {
    for event in events.read() {
        if let Rumor::PastGlory = event.rumor {
            if query.get(event.source).is_ok() {
                commands.entity(event.target).insert(Nostalgia);
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Add a resistance mechanic so not every rumor spread guarantees an infection.
- Integrate the `Nostalgia` component with the `UtilityAI` to drastically lower the score of modern tasks (e.g., using new tech) and raise the score of obsolete actions.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] Test coverage ≥85% for new code.
- [ ] Nostalgia correctly triggers based on age and low morale constraints.

## 7. Technical Guidance
- Make sure to update the action evaluation system in Utility AI to account for the `Nostalgia` component modifier.

## 8. Questions
*Builder: add questions here if spec is unclear.*

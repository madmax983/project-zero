# The Colony Mascot

## 1. Overview
**Layer:** 1
**Fantasy:** A useless, ugly little creature that the marines would die for.
**Mechanic:** A non-hostile, non-productive unique animal spawns. It wanders social zones. Interacting with it gives a massive Mood buff. If it is killed (starvation/raid), the entire colony suffers a "Grief" breakdown.
**Emergence:** During a famine, the "Mascot" is the only one eating well. A "Pragmatist" faction pop tries to butcher it, sparking a civil war with the "Sentimentalist" faction.

## 2. Dependencies
- ECS Pop and Animal systems
- Mood/Morale system

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_mascot_interaction_buffs_mood() {
        let mut app = App::new();
        app.add_event::<MascotInteractionEvent>();
        app.add_systems(Update, mascot_interaction_system);

        let pop_entity = app.world_mut().spawn(PopMood { value: 50.0 }).id();
        let mascot_entity = app.world_mut().spawn(ColonyMascot).id();

        app.world_mut().resource_mut::<Events<MascotInteractionEvent>>().send(MascotInteractionEvent {
            pop: pop_entity,
            mascot: mascot_entity,
        });

        app.update();

        let mood = app.world().get::<PopMood>(pop_entity).unwrap();
        assert!(mood.value > 50.0, "Interacting with the mascot should increase mood");
    }

    #[test]
    fn test_mascot_death_causes_colony_grief() {
        let mut app = App::new();
        app.add_event::<MascotDeathEvent>();
        app.add_systems(Update, mascot_grief_system);

        let pop1 = app.world_mut().spawn(PopMood { value: 80.0 }).id();
        let pop2 = app.world_mut().spawn(PopMood { value: 60.0 }).id();

        app.world_mut().resource_mut::<Events<MascotDeathEvent>>().send(MascotDeathEvent);

        app.update();

        assert!(app.world().get::<PopMood>(pop1).unwrap().value < 80.0);
        assert!(app.world().get::<PopMood>(pop2).unwrap().value < 60.0);
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct ColonyMascot;

#[derive(Component)]
pub struct PopMood {
    pub value: f32,
}

#[derive(Event)]
pub struct MascotInteractionEvent {
    pub pop: Entity,
    pub mascot: Entity,
}

#[derive(Event)]
pub struct MascotDeathEvent;

pub fn mascot_interaction_system(
    mut events: EventReader<MascotInteractionEvent>,
    mut query: Query<&mut PopMood>,
) {
    for event in events.read() {
        if let Ok(mut mood) = query.get_mut(event.pop) {
            mood.value += 20.0; // Massive mood buff
            mood.value = mood.value.clamp(0.0, 100.0);
        }
    }
}

pub fn mascot_grief_system(
    mut events: EventReader<MascotDeathEvent>,
    mut query: Query<&mut PopMood>,
) {
    for _ in events.read() {
        for mut mood in query.iter_mut() {
            mood.value -= 50.0; // Massive mood penalty (Grief)
            mood.value = mood.value.clamp(0.0, 100.0);
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Replace global grief broadcast with an event that gets processed per-Pop based on their traits (e.g., Pragmatist vs. Sentimentalist).
- Add specific "Grief" status effects instead of just flat mood penalties.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Specific feature behavior verified: interaction buffs mood, death causes widespread grief.

## 7. Technical Guidance
- You will need a way to track the Mascot's location and whether a pop is near it to trigger interactions.

## 8. Questions
*Builder: add questions here if spec is unclear.*

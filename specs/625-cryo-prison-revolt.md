# Spec 625: The Cryo-Prison Revolt

## 1. Overview
A massive, ancient Layer 2 penal transport ship crashes onto the Layer 1 map, scattering intact cryo-pods. As pods thaw, highly skilled but highly dangerous "Criminal" Pops emerge. You can capture them for skilled labor, but they possess massive Unrest and will actively attempt to sabotage or take over the colony.

## 2. Dependencies
- `src/layer1/pop.rs` (Pop components, Skills)
- `src/layer1/unrest.rs` (Unrest events)
- `src/layer1/map.rs` (Terrain entities, Crash sites)
- `src/layer2/events.rs` (Ship crash events)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::pop::{Pop, Skills};
    use crate::layer1::unrest::Unrest;

    #[test]
    fn test_thawed_cryo_criminal_has_high_skills_and_unrest() {
        let mut app = App::new();
        app.add_systems(Update, thaw_cryo_pod_system);

        let pod_entity = app.world_mut().spawn((
            CryoPod { thaw_progress: 1.0, is_criminal: true },
        )).id();

        app.update();

        // Pod should be gone
        assert!(app.world().get::<CryoPod>(pod_entity).is_none(), "Pod should thaw and disappear");

        // Find the newly spawned criminal
        let mut found = false;
        for (entity, skills, unrest, criminal) in app.world_mut().query::<(Entity, &Skills, &Unrest, &CriminalRecord)>().iter() {
            assert!(skills.engineering > 80, "Criminal should have high skills");
            assert!(unrest.value > 80.0, "Criminal should have massive unrest");
            found = true;
        }
        assert!(found, "A new criminal pop should have been spawned");
    }

    #[test]
    fn test_criminal_sabotage_triggers_when_unrest_high() {
        let mut app = App::new();
        app.add_event::<SabotageEvent>();
        app.add_systems(Update, criminal_sabotage_system);

        let criminal = app.world_mut().spawn((
            Pop,
            CriminalRecord,
            Unrest { value: 95.0 }, // Over threshold
        )).id();

        app.update();

        let events = app.world().get_resource::<Events<SabotageEvent>>().unwrap();
        let mut reader = events.get_reader();
        assert!(reader.read(events).next().is_some(), "SabotageEvent should be fired by high-unrest criminal");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer1::pop::{Pop, Skills};
use crate::layer1::unrest::Unrest;

#[derive(Component)]
pub struct CryoPod {
    pub thaw_progress: f32, // 0.0 to 1.0
    pub is_criminal: bool,
}

#[derive(Component)]
pub struct CriminalRecord;

#[derive(Event)]
pub struct SabotageEvent {
    pub saboteur: Entity,
}

pub fn thaw_cryo_pod_system(
    mut commands: Commands,
    query: Query<(Entity, &CryoPod)>,
) {
    for (entity, pod) in query.iter() {
        if pod.thaw_progress >= 1.0 {
            commands.entity(entity).despawn();
            if pod.is_criminal {
                commands.spawn((
                    Pop,
                    CriminalRecord,
                    Skills { engineering: 85, ..default() },
                    Unrest { value: 90.0 },
                ));
            }
        }
    }
}

pub fn criminal_sabotage_system(
    mut events: EventWriter<SabotageEvent>,
    query: Query<(Entity, &Unrest, &CriminalRecord)>,
) {
    for (entity, unrest, _) in query.iter() {
        if unrest.value > 90.0 {
            events.send(SabotageEvent { saboteur: entity });
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Abstract the thaw logic so progress increments over time (e.g., `thaw_progress += delta_time`).
- Add a specific `Skillset` to the `CryoPod` so thawing generates specialized criminals (engineers, scientists, thugs).
- Link `SabotageEvent` to target specific colony structures via Layer 1 utility AI.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] Criminal pops spawn with high unrest and skills.
- [ ] High unrest triggers sabotage events.

## 7. Technical Guidance
- The Crash Event from Layer 2 should correctly scatter pods on valid map tiles in Layer 1.
- `SabotageEvent` should be caught by an `infrastructure_damage_system` to actually break something.

## 8. Questions
*Builder: add questions here if spec is unclear.*

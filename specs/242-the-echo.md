# 242: The Echo

**Layer:** 1 (Colony Simulation)
**Status:** Draft
**Complexity:** Low

---

## 1. Overview

**Fantasy:** The past bleeds into the present. A place where something terrible happened *feels* wrong.

**Mechanic:**
- High-importance **Chronicle Events** (e.g., `Death`, `Disaster`, `Miracle`) occuring at a specific location create an **EchoSource** on the Terrain.
- **EchoSources** periodically spawn visual **Echo** entities (ghostly reenactments).
- Pops witnessing an Echo are **Distracted** (stop work to watch) and gain **Mood** modifiers (Stress/Awe) based on the event type.
- Echoes fade over time or can be "Exorcised" (removed via action).

**Why:** Reinforces "Every world remembers". Makes history tangible on the map. Adds texture to the colony layout (haunted rooms).

---

## 2. Dependencies

- `010` Chronicle System (Source of events)
- `003` Population Basics (Pops to react)
- `031` Pop Morale (Mood effects)

---

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;
    use crate::layer1::chronicle::{ChronicleEvent, Importance};
    use crate::layer1::map::GridPosition;
    use crate::layer1::pop::{Pop, PopState};
    use crate::layer1::morale::Mood;

    #[test]
    fn test_high_importance_event_creates_echo_source() {
        let mut world = World::new();
        world.init_resource::<Events<ChronicleEvent>>();
        // Register systems
        let mut schedule = Schedule::default();
        schedule.add_systems(create_echo_source_system);

        // Act: Send a Major event with a location
        world.send_event(ChronicleEvent {
            text: "Miner Bob died here.".to_string(),
            importance: Importance::Major,
            location: Some(GridPosition { x: 10, y: 10 }),
            ..Default::default()
        });

        // Run system to process event
        schedule.run(&mut world);

        // Assert: EchoSource created at 10,10
        let sources = world.query::<&EchoSource>().iter(&world).collect::<Vec<_>>();
        assert_eq!(sources.len(), 1);
        let source_pos = world.get::<GridPosition>(sources[0].0).unwrap();
        assert_eq!(*source_pos, GridPosition { x: 10, y: 10 });
    }

    #[test]
    fn test_echo_manifestation_cycle() {
        let mut world = World::new();
        // Setup source ready to manifest
        world.spawn((
            EchoSource {
                event_id: "evt_1".to_string(),
                intensity: 1.0,
                frequency: 10, // Ticks
                timer: 9,
                event_type: EchoType::Tragedy,
            },
            GridPosition { x: 5, y: 5 },
        ));

        let mut schedule = Schedule::default();
        schedule.add_systems(manifest_echo_system);

        // Tick 1: Timer hits 10, should spawn Echo
        schedule.run(&mut world);

        let echoes = world.query::<&Echo>().iter(&world).collect::<Vec<_>>();
        assert_eq!(echoes.len(), 1);
        let echo = echoes[0];
        assert_eq!(echo.event_type, EchoType::Tragedy);
    }

    #[test]
    fn test_pop_reaction_to_echo() {
        let mut world = World::new();

        // Spawn Echo
        let echo_pos = GridPosition { x: 2, y: 2 };
        world.spawn((
            Echo { event_type: EchoType::Tragedy, duration: 5 },
            echo_pos,
        ));

        // Spawn Pop nearby (within range 2)
        let pop = world.spawn((
            Pop,
            GridPosition { x: 2, y: 3 }, // Distance 1
            PopState::Idle,
            Mood::default(),
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(echo_reaction_system);

        schedule.run(&mut world);

        // Assert: Pop is distracted and mood affected
        let state = world.get::<PopState>(pop).unwrap();
        // Assuming PopState::Distracted exists or similar state
        assert!(matches!(state, PopState::Distracted(_)));

        let mood = world.get::<Mood>(pop).unwrap();
        // Verify stress increased (exact value depends on implementation)
        assert!(mood.stress > 0.0);
    }
}
```

---

## 4. GREEN Phase: Minimal Implementation

### Components

```rust
// src/layer1/echo.rs

use bevy_ecs::prelude::*;
use crate::layer1::map::GridPosition;

#[derive(Component, Debug)]
pub struct EchoSource {
    pub event_id: String,
    pub intensity: f32, // Fade over years
    pub frequency: u32, // Ticks between manifestations
    pub timer: u32,
    pub event_type: EchoType,
}

#[derive(Component, Debug)]
pub struct Echo {
    pub event_type: EchoType,
    pub duration: u32, // Ticks to exist
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EchoType {
    Tragedy, // Deaths -> Stress
    Triumph, // Success -> Inspiration
    Mystery, // Discovery -> Curiosity
}
```

### Systems

```rust
// src/layer1/echo.rs

pub fn create_echo_source_system(
    mut commands: Commands,
    mut events: EventReader<ChronicleEvent>,
) {
    for event in events.read() {
        if event.importance >= Importance::Major && event.location.is_some() {
            commands.spawn((
                EchoSource {
                    event_id: event.id.clone(),
                    intensity: 1.0,
                    frequency: 1000, // Example: once per day/week
                    timer: 0,
                    event_type: determine_type(&event),
                },
                event.location.unwrap(),
            ));
        }
    }
}

pub fn manifest_echo_system(
    mut commands: Commands,
    mut query: Query<(&mut EchoSource, &GridPosition)>,
) {
    for (mut source, pos) in &mut query {
        source.timer += 1;
        if source.timer >= source.frequency {
            source.timer = 0;
            // Spawn ephemeral Echo
            commands.spawn((
                Echo {
                    event_type: source.event_type,
                    duration: 100,
                },
                *pos, // Same location
            ));
        }
    }
}

pub fn echo_reaction_system(
    mut commands: Commands,
    echoes: Query<(&Echo, &GridPosition)>,
    mut pops: Query<(Entity, &GridPosition, &mut PopState, &mut Mood), Without<Echo>>,
) {
    for (echo, echo_pos) in &echoes {
        for (pop_entity, pop_pos, mut state, mut mood) in &mut pops {
            // Use existing distance method
            if pop_pos.distance_chebyshev(*echo_pos) <= 2 {
                // Reaction logic
                *state = PopState::Distracted(10); // Pause current task
                match echo.event_type {
                    EchoType::Tragedy => mood.stress += 5.0,
                    EchoType::Triumph => mood.morale += 5.0,
                    _ => {},
                }
            }
        }
    }
}

fn determine_type(event: &ChronicleEvent) -> EchoType {
    // Logic to map event tags/templates to EchoType
    EchoType::Mystery // Default
}
```

---

## 5. REFACTOR Phase: Quality & Design

- **Performance:** Spatial hashing for `echo_reaction_system` if many echoes/pops.
- **Visuals:** `Echo` entities need a specialized renderer (ghostly shader/transparency).
- **Cleanup:** `Echo` entities must auto-despawn when duration expires (`despawn_echo_system`).
- **Integration:** Add "Exorcise" action to `UtilityAI` to remove `EchoSource` (requires Priest/Shaman role?).
- **Persistence:** Ensure `EchoSource` is serialized in save games.

---

## 6. Acceptance Criteria

- [ ] `EchoSource` spawns from Major Chronicle events.
- [ ] `Echo` entities spawn periodically from sources.
- [ ] Pops react (State change / Mood change) when near Echoes.
- [ ] `cargo test` passes.

---

## 7. Technical Guidance

- Use `bevy::time::Time` for frequency checks in real implementation (vs ticks in tests).
- Ensure `EchoSource` persists in save data.
- `EchoType` logic: Parse Chronicle tags (e.g., `DEATH` -> `Tragedy`).
- `GridPosition::distance_chebyshev` is the standard distance metric.

---

## 8. Questions

- *Builder: Should Echoes block movement?*
- *Architect:* No, they are incorporeal and Pops can walk through them.
- *Builder: Can we harvest Echoes?*
- *Architect:* No, they cannot be interacted with directly.

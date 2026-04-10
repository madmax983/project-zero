# 941 - Bio-Rhythm Desync

## 1. Overview
**Layer:** Layer 1
**Fantasy:** A colony's biological clock shattering due to artificial light and endless work quotas.
**Mechanic:** Pops have a natural circadian rhythm expectation. If they are forced to work night shifts or live in underground sectors without simulated sunlight cycles, their "Sleep Quality" degrades over time, eventually leading to spontaneous micro-sleeps where they drop items or fail tasks.
**Emergence:** You try to squeeze extra production out of a subterranean mining outpost by running three shifts continuously under harsh floodlights. A few months later, half your miners fall asleep while carrying volatile explosives, collapsing a main artery and trapping the other half.
**Tension:** Implementing expensive, power-hungry artificial day/night cycles in your dark colonies, versus wringing every drop of efficiency out of a 24-hour cycle and risking catastrophic, exhaustion-driven accidents.

## 2. Dependencies
- Needs Layer 1 `Pop` entity with `Needs` (specifically `Rest`).
- Needs Layer 1 `Environment` or `Lighting` tracking to determine if a tile simulates a day/night cycle.
- Needs the existing Layer 1 `Assignment` or `Task` system to interrupt workflows.

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_sleep_quality_degrades_in_desynced_environment() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_systems(Update, bio_rhythm_desync_system);

        // Pop in an environment lacking a proper circadian cycle (e.g. static floodlights)
        let pop = app.world_mut().spawn((
            Pop,
            CircadianRhythm { quality: 100.0 },
            EnvironmentalLighting { has_cycle: false },
        )).id();

        // Act
        app.update();

        // Assert
        let rhythm = app.world().get::<CircadianRhythm>(pop).unwrap();
        assert!(rhythm.quality < 100.0, "Circadian rhythm quality should degrade without a lighting cycle");
    }

    #[test]
    fn test_micro_sleep_interrupts_tasks() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_event::<MicroSleepEvent>()
            .add_systems(Update, trigger_micro_sleep_system);

        // Pop with critically low rhythm quality
        let pop = app.world_mut().spawn((
            Pop,
            CircadianRhythm { quality: 5.0 },
            CurrentTask { name: "Haul Explosives".into() },
        )).id();

        // Act
        app.update();

        // Assert
        let events = app.world().resource::<Events<MicroSleepEvent>>();
        let mut reader = events.get_reader();
        let sleep_events: Vec<_> = reader.read(events).collect();

        assert_eq!(sleep_events.len(), 1, "A micro-sleep event should be triggered");
        assert_eq!(sleep_events[0].entity, pop);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use rand::Rng;

#[derive(Component)]
pub struct Pop;

#[derive(Component)]
pub struct CircadianRhythm {
    pub quality: f32, // 0.0 to 100.0
}

#[derive(Component)]
pub struct EnvironmentalLighting {
    pub has_cycle: bool,
}

#[derive(Component)]
pub struct CurrentTask {
    pub name: String,
}

#[derive(Event)]
pub struct MicroSleepEvent {
    pub entity: Entity,
}

pub fn bio_rhythm_desync_system(
    mut query: Query<(&EnvironmentalLighting, &mut CircadianRhythm)>,
    time: Res<Time>,
) {
    let dt = time.delta_seconds();
    let degrade_rate = 1.0;
    let recovery_rate = 2.0;

    for (lighting, mut rhythm) in query.iter_mut() {
        if !lighting.has_cycle {
            rhythm.quality -= degrade_rate * dt;
        } else {
            rhythm.quality += recovery_rate * dt;
        }
        rhythm.quality = rhythm.quality.clamp(0.0, 100.0);
    }
}

pub fn trigger_micro_sleep_system(
    query: Query<(Entity, &CircadianRhythm), With<CurrentTask>>,
    mut events: EventWriter<MicroSleepEvent>,
) {
    let mut rng = rand::thread_rng();
    let threshold = 20.0; // Quality below this risks micro-sleep

    for (entity, rhythm) in query.iter() {
        if rhythm.quality < threshold {
            // Chance increases as quality drops
            let chance = (threshold - rhythm.quality) / threshold * 0.1; // Max 10% chance per tick
            if rng.gen::<f32>() < chance {
                events.send(MicroSleepEvent { entity });
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Code Smells**: Relying on `EnvironmentalLighting` directly on the Pop is slightly messy. Pops should probably inherit their lighting context from the tile or room they are currently located in (or working in).
- **Refactoring**: Move the lighting check to cross-reference the `Grid` or `Room` the Pop is standing in.
- **Integration**: `MicroSleepEvent` needs to be consumed by the `task_execution_system` to actually abort the `CurrentTask` and potentially trigger an item drop or explosion.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Pops without circadian lighting lose sleep quality over time.
- [ ] Pops with low sleep quality emit `MicroSleepEvent`.

## 7. Technical Guidance
- **Integration**: To truly sell the fantasy, hook `MicroSleepEvent` into the `Inventory` system. If a Pop micro-sleeps while hauling a `Volatile` item, roll a chance for it to detonate or take damage.
- **UI**: Add a status icon (e.g., "Bleary-Eyed") to Pops suffering from severe Desync so the player understands why tasks are failing.
- **Lore**: A severe micro-sleep accident should generate a Chronicle event (e.g., `PopDiedInAccidentEvent`) highlighting the lack of natural light.

## 8. Questions
*Builder: add questions here if spec is unclear.*

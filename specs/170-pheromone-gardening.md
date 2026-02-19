# 170: Pheromone Gardening

## 1. Overview

**Pheromone Gardening** allows players to utilize alien flora not just as a resource or threat, but as a tool for social engineering. Certain flora species emit **Pheromones** that apply temporary `MoodModifier`s to nearby Pops.

Some advanced flora are **Reactive**: they change their emission based on environmental factors (e.g., high pollution) or social factors (e.g., low colony morale), acting as organic sensors or mood regulators.

## 2. Dependencies

- `092` — Antagonistic Flora (Base entity structure for plants)
- `031` — Pop Morale (Target for mood effects)
- `063` — Atmospheric Simulation (For pollution triggers)

## 3. RED Phase: Tests First

Write these tests in `src/layer1/pheromone_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::map::GridPosition;
    use crate::layer1::pop::Pop;
    use crate::layer1::morale::{Morale, MoodModifier};
    use crate::layer1::pheromone::{PheromoneEmitter, PheromoneEffect, ReactiveEmitter, TriggerType, pheromone_emission_system, reactive_emitter_system};
    use crate::layer1::atmosphere::Pollution; // Assuming from 063/049

    #[test]
    fn test_pheromone_application() {
        let mut world = World::new();

        // Spawn Pop at (0,0)
        let pop = world.spawn((
            Pop,
            GridPosition { x: 0, y: 0 },
            Morale::default(),
        )).id();

        // Spawn Emitter at (0,1) - Range 2
        world.spawn((
            GridPosition { x: 0, y: 1 },
            PheromoneEmitter {
                radius: 2,
                effect: PheromoneEffect {
                    label: "Calm Scent".to_string(),
                    value: 0.1,
                    duration: 10,
                },
                interval: 1,
                timer: 0,
            },
        ));

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(pheromone_emission_system);
        schedule.run(&mut world);

        // Verify Pop received modifier
        let morale = world.get::<Morale>(pop).unwrap();
        let modifier = morale.modifiers.iter().find(|m| m.label == "Calm Scent");
        assert!(modifier.is_some());
        assert!((modifier.unwrap().value - 0.1).abs() < f32::EPSILON);
    }

    #[test]
    fn test_pheromone_range_limit() {
        let mut world = World::new();

        // Spawn Pop at (0,0)
        let pop = world.spawn((
            Pop,
            GridPosition { x: 0, y: 0 },
            Morale::default(),
        )).id();

        // Spawn Emitter at (0,5) - Range 2 (Too far)
        world.spawn((
            GridPosition { x: 0, y: 5 },
            PheromoneEmitter {
                radius: 2,
                effect: PheromoneEffect {
                    label: "Calm Scent".to_string(),
                    value: 0.1,
                    duration: 10,
                },
                interval: 1,
                timer: 0,
            },
        ));

        let mut schedule = Schedule::default();
        schedule.add_systems(pheromone_emission_system);
        schedule.run(&mut world);

        let morale = world.get::<Morale>(pop).unwrap();
        assert!(morale.modifiers.is_empty());
    }

    #[test]
    fn test_reactive_emitter_pollution_trigger() {
        let mut world = World::new();

        // Setup Pollution resource
        world.insert_resource(Pollution { level: 100.0 }); // High pollution

        // Spawn Reactive Emitter
        let emitter = world.spawn((
            GridPosition { x: 0, y: 0 },
            PheromoneEmitter {
                radius: 2,
                effect: PheromoneEffect {
                    label: "Clean Scent".to_string(),
                    value: 0.1,
                    duration: 10,
                },
                interval: 1,
                timer: 0,
            },
            ReactiveEmitter {
                trigger: TriggerType::HighPollution(50.0),
                active_effect: PheromoneEffect {
                    label: "Toxic Warning".to_string(), // Changes to this
                    value: -0.2,
                    duration: 20,
                },
                base_effect: PheromoneEffect {
                    label: "Clean Scent".to_string(),
                    value: 0.1,
                    duration: 10,
                },
            }
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(reactive_emitter_system);
        schedule.run(&mut world);

        // Check if emitter component updated
        let emitter_comp = world.get::<PheromoneEmitter>(emitter).unwrap();
        assert_eq!(emitter_comp.effect.label, "Toxic Warning");
        assert!((emitter_comp.effect.value - -0.2).abs() < f32::EPSILON);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

### 1. Define Components (`src/layer1/pheromone.rs`)

```rust
use bevy_ecs::prelude::*;
use crate::layer1::morale::MoodModifier;

#[derive(Clone, Debug)]
pub struct PheromoneEffect {
    pub label: String,
    pub value: f32,
    pub duration: u32,
}

#[derive(Component)]
pub struct PheromoneEmitter {
    pub radius: u32,
    pub effect: PheromoneEffect,
    pub interval: u32, // Ticks between emissions
    pub timer: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TriggerType {
    HighPollution(f32),
    LowMorale(f32),
}

#[derive(Component)]
pub struct ReactiveEmitter {
    pub trigger: TriggerType,
    pub active_effect: PheromoneEffect,
    pub base_effect: PheromoneEffect,
}
```

### 2. Implement Systems

```rust
use crate::layer1::map::GridPosition;
use crate::layer1::pop::Pop;
use crate::layer1::morale::Morale;
use crate::layer1::atmosphere::Pollution;

pub fn pheromone_emission_system(
    mut emitters: Query<(&mut PheromoneEmitter, &GridPosition)>,
    mut pops: Query<(&GridPosition, &mut Morale), With<Pop>>,
) {
    for (mut emitter, emitter_pos) in emitters.iter_mut() {
        if emitter.timer > 0 {
            emitter.timer -= 1;
            continue;
        }
        emitter.timer = emitter.interval;

        // Apply to Pops in range
        for (pop_pos, mut morale) in pops.iter_mut() {
            let dx = (emitter_pos.x - pop_pos.x).abs();
            let dy = (emitter_pos.y - pop_pos.y).abs();
            if dx <= emitter.radius as i32 && dy <= emitter.radius as i32 {
                // Check if modifier already exists to avoid stacking (optional logic)
                // For MVP, we just push. Morale system might handle stacking or decay.
                morale.add_modifier(MoodModifier {
                    label: emitter.effect.label.clone(),
                    value: emitter.effect.value,
                    duration: emitter.effect.duration,
                });
            }
        }
    }
}

pub fn reactive_emitter_system(
    mut query: Query<(&mut PheromoneEmitter, &ReactiveEmitter)>,
    pollution: Option<Res<Pollution>>,
    // morale_avg: Option<Res<AverageMorale>>, // If implemented
) {
    let pollution_level = pollution.map(|p| p.level).unwrap_or(0.0);

    for (mut emitter, reactive) in query.iter_mut() {
        let active = match reactive.trigger {
            TriggerType::HighPollution(threshold) => pollution_level > threshold,
            TriggerType::LowMorale(_) => false, // Placeholder
        };

        if active {
            emitter.effect = reactive.active_effect.clone();
        } else {
            emitter.effect = reactive.base_effect.clone();
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design

- **Optimization**: Use a spatial index (Grid/Quadtree) instead of iterating all Pops for every Emitter. This is O(N*M) complexity.
- **Visuals**: Add particle effects (`ParticleEmitter`) that match the Pheromone color/type.
- **Stacking**: Prevent a Pop standing next to 10 flowers from getting +1.0 Morale instantly. Add a `Cooldown` or `Immunity` timer for specific pheromones on the Pop.
- **Integration**: Add "Planting" actions to allow players to move/place these flora (if they are domesticable).

## 6. Acceptance Criteria

- [ ] `PheromoneEmitter` component exists.
- [ ] Pops within radius receive the mood modifier.
- [ ] Pops outside radius do not receive the modifier.
- [ ] `ReactiveEmitter` correctly switches effects based on trigger (Pollution).
- [ ] Tests pass.

## 7. Technical Guidance

- Use `Pollution` resource if available. If not, mock it or use `ColonyResources` as a proxy trigger for testing.
- The `MoodModifier` label should be unique enough to identify the source (e.g., "Scent: Rose").

# 131 - Bioluminescent Flora

## 1. Overview
Adds a new type of flora entity that emits light during the night. These plants provide natural illumination, reducing the need for artificial light in early game or wild areas, but may be hazardous or simply decorative.

This spec introduces the `Bioluminescent` component, which toggles a `LightSource` component on the same entity based on the `DayNightCycle`.

## 2. Dependencies
- `053` Lighting System (LightSource component)
- `065` Day/Night Cycle (DayNightCycle resource)
- `092` Antagonistic Flora (Flora entity structure)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::lighting::{LightSource, LightMap, AmbientLight};
    use crate::layer1::day_night::{DayNightCycle, TimeOfDay};
    use crate::layer1::flora::{Flora, FloraType};
    use crate::layer1::map::GridPosition;
    use bevy_ecs::prelude::*;

    #[test]
    fn test_bioluminescent_flora_emits_light_at_night() {
        let mut world = World::new();
        let mut schedule = Schedule::default();
        schedule.add_systems(update_bioluminescence_system);

        // Setup Resources
        world.insert_resource(DayNightCycle {
            time_of_day: TimeOfDay::Night,
            ..Default::default()
        });

        // Spawn Bioluminescent Flora
        let flora = world.spawn((
            Flora::default(),
            GridPosition { x: 5, y: 5 },
            Bioluminescent {
                color: (0, 255, 255),
                radius: 5.0,
                intensity: 0.8,
            },
        )).id();

        // Run System
        schedule.run(&mut world);

        // Assert: Entity has LightSource
        let light = world.get::<LightSource>(flora);
        assert!(light.is_some(), "Bioluminescent flora should have LightSource at night");

        let light = light.unwrap();
        assert_eq!(light.intensity, 0.8);
        assert_eq!(light.radius, 5.0);
        assert_eq!(light.color, (0, 255, 255));
    }

    #[test]
    fn test_bioluminescent_flora_dormant_during_day() {
        let mut world = World::new();
        let mut schedule = Schedule::default();
        schedule.add_systems(update_bioluminescence_system);

        // Setup Resources (Day)
        world.insert_resource(DayNightCycle {
            time_of_day: TimeOfDay::Day,
            ..Default::default()
        });

        // Spawn Flora WITH LightSource (simulating leftover from night)
        let flora = world.spawn((
            Flora::default(),
            GridPosition { x: 5, y: 5 },
            Bioluminescent {
                color: (0, 255, 255),
                radius: 5.0,
                intensity: 0.8,
            },
            LightSource {
                radius: 5.0,
                intensity: 0.8,
                color: (0, 255, 255),
            },
        )).id();

        // Run System
        schedule.run(&mut world);

        // Assert: LightSource removed
        let light = world.get::<LightSource>(flora);
        assert!(light.is_none(), "Bioluminescent flora should NOT have LightSource during day");
    }

    #[test]
    fn test_harvesting_removes_light_source() {
        // This is implicit in ECS (despawn removes components), but integration test validates logic.
        let mut world = World::new();

        // Spawn Flora
        let flora = world.spawn((
            Flora::default(),
            Bioluminescent::default(),
            LightSource::default(), // Active
        )).id();

        // Despawn
        world.despawn(flora);

        // Verify entity is gone (and thus light source is gone from queries)
        assert!(world.get_entity(flora).is_err());
    }
}
```

## 4. GREEN Phase: Minimal Implementation

### 4.1 Component Definition
In `src/layer1/flora.rs`:

```rust
#[derive(Component, Default, Clone)]
pub struct Bioluminescent {
    pub color: (u8, u8, u8),
    pub radius: f32,
    pub intensity: f32,
}
```

### 4.2 System Implementation
In `src/layer1/flora.rs`:

```rust
use crate::layer1::lighting::LightSource;
use crate::layer1::day_night::{DayNightCycle, TimeOfDay};

pub fn update_bioluminescence_system(
    mut commands: Commands,
    cycle: Res<DayNightCycle>,
    query: Query<(Entity, &Bioluminescent, Option<&LightSource>)>,
) {
    let is_night = cycle.time_of_day == TimeOfDay::Night || cycle.time_of_day == TimeOfDay::Dusk;

    for (entity, bio, light_source) in &query {
        if is_night {
            if light_source.is_none() {
                commands.entity(entity).insert(LightSource {
                    radius: bio.radius,
                    intensity: bio.intensity,
                    color: bio.color,
                });
            }
        } else {
            if light_source.is_some() {
                commands.entity(entity).remove::<LightSource>();
            }
        }
    }
}
```

### 4.3 Integration
- Register `update_bioluminescence_system` in `src/layer1/mod.rs` or `setup.rs`.
- Add `GlowShroom` variant to `FloraType` enum in `src/layer1/flora.rs` (optional, for flavor).

## 5. REFACTOR Phase: Quality & Design

- **Optimization**: Use `Changed<DayNightCycle>` (if it were a component, but it's a Resource, so `Res<DayNightCycle>` change detection is trickier without `Changed<Res<T>>` which doesn't exist directly in older Bevy versions, but `resource_changed` condition works).
- **Optimization**: The current loop runs every frame. Since day/night changes rarely, we should gate this system with a run condition: `run_if(resource_changed::<DayNightCycle>())`.
- **Polish**: Add a gentle fade-in/fade-out by adjusting `intensity` over time instead of hard add/remove. (Future work).

## 6. Acceptance Criteria
- [ ] New `Bioluminescent` component exists.
- [ ] `Flora` entities with this component gain `LightSource` at night.
- [ ] `Flora` entities lose `LightSource` during the day.
- [ ] `cargo test` passes.

# 123: Technological Rituals

## 1. Overview

Implements **Technological Rituals**, a mechanic where aging or jury-rigged machines (specifically Heirlooms) develop **Quirks** (negative traits). Pops must perform specific **Ritual** actions to appease the "Machine Spirit" and temporarily suppress these quirks.

This adds flavor to the late-game maintenance loop for ancient technology, turning it into a "pet" that needs care rather than just a building that needs resources.

## 2. Dependencies

- `070` — Heirloom Tech (Heirlooms are the primary targets).
- `112` — Maintenance Debt (for the concept of degradation).
- `009` — Job System (for `Ritual` action).

## 3. RED Phase: Tests First

Write these tests in `src/layer1/rituals_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::heirloom::Heirloom;
    use crate::layer1::rituals::{MachineSpirit, Quirk, QuirkType, perform_ritual, spirit_decay_system};
    use crate::layer1::GridPosition;

    #[test]
    fn test_machine_spirit_initialization() {
        let mut world = World::new();
        let entity = world.spawn((
            Building { building_type: BuildingType::AncientReactor },
            Heirloom,
            MachineSpirit::default(), // Should start Happy/Neutral
        )).id();

        let spirit = world.get::<MachineSpirit>(entity).unwrap();
        assert!(spirit.anger <= 0.0);
    }

    #[test]
    fn test_spirit_decay_increases_anger() {
        let mut world = World::new();
        let entity = world.spawn((
            Building { building_type: BuildingType::AncientReactor },
            Heirloom,
            MachineSpirit { anger: 0.0, ..Default::default() },
        )).id();

        // Run decay system
        spirit_decay_system(&mut world);

        let spirit = world.get::<MachineSpirit>(entity).unwrap();
        assert!(spirit.anger > 0.0, "Spirit should get angry over time");
    }

    #[test]
    fn test_high_anger_causes_quirk() {
        let mut world = World::new();
        let entity = world.spawn((
            Building { building_type: BuildingType::AncientReactor },
            Heirloom,
            MachineSpirit { anger: 100.0, ..Default::default() }, // Furious
        )).id();

        // Run quirk generation system (mocked or actual)
        crate::layer1::rituals::quirk_generation_system(&mut world);

        let quirk = world.get::<Quirk>(entity);
        assert!(quirk.is_some(), "High anger should manifest a Quirk");
    }

    #[test]
    fn test_ritual_reduces_anger() {
        let mut world = World::new();
        let entity = world.spawn((
            Building { building_type: BuildingType::AncientReactor },
            Heirloom,
            MachineSpirit { anger: 50.0, ..Default::default() },
            Quirk { quirk_type: QuirkType::Glitchy },
        )).id();

        // Perform ritual
        perform_ritual(&mut world, entity);

        let spirit = world.get::<MachineSpirit>(entity).unwrap();
        assert!(spirit.anger < 50.0, "Ritual should reduce anger");

        // Quirk might be removed or suppressed
        let quirk = world.get::<Quirk>(entity);
        assert!(quirk.is_none(), "Ritual should remove/suppress active Quirk");
    }

    #[test]
    fn test_glitchy_quirk_stops_production() {
        // This test requires integrating with production systems,
        // but unit test can check the flag.
        let quirk = Quirk { quirk_type: QuirkType::Glitchy };
        assert!(quirk.stops_production(), "Glitchy quirk should stop production");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

### 1. Define Components

In `src/layer1/rituals.rs`:

```rust
use bevy_ecs::prelude::*;

#[derive(Component, Debug, Clone, Default)]
pub struct MachineSpirit {
    pub anger: f32, // 0.0 to 100.0
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuirkType {
    Glitchy,      // Stops production randomly
    Overheating,  // Risk of fire
    Demanding,    // Increases anger faster
}

#[derive(Component, Debug, Clone)]
pub struct Quirk {
    pub quirk_type: QuirkType,
}

impl Quirk {
    pub fn stops_production(&self) -> bool {
        matches!(self.quirk_type, QuirkType::Glitchy)
    }
}
```

### 2. Implement Systems

In `src/layer1/rituals.rs`:

```rust
pub fn spirit_decay_system(mut query: Query<&mut MachineSpirit>) {
    for mut spirit in query.iter_mut() {
        spirit.anger = (spirit.anger + 0.1).min(100.0);
    }
}

pub fn quirk_generation_system(
    mut commands: Commands,
    query: Query<(Entity, &MachineSpirit), Without<Quirk>>,
) {
    for (entity, spirit) in query.iter() {
        if spirit.anger > 80.0 {
            // Manifest Quirk
            commands.entity(entity).insert(Quirk {
                quirk_type: QuirkType::Glitchy, // Default for MVP
            });
        }
    }
}

pub fn perform_ritual(world: &mut World, target: Entity) {
    if let Some(mut spirit) = world.get_mut::<MachineSpirit>(target) {
        spirit.anger = (spirit.anger - 50.0).max(0.0);
    }
    // Remove Quirk if anger is low enough
    if let Some(spirit) = world.get::<MachineSpirit>(target) {
        if spirit.anger < 50.0 {
            world.entity_mut(target).remove::<Quirk>();
        }
    }
}
```

### 3. Integrate with Heirloom

In `src/layer1/heirloom.rs` or setup:
- Add `MachineSpirit::default()` to all `Heirloom` entities spawned.

## 5. REFACTOR Phase: Quality & Design

- **Flavor**: Rename `anger` to `Entropy` or `Discord`? "Machine Spirit Anger" fits the theme well.
- **Ritual Types**: Different quirks might require different rituals (e.g., "Percussive Maintenance" vs "Anoint with Oil").
- **UI**: Display the "Mood" of the machine in the inspector.
- **Integration**: Update `production_system` to check `Quirk::stops_production()` before producing.

## 6. Acceptance Criteria

- [ ] `MachineSpirit` component tracks anger.
- [ ] Anger increases over time (decay).
- [ ] High anger adds a `Quirk` component.
- [ ] `Quirk` can stop production (verified in logic).
- [ ] `perform_ritual` reduces anger and removes Quirk.
- [ ] Tests pass.

## 7. Technical Guidance

- Ensure `perform_ritual` is called by an actual Pop action (needs a new `ActionType::Ritual`).
- For MVP, just expose `perform_ritual` as a public function and test it. The Pop AI integration can be a follow-up or part of the `JobSystem` update if scope allows, but strictly speaking, the *mechanic* is the priority here.

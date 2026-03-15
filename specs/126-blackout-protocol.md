# 126: Blackout Protocol

## Overview

The **Blackout Protocol** is a global emergency system that allows the player (or automated systems) to instantly cut all power to the colony.
This serves multiple purposes:
1.  **Heat Management**: Quickly reduces heat generation from machines during critical overheating events.
2.  **Reset Grid**: Clears overloaded states (from Spec 125).
3.  **Stealth**: (Future) Avoid detection by hostile entities sensitive to EM emissions.
4.  **Panic**: Plunging the colony into darkness causes stress, especially for specific traits.

Mechanically, it introduces a global `BlackoutProtocol` resource. When active, all `PowerConsumer` components are forced inactive, and `LightSource` components attached to consumers stop emitting light.

## Dependencies

- `042` — Energy System (Base power logic)
- `053` — Lighting System (Light levels and penalties)
- `125` — Grid Instability (Grid overload context)
- `031` — Pop Morale (Stress mechanics)
- `084` — Pop Traits (Personality quirks)

## RED Phase: Tests First

Write these tests in `src/layer1/energy/blackout_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::energy::{PowerConsumer, PowerSource, power_grid_system};
    use crate::layer1::lighting::{LightSource, update_lighting_system, LightMap, AmbientLight};
    use crate::layer1::map::GridPosition;
    use crate::layer1::morale::{Morale, MoodModifier}; // Or appropriate module
    use crate::layer1::needs::Needs;
    use crate::layer1::pop::Pop;
    use crate::layer1::traits::{Trait, Traits};
    use std::collections::HashSet;

    // Define the resource (will be added in Green phase)
    #[derive(Resource, Default)]
    pub struct BlackoutProtocol {
        pub active: bool,
    }

    #[test]
    fn test_blackout_cuts_power() {
        let mut world = World::new();
        world.insert_resource(BlackoutProtocol { active: true });

        // Generator (Output 10)
        world.spawn((
            PowerSource { output: 10.0, active: true },
            GridPosition { x: 0, y: 0 },
        ));

        // Consumer (Demand 5)
        let consumer = world.spawn((
            PowerConsumer { demand: 5.0, active: true }, // Starts active
            GridPosition { x: 0, y: 1 },
        )).id();

        // Run grid system
        // Note: You might need to register the resource if the system expects it
        power_grid_system(&mut world);

        let state = world.get::<PowerConsumer>(consumer).unwrap();
        assert!(!state.active, "Consumer should be inactive during blackout");
    }

    #[test]
    fn test_blackout_disables_lights() {
        let mut world = World::new();
        world.insert_resource(LightMap::new(10, 10));
        world.insert_resource(AmbientLight { level: 0.0 }); // Pitch black

        // Light Source WITH PowerConsumer (e.g. Lamp)
        // Should be disabled if PowerConsumer is inactive
        world.spawn((
            LightSource { radius: 5.0, intensity: 1.0, color: (255, 255, 255) },
            PowerConsumer { demand: 1.0, active: false }, // Inactive due to blackout
            GridPosition { x: 5, y: 5 },
        ));

        // Light Source WITHOUT PowerConsumer (e.g. Torch)
        // Should remain active
        world.spawn((
            LightSource { radius: 5.0, intensity: 1.0, color: (255, 200, 100) },
            GridPosition { x: 2, y: 2 },
        ));

        update_lighting_system(&mut world);

        let map = world.resource::<LightMap>();

        // Lamp at 5,5 should be dark
        assert!(map.get(5, 5) < 0.1, "Powered light should be off");

        // Torch at 2,2 should be lit
        assert!(map.get(2, 2) > 0.5, "Unpowered light (torch) should be on");
    }

    #[test]
    fn test_panic_in_darkness() {
        // Test Trait-based stress response
        let mut world = World::new();
        world.insert_resource(LightMap::new(10, 10));
        // Pitch black
        {
            let mut map = world.resource_mut::<LightMap>();
            map.tiles.fill(0.0);
        }

        // Anxious Pop (Scared of dark)
        let anxious = world.spawn((
            Pop,
            GridPosition { x: 0, y: 0 },
            Traits(HashSet::from([Trait::Anxious])),
            Needs { leisure: 1.0, ..Default::default() },
        )).id();

        // NightOwl Pop (Likes dark)
        let night_owl = world.spawn((
            Pop,
            GridPosition { x: 0, y: 0 },
            Traits(HashSet::from([Trait::NightOwl])),
            Needs { leisure: 1.0, ..Default::default() },
        )).id();

        // Normal Pop
        let normal = world.spawn((
            Pop,
            GridPosition { x: 0, y: 0 },
            Traits(HashSet::new()),
            Needs { leisure: 1.0, ..Default::default() },
        )).id();

        // Run lighting penalties system (enhanced)
        // Note: System name matches Spec 053
        crate::layer1::lighting::apply_lighting_penalties_system(&mut world);

        let n_anxious = world.get::<Needs>(anxious).unwrap();
        let n_nightowl = world.get::<Needs>(night_owl).unwrap();
        let n_normal = world.get::<Needs>(normal).unwrap();

        // Anxious should lose MORE leisure (stress)
        // Normal loses some
        // NightOwl loses LESS or NONE
        assert!(n_anxious.leisure < n_normal.leisure, "Anxious pop should be more stressed");
        assert!(n_normal.leisure < n_nightowl.leisure, "Normal pop should be more stressed than NightOwl");
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Define Resource

In `src/layer1/energy/mod.rs`:

```rust
#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct BlackoutProtocol {
    pub active: bool,
}
```

### 2. Update `power_grid_system`

Modify `src/layer1/energy/mod.rs`:

```rust
pub fn power_grid_system(world: &mut World) {
    // 0. Check Protocol
    let blackout = world.get_resource::<BlackoutProtocol>().is_some_and(|b| b.active);

    // If blackout is active, force all consumers inactive and return early (or skip calculation)
    if blackout {
        let mut consumers = world.query::<&mut PowerConsumer>();
        for mut consumer in consumers.iter_mut(world) {
            consumer.active = false;
        }
        return;
    }

    // ... existing logic ...
}
```

### 3. Update `update_lighting_system`

Modify `src/layer1/lighting.rs`:

```rust
pub fn update_lighting_system(
    mut light_map: ResMut<LightMap>,
    ambient: Res<AmbientLight>,
    // Add optional PowerConsumer component to query
    sources: Query<(&LightSource, &GridPosition, Option<&crate::layer1::energy::PowerConsumer>)>,
) {
    light_map.tiles.fill(ambient.level);

    for (source, pos, power) in &sources {
        // Check if power exists and is inactive
        if let Some(p) = power {
            if !p.active { continue; }
        }

        // ... existing light spread logic ...
    }
}
```

### 4. Enhance `apply_lighting_penalties_system`

Modify `src/layer1/lighting.rs`:

```rust
use crate::layer1::traits::{Trait, Traits};

pub fn apply_lighting_penalties_system(
    light_map: Res<LightMap>,
    mut pops: Query<(&GridPosition, &mut Speed, &mut Needs, Option<&Traits>)>,
) {
    for (pos, mut speed, mut needs, traits) in &mut pops {
        // Safe cast...
        let light = light_map.get(pos.x as u32, pos.y as u32);

        if light < 0.2 {
            // Speed penalty
            speed.current = speed.base * 0.5;

            // Stress/Morale penalty logic
            let mut stress_factor = 1.0;

            if let Some(t) = traits {
                if t.0.contains(&Trait::Anxious) {
                    stress_factor = 2.0; // Panic!
                } else if t.0.contains(&Trait::NightOwl) {
                    stress_factor = 0.1; // Minimal stress
                }
            }

            // Apply penalty scaled by factor
            // Default penalty 0.005
            let penalty = 0.005 * stress_factor;
            needs.leisure = (needs.leisure - penalty).max(0.0);

        } else {
            speed.current = speed.base;
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **Optimization**: The `power_grid_system` modification to iterate all consumers during blackout is O(N). Ensure this doesn't conflict with other systems attempting to activate them.
- **UI**: Add a `BlackoutProtocol` toggle to the `Inspector` or `Energy` UI overlay.
- **Feedback**: Add a "klaxon" sound or notification when Blackout is engaged.
- **Integration**: Ensure `Battery` logic (Spec 125) also halts during Blackout (no charging/discharging). Batteries are effectively isolated.

## Acceptance Criteria

- [ ] `BlackoutProtocol` resource exists.
- [ ] Activating protocol immediately cuts power to all `PowerConsumer`s.
- [ ] Lights powered by the grid turn off.
- [ ] Unpowered lights (torches) stay on.
- [ ] Pops in darkness suffer stress based on Traits (`Anxious` > Normal > `NightOwl`).
- [ ] Deactivating protocol restores normal grid function.
- [ ] Tests pass.

## Technical Guidance

- Ensure `PowerConsumer` is imported correctly in `lighting.rs`. It might need `use crate::layer1::energy::PowerConsumer`.
- The `power_grid_system` early return is the simplest implementation but verify it doesn't leave the grid map in a dirty state for other systems (e.g. `grid_stats`).
- `Trait::NightOwl` implies comfort in darkness; ensure the math reflects this (low or zero penalty).

## Questions

*Builder: Should Blackout disable Batteries too?*
*Architect:* Yes, the grid disconnects completely, so batteries do not charge or discharge.
*Architect:* Yes, all power flow is completely halted, effectively disconnecting the battery networks.

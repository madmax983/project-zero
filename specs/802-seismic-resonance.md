# Seismic Resonance

## 1. Overview
Industrial expansion isn't just loud; it's physically impactful. Heavy machinery (drills, stampers) generates "Vibrations," a phenomenon distinct from noise. These vibrations travel through the grid, attracting specific xenofauna (like Thumper-Beasts) and destabilizing fragile environmental features (like Singing Crystals). Players must balance industrial throughput against the risk of catastrophic ecological reactions.

## 2. Dependencies
- Layer 1 `TerrainGrid` and pathfinding.
- Existing machine components (e.g., `WorkStation`, `PowerConsumer`).
- `Health` component and damage events.

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::map::{TerrainGrid, GridPosition};
    use crate::layer1::structure::Health;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_event::<VibrationEvent>();
        app.init_resource::<TerrainGrid>();
        app.add_systems(Update, (
            emit_vibration_system,
            process_vibration_resonance_system,
            thumper_beast_attraction_system,
        ));
        app
    }

    #[test]
    fn test_heavy_machinery_emits_vibration() {
        let mut app = setup_app();

        // Spawn active heavy machinery
        let machine = app.world.spawn((
            GridPosition { x: 5, y: 5 },
            HeavyMachinery { active: true, intensity: 10 },
        )).id();

        app.update();

        let vibration_events = app.world.resource::<Events<VibrationEvent>>();
        let mut reader = vibration_events.get_reader();
        let events: Vec<_> = reader.read(&vibration_events).collect();

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].origin, GridPosition { x: 5, y: 5 });
        assert_eq!(events[0].intensity, 10);
    }

    #[test]
    fn test_vibration_shatters_crystals() {
        let mut app = setup_app();

        // Spawn fragile crystal
        let crystal = app.world.spawn((
            GridPosition { x: 6, y: 5 }, // Adjacent
            ResonantCrystal { fragility_threshold: 5 },
            Health { current: 50, max: 50 },
        )).id();

        // Trigger high intensity vibration
        app.world.send_event(VibrationEvent {
            origin: GridPosition { x: 5, y: 5 },
            intensity: 10,
        });

        app.update();

        // Crystal should take damage/shatter due to threshold exceeded
        let health = app.world.get::<Health>(crystal).unwrap();
        assert!(health.current < 50);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer1::map::GridPosition;
use crate::layer1::structure::Health;

#[derive(Component)]
pub struct HeavyMachinery {
    pub active: bool,
    pub intensity: u32,
}

#[derive(Component)]
pub struct ResonantCrystal {
    pub fragility_threshold: u32,
}

#[derive(Event)]
pub struct VibrationEvent {
    pub origin: GridPosition,
    pub intensity: u32,
}

pub fn emit_vibration_system(
    query: Query<(&GridPosition, &HeavyMachinery)>,
    mut vibration_events: EventWriter<VibrationEvent>,
) {
    for (pos, machine) in query.iter() {
        if machine.active {
            vibration_events.send(VibrationEvent {
                origin: *pos,
                intensity: machine.intensity,
            });
        }
    }
}

pub fn process_vibration_resonance_system(
    mut vibration_events: EventReader<VibrationEvent>,
    mut crystal_query: Query<(&GridPosition, &ResonantCrystal, &mut Health)>,
) {
    for event in vibration_events.read() {
        for (pos, crystal, mut health) in crystal_query.iter_mut() {
            // Simple distance check (Manhattan distance)
            let distance = (pos.x as i32 - event.origin.x as i32).abs()
                         + (pos.y as i32 - event.origin.y as i32).abs();

            // Attenuate intensity by distance
            let effective_intensity = event.intensity.saturating_sub(distance as u32);

            if effective_intensity >= crystal.fragility_threshold {
                // Shatter damage
                health.current = health.current.saturating_sub(50);
            }
        }
    }
}

// Placeholder for the AI attraction system
pub fn thumper_beast_attraction_system() {
    // Logic to modify Thumper-Beast Utility AI targets based on vibration heatmaps
}
```

## 5. REFACTOR Phase: Quality & Design
- **Heatmap:** Instead of discrete events calculated per entity, consider writing vibrations to a `VibrationGrid` resource (similar to atmosphere diffusion) so AI can pathfind toward the epicenter organically.
- **Damping:** Introduce `VibrationDamping` components for structural foundations to allow players to mitigate the effect.

## 6. Acceptance Criteria (Testable!)
- [ ] Active heavy machinery emits `VibrationEvent`s.
- [ ] Resonant crystals take damage when exposed to vibrations exceeding their threshold.
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.

## 7. Technical Guidance
- If implementing the `VibrationGrid`, ensure it decays over time so the map doesn't become permanently "loud" after a machine is turned off.
- Crystals shattering should probably emit a dangerous AoE shrapnel event to harm nearby Pops.

## 8. Questions
*Builder: add questions here if spec is unclear.*

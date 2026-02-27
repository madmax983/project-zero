# 259: Harmonic Mining

## Overview

"Sing the stone to dust."

**Harmonic Mining** introduces a high-tech alternative to mechanical drilling. The **Harmonic Drill** uses focused sonic waves to disintegrate ore instantly within a radius. It is extremely fast and efficient but dangerous.

Every material has a **Resonant Frequency**. If the drill's frequency matches the material of nearby structures (e.g., Glass Greenhouses, Crystal Statues, or even other machinery), those structures suffer massive damage or shatter instantly.

Players must tune the drill to match the Ore they want to mine, while ensuring they don't accidentally match the frequency of their own base.

## Dependencies

- `018` — Mining Resources (Resource entities)
- `045` — Structure Durability (Damage mechanics)
- `060` — Acoustic Simulation (Concept of sound/frequency)

## RED Phase: Tests First

Write these tests in `src/layer1/tech/harmonic_mining_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::map::GridPosition;
    use crate::layer1::tech::harmonic::{HarmonicDrill, ResonantMaterial, Frequency, process_harmonic_mining};
    use crate::layer1::resources::{ResourceType, ColonyResources};
    use crate::layer1::structure::{Structure, StructureType};
    use crate::layer1::health::Health;

    #[test]
    fn test_drill_mines_matching_ore() {
        let mut world = World::new();
        world.insert_resource(ColonyResources::default());

        // Spawn Drill tuned to Iron (Frequency 100)
        let drill = world.spawn((
            HarmonicDrill {
                frequency: Frequency(100),
                radius: 3.0,
                active: true,
            },
            GridPosition { x: 5, y: 5 },
        )).id();

        // Spawn Ore Node (Iron) nearby
        let ore = world.spawn((
            ResonantMaterial { frequency: Frequency(100), material_type: ResourceType::Iron },
            GridPosition { x: 6, y: 5 },
            // Add Ore component or tag if needed by 018
        )).id();

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(process_harmonic_mining);
        schedule.run(&mut world);

        // Ore should be destroyed/mined
        assert!(world.get_entity(ore).is_none());

        // Resources should increase
        let resources = world.resource::<ColonyResources>();
        assert!(resources.get(ResourceType::Iron) > 0.0);
    }

    #[test]
    fn test_drill_shatters_matching_structure() {
        let mut world = World::new();

        // Spawn Drill tuned to Glass (Frequency 200)
        world.spawn((
            HarmonicDrill {
                frequency: Frequency(200),
                radius: 5.0,
                active: true,
            },
            GridPosition { x: 10, y: 10 },
        ));

        // Spawn Greenhouse (Glass) nearby
        let greenhouse = world.spawn((
            Structure { structure_type: StructureType::Greenhouse, ..Default::default() },
            ResonantMaterial { frequency: Frequency(200), material_type: ResourceType::Glass }, // Assuming Glass is a ResourceType or Material enum
            Health { current: 100.0, max: 100.0 },
            GridPosition { x: 12, y: 10 },
        )).id();

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(process_harmonic_mining);
        schedule.run(&mut world);

        // Greenhouse should take massive damage or be destroyed
        let health = world.get::<Health>(greenhouse);
        // Either entity is gone OR health is 0
        if let Some(h) = health {
            assert_eq!(h.current, 0.0);
        } else {
            // Entity despawned implies destruction
            assert!(true);
        }
    }

    #[test]
    fn test_drill_ignores_mismatched_objects() {
        let mut world = World::new();

        // Drill tuned to Iron (100)
        world.spawn((
            HarmonicDrill { frequency: Frequency(100), radius: 3.0, active: true },
            GridPosition { x: 0, y: 0 },
        ));

        // Glass Structure (200)
        let greenhouse = world.spawn((
            ResonantMaterial { frequency: Frequency(200), material_type: ResourceType::Glass },
            Health { current: 100.0, max: 100.0 },
            GridPosition { x: 1, y: 0 },
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(process_harmonic_mining);
        schedule.run(&mut world);

        // Should be unharmed
        let health = world.get::<Health>(greenhouse).unwrap();
        assert_eq!(health.current, 100.0);
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Components

```rust
// src/layer1/tech/harmonic.rs

use bevy_ecs::prelude::*;
use crate::layer1::resources::ResourceType;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Frequency(pub u32);

#[derive(Component, Debug, Clone)]
pub struct HarmonicDrill {
    pub frequency: Frequency,
    pub radius: f32,
    pub active: bool,
}

#[derive(Component, Debug, Clone)]
pub struct ResonantMaterial {
    pub frequency: Frequency,
    pub material_type: ResourceType, // Or a custom Material enum
}
```

### 2. System

```rust
use crate::layer1::map::GridPosition;
use crate::layer1::resources::ColonyResources;
use crate::layer1::health::Health;

pub fn process_harmonic_mining(
    mut commands: Commands,
    drills: Query<(&HarmonicDrill, &GridPosition)>,
    mut materials: Query<(Entity, &ResonantMaterial, &GridPosition, Option<&mut Health>)>,
    mut resources: ResMut<ColonyResources>,
) {
    for (drill, drill_pos) in drills.iter() {
        if !drill.active { continue; }

        for (target_entity, material, target_pos, health) in materials.iter_mut() {
            // Distance check
            if drill_pos.distance_chebyshev(*target_pos) <= drill.radius as i32 {
                // Resonance Check
                if drill.frequency == material.frequency {
                    if let Some(mut h) = health {
                        // It's a structure/unit -> Damage it
                        h.current = 0.0;
                        // Logic to handle destruction event?
                    } else {
                        // It's raw resource -> Mine it
                        resources.add(material.material_type, 10.0); // Arbitrary amount
                        commands.entity(target_entity).despawn();
                    }
                }
            }
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **Frequency Tuning UI**: Drill needs a UI to select target Material (which sets Frequency).
- **Material Lookup**: Instead of hardcoding frequencies on entities, use a lookup `MaterialProperties::get_frequency(ResourceType)`.
- **Collateral Damage**: Resonance shouldn't be binary. Near-matches (e.g. 100 vs 105) should cause *vibration* or minor damage.
- **Visuals**: expanding ring effect when active.

## Acceptance Criteria

- [ ] `HarmonicDrill` mines resources matching its frequency.
- [ ] `HarmonicDrill` destroys structures matching its frequency.
- [ ] Safe radius checks work.
- [ ] Tests pass.

## Technical Guidance

- Use `GridPosition::distance_euclidean` for circle radius, or `chebyshev` for square. Spec says "Radius", usually implies Euclidean but Chebyshev matches the grid better visually for "Area of Effect".
- Ensure `ResonantMaterial` is added to standard prefabs (Walls, Ore Nodes).

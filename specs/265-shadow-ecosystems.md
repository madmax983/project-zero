# 265: Shadow Ecosystems

## 1. Overview

"Technology creates its own nature."

High-tech zones (Servers, Reactors, Shield Generators) spawn invisible **Data Fauna** or **Energy Spectres**. These entities feed on radiation, Wi-Fi signals, or waste heat. They are normally invisible and harmless, but if they "Overfeed" (population density too high), they can cause brownouts, short circuits, or data corruption.

**Mechanic:**
- **Spawn Conditions**: High `EnergyGrid` load or `DataDensity` (from Spec 248) spawns `ShadowEntity`.
- **Visibility**: Only visible with "Spectrometer" sensors or during "Magnetic Storms".
- **Interaction**:
    - `StaticMites`: Eat power. Reduce efficiency.
    - `DataRot`: Corrupts research/blueprints.
    - `VoidEels`: Passive, float through walls, high beauty (if seen).

**Why:** Adds an ecosystem to the sterile, high-tech late game.

## 2. Dependencies

- `042` — Energy System (Grid load)
- `156` — Xeno-Artifacts (Sensor logic base)
- `248` — Infinite Archive (Data density context)

## 3. RED Phase: Tests First

Write these tests in `src/layer1/fauna/shadow_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::fauna::shadow::{ShadowEntity, ShadowType, spawn_shadow_fauna_system, ShadowFeed};
    use crate::layer1::map::GridPosition;
    use crate::layer1::energy::PowerGrid; // Mocked or real

    #[test]
    fn test_high_energy_spawns_static_mites() {
        let mut world = World::new();
        // Setup high energy grid cell at (5,5)
        let mut grid = PowerGrid::default();
        grid.set_load(5, 5, 1000.0); // Very high load
        world.insert_resource(grid);

        // Run spawn system
        let mut schedule = Schedule::default();
        schedule.add_systems(spawn_shadow_fauna_system);
        schedule.run(&mut world);

        // Check for entity
        let mut query = world.query::<(&ShadowEntity, &GridPosition)>();
        let found = query.iter(&world).any(|(e, pos)|
            e.entity_type == ShadowType::StaticMite && pos.x == 5 && pos.y == 5
        );
        assert!(found);
    }

    #[test]
    fn test_shadow_feed_reduces_power_efficiency() {
        // ... logic to verify Mite reduces local machine output
    }

    #[test]
    fn test_visibility_toggle() {
        // Verify they are hidden by default, visible with Sensor
    }
}
```

## 4. GREEN Phase: Minimal Implementation

### 1. Components

```rust
// src/layer1/fauna/shadow.rs

use bevy_ecs::prelude::*;
use crate::layer1::map::GridPosition;
use crate::layer1::energy::PowerGrid;
use rand::Rng;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShadowType {
    StaticMite,
    DataRot,
    VoidEel,
}

#[derive(Component)]
pub struct ShadowEntity {
    pub entity_type: ShadowType,
    pub hunger: f32,
    pub visible: bool,
}

pub fn spawn_shadow_fauna_system(
    mut commands: Commands,
    power: Res<PowerGrid>,
    query: Query<&GridPosition, With<ShadowEntity>>,
) {
    let mut rng = rand::thread_rng();

    // Check for high load spots
    // (Simplified: random sample or iterate known hotspots)
    // If load > Threshold && Random Chance:
    // Spawn ShadowEntity { visible: false }
}

pub fn shadow_visibility_system(
    mut query: Query<&mut ShadowEntity>,
    // Res<Sensors>, Res<Weather>
) {
    // If Sensor active or Storm: visible = true
    // Else: visible = false
}
```

## 5. REFACTOR Phase: Quality & Design

- **Containment**: Can we build "Faraday Cages" to block them?
- **Domestication**: Can `StaticMites` be harvested for "Living Batteries"?
- **UI**: Render them as semi-transparent or distortion shaders when "Invisible" but detected.

## 6. Acceptance Criteria

- [ ] `ShadowEntity` spawns in high-energy zones.
- [ ] Entities are invisible by default.
- [ ] Entities consume/affect the resource they spawned from (Power/Data).
- [ ] Tests pass.

## 7. Technical Guidance

- Use a localized `spawn_chance` based on `PowerGrid` density.
- Do not make them combat enemies (yet). They are pests/environment.

## 8. Questions

*Builder: Do they move?*
*Architect: Yes, random walk towards higher density (gradient descent).*

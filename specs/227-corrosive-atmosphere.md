# 227: Corrosive Atmosphere

## 1. Overview

Some planets have hostile atmospheres that actively degrade exposed structures. This introduces the `CorrosiveAtmosphere` resource, which applies damage over time to any `Structure` entity that is not "Indoors" (covered by a roof or natural terrain).

This adds a survival pressure: Players must build quickly to cover their critical infrastructure, or use expensive "Corrosion Resistant" materials.

## 2. Dependencies

- `specs/045-structure-durability.md` — `Structure` component (HP).
- `specs/004-basic-building.md` — Building entities.
- `specs/063-atmospheric-simulation.md` — `Roof` / Indoor detection logic (if available, otherwise simple `Outdoors` check).

## 3. RED Phase: Tests First

Write these tests in `src/layer1/atmosphere_corrosion_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::structure::Structure;
    use crate::layer1::atmosphere::{CorrosiveAtmosphere, CorrosionResistant, corrosion_damage_system};
    use crate::layer1::map::{Roof, GridPosition}; // Assuming Roof component or map check exists

    #[test]
    fn test_corrosive_atmosphere_resource_exists() {
        let mut world = World::new();
        world.insert_resource(CorrosiveAtmosphere { intensity: 0.5 });

        let atmos = world.get_resource::<CorrosiveAtmosphere>().unwrap();
        assert_eq!(atmos.intensity, 0.5);
    }

    #[test]
    fn test_structure_takes_damage_when_exposed() {
        let mut world = World::new();
        world.insert_resource(CorrosiveAtmosphere { intensity: 1.0 });

        let structure = world.spawn((
            Structure { current_hp: 100.0, max_hp: 100.0 },
            GridPosition { x: 0, y: 0 },
            // No Roof component implies exposed/outdoors for this test context
        )).id();

        // Run system
        corrosion_damage_system(&mut world);

        let s = world.get::<Structure>(structure).unwrap();
        assert!(s.current_hp < 100.0, "Exposed structure should take damage");
    }

    #[test]
    fn test_structure_safe_under_roof() {
        let mut world = World::new();
        world.insert_resource(CorrosiveAtmosphere { intensity: 1.0 });

        let structure = world.spawn((
            Structure { current_hp: 100.0, max_hp: 100.0 },
            GridPosition { x: 0, y: 0 },
            Roof, // Component indicating protection
        )).id();

        corrosion_damage_system(&mut world);

        let s = world.get::<Structure>(structure).unwrap();
        assert_eq!(s.current_hp, 100.0, "Roofed structure should be safe");
    }

    #[test]
    fn test_corrosion_resistance_mitigates_damage() {
        let mut world = World::new();
        world.insert_resource(CorrosiveAtmosphere { intensity: 1.0 });

        // Normal Structure
        let s1 = world.spawn((
            Structure { current_hp: 100.0, max_hp: 100.0 },
            GridPosition { x: 0, y: 0 },
        )).id();

        // Resistant Structure (50% resistance)
        let s2 = world.spawn((
            Structure { current_hp: 100.0, max_hp: 100.0 },
            GridPosition { x: 1, y: 0 },
            CorrosionResistant { factor: 0.5 },
        )).id();

        corrosion_damage_system(&mut world);

        let hp1 = world.get::<Structure>(s1).unwrap().current_hp;
        let hp2 = world.get::<Structure>(s2).unwrap().current_hp;

        let damage1 = 100.0 - hp1;
        let damage2 = 100.0 - hp2;

        assert!(damage2 < damage1, "Resistant structure should take less damage");
        // Specifically, check math if deterministic
        // assert_eq!(damage2, damage1 * 0.5);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

### 1. Define Resources and Components

In `src/layer1/atmosphere.rs`:

```rust
use bevy_ecs::prelude::*;

#[derive(Resource, Default)]
pub struct CorrosiveAtmosphere {
    pub intensity: f32, // 0.0 to 1.0 multiplier
}

#[derive(Component, Default)]
pub struct CorrosionResistant {
    pub factor: f32, // 0.0 = no resistance, 1.0 = immunity
}

// Marker for "Inside/Protected" if Roof component doesn't exist globally yet
#[derive(Component)]
pub struct ProtectedFromAtmosphere;
```

### 2. Implement `corrosion_damage_system`

```rust
use crate::layer1::structure::Structure;
use crate::layer1::map::Roof; // Ensure this exists or use ProtectedFromAtmosphere

pub fn corrosion_damage_system(world: &mut World) {
    let intensity = world.get_resource::<CorrosiveAtmosphere>()
        .map(|a| a.intensity)
        .unwrap_or(0.0);

    if intensity <= 0.0 { return; }

    let damage_per_tick = 1.0 * intensity; // Base damage

    // Query for exposed structures
    // Assuming Roof is a component. If Roof is calculated via map grid, this query changes.
    // For MVP, we'll assume entities have a 'Roof' component if they are covered.
    // OR we iterate all structures and check the map.

    let mut query = world.query::<(&mut Structure, Option<&CorrosionResistant>, Option<&Roof>)>();

    for (mut structure, resistance, roof) in query.iter_mut(world) {
        if roof.is_some() {
            continue; // Safe
        }

        let resist = resistance.map(|r| r.factor).unwrap_or(0.0);
        let actual_damage = damage_per_tick * (1.0 - resist).max(0.0);

        structure.current_hp = (structure.current_hp - actual_damage).max(0.0);
    }
}
```

## 5. REFACTOR Phase: Quality & Design

- **Map Integration**: Instead of a `Roof` component on the *structure*, the system should likely check `Map::is_indoors(position)`. This decouples the building from the roof logic (e.g., a building inside a cave is safe).
- **Materials**: `CorrosionResistant` should be applied automatically to `BuildingType::StoneWall` or `BuildingType::ReinforcedPlating`.
- **Visuals**: Spawn "Fizz" particles or green smoke on damaged entities.
- **Log**: Add message log entry when a building falls below 50% HP due to corrosion.

## 6. Acceptance Criteria

- [ ] `CorrosiveAtmosphere` resource exists and can be configured.
- [ ] `Structure` entities take damage when `CorrosiveAtmosphere.intensity > 0`.
- [ ] Entities with `Roof` (or indoor status) are immune.
- [ ] `CorrosionResistant` component reduces damage.
- [ ] Tests pass.

## 7. Technical Guidance

- Be careful with the "Roof" check. If `Roof` is a component on the building itself (e.g. "This building *has* a roof"), that's different from "This building is *under* a roof".
- For the RED/GREEN phase, use a simple `Component` check. In REFACTOR, switch to a Map lookup if available (`layer1::map::Map`).

## 8. Questions

- *Builder: Does corrosion affect units (Pops)?*
*Architect:* It affects their equipment (reducing tool/suit durability rapidly) but not their flesh directly, as long as they are wearing standard environmental gear.
  *Architect: Yes, unshielded Pops outdoors in a corrosive atmosphere take constant health damage and equipment degradation.*
*Architect: Only if they are unarmored or outdoors without protective gear. It should act as a slow health drain over time.*
    - *Architect:* Not in this spec. That would be "Health/Biocompatibility". This spec focuses on *Structure*.

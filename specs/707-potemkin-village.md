# 707 - The Potemkin Village

## 1. Overview
Faking prosperity to appease the inspector, at the cost of your own people's sanity. Players can use a "Facade" tool to cheaply disguise low-quality buildings or squalor as high-quality "Luxury" tiles. These facades fool external NPCs (like Layer 2 Inspectors or Tourists) and satisfy their requirements. However, the Pops who actually live and work behind the facades suffer a massive "Cognitive Dissonance" stress penalty from living a lie.

## 2. Dependencies
- `004-basic-building` (Building entities and quality)
- `091-the-inspector` or `408-galactic-tourism` (External NPCs measuring colony value)
- `031-pop-morale` (Stress mechanics)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::social::morale::Morale;
    use crate::layer1::needs::Stress;
    use crate::layer1::buildings::{Building, Quality};

    #[test]
    fn test_facade_improves_apparent_quality() {
        let mut world = World::new();
        let building = world.spawn((
            Building,
            Quality { value: 10.0 }, // Low quality slum
        )).id();

        // Apply a facade
        world.entity_mut(building).insert(Facade { apparent_quality: 100.0 });

        // System that evaluates building quality for an inspector
        let apparent_value = get_apparent_quality(&world, building);

        assert_eq!(apparent_value, 100.0, "Apparent quality should be overridden by the facade");
    }

    #[test]
    fn test_facade_causes_cognitive_dissonance_stress() {
        let mut world = World::new();
        let building = world.spawn((
            Building,
            Quality { value: 10.0 },
            Facade { apparent_quality: 100.0 },
        )).id();

        // Pop living/working in this building
        let pop = world.spawn((
            Pop,
            Stress { value: 0.0, ..Default::default() },
            AssignedTo { building_entity: building }, // Or just proximity
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(evaluate_facade_stress);

        schedule.run(&mut world);

        let stress = world.get::<Stress>(pop).unwrap();
        assert!(stress.value > 0.0, "Pop should accumulate stress from the facade's cognitive dissonance");
    }

    #[test]
    fn test_facade_destroyed_if_building_destroyed() {
        // Simple sanity check that the facade is a component on the building
        let mut world = World::new();
        let building = world.spawn((
            Building,
            Facade { apparent_quality: 100.0 },
        )).id();

        world.entity_mut(building).despawn_recursive();

        // Facade is gone with the building
        assert!(world.get_entity(building).is_none());
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// In src/layer1/buildings/facade.rs

use bevy::prelude::*;
use crate::layer1::needs::Stress;
use crate::layer1::buildings::Quality;
use crate::layer1::pop::{Pop, AssignedTo}; // Assuming some assignment relation

#[derive(Component, Debug, Clone)]
pub struct Facade {
    pub apparent_quality: f32,
}

// Helper to get the value used by Inspectors/Tourists
pub fn get_apparent_quality(world: &World, entity: Entity) -> f32 {
    if let Some(facade) = world.get::<Facade>(entity) {
        return facade.apparent_quality;
    }
    if let Some(quality) = world.get::<Quality>(entity) {
        return quality.value;
    }
    0.0
}

pub fn evaluate_facade_stress(
    mut pop_query: Query<(&AssignedTo, &mut Stress), With<Pop>>,
    facade_query: Query<&Facade>,
    time: Res<Time>,
) {
    for (assigned, mut stress) in pop_query.iter_mut() {
        if facade_query.get(assigned.building_entity).is_ok() {
            // Constant stress drip for living a lie
            stress.value += 10.0 * time.delta_seconds();
            // Clamp if necessary
            if stress.value > 100.0 {
                stress.value = 100.0;
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Proximity vs Assignment:** The minimal implementation assumes Pops get stressed because they are `AssignedTo` the building. It should be refactored so that *any* Pop pathfinding through or standing on a tile with a `Facade` feels the dissonance, as they see the fake walls covering the squalor.
- **Facade Cost & Maintenance:** Facades should be incredibly cheap to build but perhaps have high maintenance (they are flimsy).
- **Inspector Interaction:** Hook `get_apparent_quality` into the actual `Inspector` AI (from `091-the-inspector`) so their evaluation algorithm uses this function instead of querying `Quality` directly.

## 6. Acceptance Criteria
- [ ] `Facade` component exists and tracks `apparent_quality`.
- [ ] `get_apparent_quality` returns the facade value if present, otherwise the real quality.
- [ ] `evaluate_facade_stress` increases `Stress` for Pops interacting with the building.
- [ ] `cargo test` passes.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage for `facade.rs` is ≥85%.

## 7. Technical Guidance
- Create `src/layer1/buildings/facade.rs`.
- Ensure `evaluate_facade_stress` is added to the `Update` or `FixedUpdate` schedule in `src/layer1/systems/social.rs` (or `needs.rs`).
- You will need to locate the `Inspector` evaluation logic and update it to call the helper method or refactor it to respect the `Facade` component.

## 8. Questions
*Builder: Add questions here if spec is unclear.*

# 567 The Orbital Scrapyard

## 1. Overview
This feature introduces the "Orbital Scrapyard", an emergent Layer 2 element that directly impacts Layer 1. A massive field of orbital debris provides a steady stream of highly valuable salvage (advanced precursor tech, rare alloys) via small drops, but occasionally, larger, unpredictable chunks rain destruction upon the planetary surface. It creates a tension between the immense economic benefit of free salvage and the constant threat of localized, unpredictable kinetic strikes.

## 2. Dependencies
- Layer 2 orbit abstraction
- Layer 1 colony resource system
- Chronicle system for recording impact events and discoveries

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use scale::layer2::orbit::{OrbitalScrapyard, DebrisDrop, ImpactEvent};

    #[test]
    fn test_salvage_drop() {
        let mut world = setup_test_world();
        let scrapyard = world.spawn(OrbitalScrapyard { drop_rate: 0.1, threat_level: 0.05 }).id();

        let drop = trigger_debris_drop(&mut world, scrapyard);

        assert!(drop.is_salvage());
        let resources = get_colony_resources(&world);
        assert!(resources.rare_alloys > 0);
    }

    #[test]
    fn test_kinetic_strike_impact() {
        let mut world = setup_test_world();
        let scrapyard = world.spawn(OrbitalScrapyard { drop_rate: 0.1, threat_level: 0.05 }).id();
        let colony_pos = GridPosition { x: 10, y: 10 };

        let impact = trigger_kinetic_strike(&mut world, scrapyard, colony_pos);

        assert!(impact.is_destructive());
        assert!(world.get::<Building>(colony_pos).unwrap().is_destroyed());
        assert!(get_chronicle_events(&world).contains("orbital_impact"));
    }

    #[test]
    fn test_scrapyard_depletion() {
        let mut world = setup_test_world();
        let scrapyard = world.spawn(OrbitalScrapyard { drop_rate: 1.0, threat_level: 0.0, remaining_mass: 100 }).id();

        for _ in 0..100 {
            trigger_debris_drop(&mut world, scrapyard);
        }

        let final_state = world.get::<OrbitalScrapyard>(scrapyard).unwrap();
        assert_eq!(final_state.remaining_mass, 0);
        assert!(trigger_debris_drop(&mut world, scrapyard).is_empty());
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct OrbitalScrapyard {
    pub drop_rate: f32,
    pub threat_level: f32,
    pub remaining_mass: u32,
}

pub enum DebrisDrop {
    Salvage(u32),
    KineticStrike(GridPosition),
    Empty,
}

pub fn trigger_debris_drop(world: &mut World, entity: Entity) -> DebrisDrop {
    let mut scrapyard = world.get_mut::<OrbitalScrapyard>(entity).unwrap();
    if scrapyard.remaining_mass > 0 {
        scrapyard.remaining_mass -= 1;
        let mut resources = world.get_resource_mut::<ColonyResources>().unwrap();
        resources.rare_alloys += 10;
        DebrisDrop::Salvage(10)
    } else {
        DebrisDrop::Empty
    }
}

pub fn trigger_kinetic_strike(world: &mut World, entity: Entity, target: GridPosition) -> DebrisDrop {
    if let Some(mut building) = world.get_mut::<Building>(target) {
        building.health = 0; // Destroyed
    }
    let mut chronicle = world.get_resource_mut::<Chronicle>().unwrap();
    chronicle.add_event("orbital_impact".to_string());
    DebrisDrop::KineticStrike(target)
}
```

## 5. REFACTOR Phase: Quality & Design
- **Refactoring Opportunities:** Implement a probabilistic system in `trigger_debris_drop` to differentiate between harmless salvage and destructive kinetic strikes based on `threat_level`.
- **Code Smells:** Avoid direct health modification (`health = 0`). Instead, emit a `DamageEvent` that existing systems can handle gracefully.
- **Performance:** Ensure debris checks are processed in batches or at specific intervals, not every tick, to reduce overhead.
- **API Improvements:** Create a robust event system (`DebrisEvent`) for cross-layer communication (Layer 2 generating the event, Layer 1 processing it).

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] The Scrapyard generates both positive salvage drops and negative kinetic impacts.

## 7. Technical Guidance
- **Code Structure:** Add the `OrbitalScrapyard` component to `src/layer2/orbit.rs` and the processing system to `src/layer1/events.rs`.
- **Integration Points:** Link kinetic strikes to the existing building destruction and pop injury logic.
- **Gotchas:** Ensure the `remaining_mass` correctly halts drops when the scrapyard is fully depleted.

## 8. Questions
*Builder: add questions here if spec is unclear. Architect will address.*

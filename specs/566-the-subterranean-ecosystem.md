# 566 The Subterranean Ecosystem

## 1. Overview
This feature introduces a subterranean ecosystem on Layer 1. As colonies expand downwards, they discover interconnected, independent biomes that offer immense resources (rare fungi, geothermal energy, ancient aquifers) but also hide unpredictable dangers (toxic gas pockets, aggressive subterranean megafauna, structural collapses). It creates a "push your luck" mining dynamic where deeper excavations yield higher rewards but significantly increase the risk of colony-ending subterranean events.

## 2. Dependencies
- Layer 1 tile grid (for Z-axis depth tracking)
- Layer 1 mining and gathering systems
- `Chronicle` system (for recording subterranean discoveries and disasters)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use scale::layer1::ecosystem::{SubterraneanBiome, BiomeThreat};

    #[test]
    fn test_discover_subterranean_biome() {
        let mut world = setup_test_world();
        let tile = GridPosition { x: 5, y: 5, z: -10 };

        let discovery = discover_biome(&mut world, tile);

        assert!(discovery.is_some());
        assert_eq!(world.get::<SubterraneanBiome>(tile).unwrap().discovered, true);
    }

    #[test]
    fn test_biome_resource_yield() {
        let mut world = setup_test_world();
        let biome_tile = GridPosition { x: 5, y: 5, z: -15 };
        spawn_biome(&mut world, biome_tile, SubterraneanBiomeType::FungalCavern);

        let initial_resources = get_colony_resources(&world);
        harvest_biome(&mut world, biome_tile);

        assert!(get_colony_resources(&world).rare_fungi > initial_resources.rare_fungi);
    }

    #[test]
    fn test_biome_threat_trigger() {
        let mut world = setup_test_world();
        let biome_tile = GridPosition { x: 5, y: 5, z: -20 };
        spawn_biome_with_threat(&mut world, biome_tile, BiomeThreat::ToxicGas);

        trigger_threat(&mut world, biome_tile);

        assert!(world.contains_resource::<ToxicGasCloud>());
        assert!(get_chronicle_events(&world).contains("toxic_gas_release"));
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// The SIMPLEST code that makes tests pass
use bevy::prelude::*;

#[derive(Component)]
pub struct SubterraneanBiome {
    pub discovered: bool,
    pub biome_type: SubterraneanBiomeType,
}

#[derive(Clone, Copy, PartialEq)]
pub enum SubterraneanBiomeType {
    FungalCavern,
    GeothermalVent,
}

pub enum BiomeThreat {
    ToxicGas,
}

pub fn discover_biome(world: &mut World, pos: GridPosition) -> Option<SubterraneanBiomeType> {
    if let Some(mut biome) = world.get_mut::<SubterraneanBiome>(pos) {
        biome.discovered = true;
        Some(biome.biome_type)
    } else {
        None
    }
}

pub fn harvest_biome(world: &mut World, pos: GridPosition) {
    if let Some(biome) = world.get::<SubterraneanBiome>(pos) {
        if biome.biome_type == SubterraneanBiomeType::FungalCavern {
            let mut resources = world.get_resource_mut::<ColonyResources>().unwrap();
            resources.rare_fungi += 10;
        }
    }
}

pub fn trigger_threat(world: &mut World, pos: GridPosition) {
    world.insert_resource(ToxicGasCloud { origin: pos });
    let mut chronicle = world.get_resource_mut::<Chronicle>().unwrap();
    chronicle.add_event("toxic_gas_release".to_string());
}
```

## 5. REFACTOR Phase: Quality & Design
- **Refactoring Opportunities:** Implement a weighted random generation system for subterranean biomes based on planetary traits.
- **Code Smells:** Avoid hardcoding resource yields; integrate with a broader loot table or configuration file.
- **Performance:** Ensure biome discovery checks only occur during active mining or expansion actions, not every tick.
- **API Improvements:** Create a unified `SubterraneanEvent` enum to handle both discoveries and threats cleanly for the Chronicle.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Digging deep triggers biome discovery and potential threats.

## 7. Technical Guidance
- **Code Structure:** Place the logic in `src/layer1/ecosystem/subterranean.rs`.
- **Integration Points:** Connect `trigger_threat` to the existing environmental hazard systems (e.g., Gas or Heat systems).
- **Gotchas:** Make sure biomes are only accessible when directly adjacent to a mined tile to prevent "telepathic" discovery.

## 8. Questions
*Builder: add questions here if spec is unclear. Architect will address.*

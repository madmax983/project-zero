# 1154: The Blind Terraformer

## 1. Overview
The Blind Terraformer introduces an ancient, automated terraforming seed-ship that players can capture and direct to a new colony. However, due to severely degraded sensors, the ship misreads local atmospheric data and wildly overcompensates, creating bizarre, hybrid biomes instead of a perfect Earth-like environment. This offers unique resources but requires completely redesigning the colony's infrastructure.

## 2. Dependencies
- Cross-layer support
- `Environment` / `Biome` definitions
- Terrain modifications
- Atmospheric or weather state
- Events and resource systems

## 3. RED Phase: Tests First

```rust
// src/environment/blind_terraformer_tests.rs
#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use super::*;

    #[test]
    fn test_blind_terraformer_changes_biome() {
        let mut app = App::new();
        app.add_plugins(EnvironmentPlugin);
        app.add_systems(Update, apply_blind_terraformer_system);

        // Spawn a planet with a barren biome
        let planet = app.world_mut().spawn((
            Planet { name: "Barren Rock".to_string() },
            Biome { biome_type: BiomeType::Barren },
        )).id();

        // Spawn the blind terraformer targeting the planet
        app.world_mut().spawn(BlindTerraformer {
            target: planet,
            active: true,
        });

        app.update();

        // The biome should change to something weird and unpredictable
        let new_biome = app.world().get::<Biome>(planet).unwrap();
        assert_ne!(new_biome.biome_type, BiomeType::Barren, "The terraformer should alter the biome");
        assert!(
            matches!(new_biome.biome_type, BiomeType::Hybrid(_)),
            "The resulting biome should be a bizarre hybrid type"
        );
    }

    #[test]
    fn test_blind_terraformer_causes_infrastructure_damage() {
        let mut app = App::new();
        app.add_plugins(EnvironmentPlugin);
        app.add_systems(Update, apply_blind_terraformer_system);

        let planet = app.world_mut().spawn((
            Planet { name: "Barren Rock".to_string() },
            Biome { biome_type: BiomeType::Barren },
        )).id();

        // Add some existing infrastructure
        let solar_array = app.world_mut().spawn((
            Infrastructure { hp: 100 },
            Location { planet_id: planet },
        )).id();

        app.world_mut().spawn(BlindTerraformer {
            target: planet,
            active: true,
        });

        app.update();

        let damaged_infrastructure = app.world().get::<Infrastructure>(solar_array).unwrap();
        assert!(damaged_infrastructure.hp < 100, "Existing infrastructure should take damage due to extreme environmental shifts");
    }

    #[test]
    fn test_blind_terraformer_spawns_exotic_resources() {
        let mut app = App::new();
        app.add_plugins(EnvironmentPlugin);
        app.add_systems(Update, apply_blind_terraformer_system);

        let planet = app.world_mut().spawn((
            Planet { name: "Barren Rock".to_string() },
            Biome { biome_type: BiomeType::Barren },
        )).id();

        app.world_mut().spawn(BlindTerraformer {
            target: planet,
            active: true,
        });

        app.update();

        // Check if exotic resources are now present
        let mut query = app.world_mut().query::<&ExoticResourceDeposit>();
        let deposits_count = query.iter(app.world()).count();
        assert!(deposits_count > 0, "The terraformed hybrid biome should spawn exotic resources");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// The SIMPLEST code that makes tests pass
// No optimization, no extras
// Just enough to turn RED → GREEN
```

## 5. REFACTOR Phase: Quality & Design
- **Randomization:** The `BiomeType::Hybrid` selection should rely on a pseudo-random number generator (e.g., using `rand` crate or Bevy's deterministic randomness tools) rather than hardcoded logic.
- **Gradual Terraforming:** Instead of instantaneous biome changes, consider a progression over time (e.g., using a timer or progression float).
- **Damage Logic:** Infrastructure damage should be based on its type; e.g., solar arrays are crushed, but underground bunkers might survive.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for `src/environment/blind_terraformer.rs`.
- [ ] Activating the Terraformer alters the planet's biome to a `Hybrid` type.
- [ ] Activating the Terraformer applies damage to existing surface infrastructure.
- [ ] New `Hybrid` biomes yield `ExoticResourceDeposit` entities.

## 7. Technical Guidance
- Integrate with `src/layer1/environment.rs` and the broader biome/terrain generation logic.
- Consider adding new `BiomeType` variants (like `Hybrid(HybridBiomeType)`) if they do not exist.
- Damage should be dispatched via an event system (e.g., `EnvironmentalDamageEvent`) rather than direct modification, to allow other systems (like shields or repairs) to react.

## 8. Questions
*Builder: add questions here if spec is unclear.*

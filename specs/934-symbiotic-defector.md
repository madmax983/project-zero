# 934 - The Symbiotic Defector

## 1. Overview

**Layer:** Cross-layer (3 -> 1)
**Fantasy:** Accepting an alien defector who brings incredible tech, but their biology begins to terraform your colony.
**Mechanic:** A high-level alien leader defects to your colony, granting massive research points and diplomatic intel on Layer 3. However, they naturally emit spores or radiation that slowly changes the Layer 1 biome around them to match their homeworld. Native crops die, and new, alien flora sprouts, which your human pops can't easily digest.
**Emergence:** You welcome the defector and assign them to the central research lab. A year later, the lab is overgrown with toxic crystalline vines, the adjacent farms have withered, and the defector is perfectly happy while your human scientists are suffocating in the new atmosphere.
**Tension:** Massive technological leaps and strategic Layer 3 advantages vs. the localized, uncontrollable terraforming of your own capital.

## 2. Dependencies

- Layer 1 Biome / Terrain system (for terraforming)
- Layer 1 Needs system (for human pop reactions to alien environments)
- Layer 3 Diplomatic / Research event triggers

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[derive(Component)]
    struct SymbioticDefector { terraform_radius: f32, active: bool }

    #[derive(Component)]
    struct TerrainTile { is_alien: bool, can_grow_human_crops: bool }

    #[derive(Component)]
    struct ResearchLab { research_points_generated: u32 }

    #[test]
    fn test_defector_boosts_research() {
        let mut app = App::new();
        app.add_systems(Update, defector_research_boost_system);

        let lab = app.world_mut().spawn((
            ResearchLab { research_points_generated: 10 },
            SymbioticDefector { terraform_radius: 5.0, active: true },
        )).id();

        app.update();

        // The defector should significantly increase research output
        let lab_data = app.world().get::<ResearchLab>(lab).unwrap();
        assert!(lab_data.research_points_generated > 10);
    }

    #[test]
    fn test_defector_terraforms_surroundings() {
        let mut app = App::new();
        app.add_systems(Update, defector_terraforming_system);

        // Spawn a defector
        let defector = app.world_mut().spawn((
            Transform::from_xyz(0.0, 0.0, 0.0),
            SymbioticDefector { terraform_radius: 10.0, active: true },
        )).id();

        // Spawn nearby terrain
        let tile = app.world_mut().spawn((
            Transform::from_xyz(5.0, 0.0, 0.0),
            TerrainTile { is_alien: false, can_grow_human_crops: true },
        )).id();

        app.update();

        // The nearby terrain should be converted to an alien biome
        let terrain = app.world().get::<TerrainTile>(tile).unwrap();
        assert!(terrain.is_alien);
        assert!(!terrain.can_grow_human_crops);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

// pub fn defector_research_boost_system(...) { ... }
// pub fn defector_terraforming_system(...) { ... }
```

## 5. REFACTOR Phase: Quality & Design

- Terraforming should spread gradually over time using a flood-fill or diffusion algorithm from the defector's current location.
- Alien biome should apply negative status effects (e.g., suffocation, sickness) to human pops who spend too much time in it.
- Ensure research boosts are tied to the defector being actively assigned to a relevant job/location.

## 6. Acceptance Criteria

- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] The defector provides tangible research/intel benefits but actively corrupts nearby terrain into an alien state that hurts human agriculture.

## 7. Technical Guidance

- Utilize the existing `TerrainGrid` or `AtmosphereGrid` to represent the spread of alien spores/radiation.
- The `SymbioticDefector` component might need to be added to an existing `Pop` entity or treated as a unique VIP entity depending on how leaders are handled.

## 8. Questions
*Builder: add questions here if spec is unclear.*

# 557 - The Subterranean Ecosystem

## 1. Overview
Excavating deep enough reveals cavernous spaces filled with bioluminescent flora and blind, subterranean fauna. These biomes provide unique, high-yield resources (e.g., luminescent fungi for medicine). However, disturbing the ecosystem can trigger massive cave-ins or awaken highly adapted predators.

**Fantasy:** You carefully mine a rich vein of luminescent fungi to cure an illness. The removal collapses a critical support structure, triggering a cave-in that breaches an underground lake, flooding operations and introducing aggressive aquatic predators to lower levels.

**Layer:** 1 (Colony)

## 2. Dependencies
- `018-mining-resources.md` (Mining)
- `153-geological-instability.md` (Cave-ins)
- `131-bioluminescent-flora.md` (Flora/Lighting)
- `164-modular-fauna.md` (Fauna/Predators)
- `412-fluid-simulation.md` (Flooding)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    // 1. Excavation Triggers
    #[test]
    fn test_deep_mining_spawns_subterranean_biome() {
        // Arrange: Tile deep underground
        // Act: Mine tile
        // Assert: Surrounding tiles generate 'SubterraneanBiome' component and resources (LuminescentFungi)
    }

    // 2. Harvesting Hazards
    #[test]
    fn test_harvesting_subterranean_resources_increases_instability() {
        // Arrange: Subterranean tile with Fungi
        // Act: Harvest Fungi
        // Assert: Tile instability value increases, occasionally triggering cave-in event
    }

    // 3. Predator Awakening
    #[test]
    fn test_mining_noise_awakens_predators() {
        // Arrange: Noise generated in Subterranean Biome
        // Act: Simulate noise propagation
        // Assert: Nearby dormant predator entities become active and hostile
    }

    // 4. Subterranean Flooding
    #[test]
    fn test_cave_in_breaches_underground_lake() {
        // Arrange: Cave-in next to an 'UndergroundLake' tile
        // Act: Trigger cave-in
        // Assert: Fluid source spawned, surrounding tiles flooded
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct SubterraneanBiome;

#[derive(Component)]
pub struct LuminescentFungi {
    pub instability_value: f32,
}

#[derive(Component)]
pub struct DormantPredator;

#[derive(Component)]
pub struct UndergroundLake;

// System to generate biome when mining deep
pub fn generate_subterranean_biome_on_mine(
    mut mining_events: EventReader<MineTileEvent>,
    mut commands: Commands,
) {
    for event in mining_events.iter() {
        if event.depth > 50 {
            // Spawn SubterraneanBiome components and LuminescentFungi
            commands.spawn((SubterraneanBiome, LuminescentFungi { instability_value: 0.1 }));
        }
    }
}

// System to handle harvesting and instability
pub fn handle_harvesting_instability(
    mut harvest_events: EventReader<HarvestEvent>,
    mut instability_query: Query<&mut Instability, With<SubterraneanBiome>>,
) {
    for event in harvest_events.iter() {
        // Increase instability
        // Check if cave-in should occur
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Lighting:** Subterranean biomes should emit their own weak light (bioluminescence) saving the player from building lights initially.
- **Pathfinding:** Predators should only navigate within the subterranean biome or flooded areas.
- **Loot Table:** Ensure Luminescent Fungi drops are high-tier to justify the risk.

## 6. Acceptance Criteria
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for `subterranean_ecosystem.rs`.
- [ ] Mining deep generates the subterranean biome.
- [ ] Harvesting resources from this biome causes instability.
- [ ] Instability can trigger cave-ins or flood events.
- [ ] Noise or cave-ins awaken predators.

## 7. Technical Guidance
- The instability logic should hook into the existing `153-geological-instability.md` systems rather than rewriting it.
- Use `AcousticZones` (`278`) for the noise propagation to awaken predators.
- Ensure the fluid system can handle large, sudden introductions of water from lake breaches.

## 8. Questions
*Builder: add questions here if spec is unclear.*

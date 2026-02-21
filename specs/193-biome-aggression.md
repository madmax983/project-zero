# 193: Biome Aggression

## Overview

The planet is not a passive backdrop; it reacts to colonization. As the colony expands and destroys nature, the local biome becomes hostile.

This spec introduces a **Biome Aggression** system that tracks the "anger" of the local ecosystem. High aggression accelerates **Ecological Succession** (Spec 161) and empowers **Antagonistic Flora** (Spec 092) to reclaim developed land more aggressively.

This creates a dynamic "Man vs. Nature" loop:
1. Colony builds → Aggression rises.
2. Aggression rises → Plants grow faster and attack buildings.
3. Colony clears plants → Aggression rises further (short term) but clears space.

## Dependencies

- `161` — Ecological Succession (Target for modification)
- `092` — Antagonistic Flora (Target for modification)
- `004` — Building System (Trigger for aggression)

## RED Phase: Tests First

Write these tests in `src/layer1/biome_aggression_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::biome_aggression::{BiomeAggression, update_biome_aggression_system, apply_aggression_effects_system};
    use crate::layer1::ecology::EcologyConfig;
    use crate::layer1::flora::Flora;

    #[test]
    fn test_aggression_initial_state() {
        let aggression = BiomeAggression::default();
        assert_eq!(aggression.current_level, 0.0);
        assert_eq!(aggression.decay_rate, 0.001);
    }

    #[test]
    fn test_aggression_decays_over_time() {
        let mut world = World::new();
        world.insert_resource(BiomeAggression {
            current_level: 0.5,
            decay_rate: 0.1,
            ..Default::default()
        });

        let mut schedule = Schedule::default();
        schedule.add_systems(update_biome_aggression_system);
        schedule.run(&mut world);

        let aggression = world.resource::<BiomeAggression>();
        assert!(aggression.current_level < 0.5);
        assert!(aggression.current_level >= 0.0);
    }

    #[test]
    fn test_aggression_increases_on_events() {
        // This test simulates an event writer for building construction
        // For MVP, we might just increment it directly in the test to verify the logic
        // but ideally we hook into an event system.
        // Let's assume a manual increment for this unit test.
        let mut aggression = BiomeAggression::default();
        aggression.add_aggression(0.1);
        assert_eq!(aggression.current_level, 0.1);

        aggression.add_aggression(1.0); // Should cap at 1.0
        assert_eq!(aggression.current_level, 1.0);
    }

    #[test]
    fn test_ecology_scales_with_aggression() {
        let mut world = World::new();
        world.insert_resource(BiomeAggression { current_level: 1.0, ..Default::default() });
        world.insert_resource(EcologyConfig {
            growth_rate: 0.05,
            ..Default::default()
        });

        let mut schedule = Schedule::default();
        schedule.add_systems(apply_aggression_effects_system);
        schedule.run(&mut world);

        let config = world.resource::<EcologyConfig>();
        // At max aggression, growth rate should be significantly higher
        assert!(config.growth_rate > 0.05);
    }

    #[test]
    fn test_flora_scales_with_aggression() {
        let mut world = World::new();
        world.insert_resource(BiomeAggression { current_level: 1.0, ..Default::default() });

        // Spawn a Flora entity
        let flora = world.spawn(Flora {
            spread_chance: 0.1,
            ..Default::default()
        }).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(apply_aggression_effects_system);
        schedule.run(&mut world);

        let updated_flora = world.entity(flora).get::<Flora>().unwrap();
        // At max aggression, spread chance should be higher
        assert!(updated_flora.spread_chance > 0.1);
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Define `BiomeAggression` Resource

In `src/layer1/biome_aggression.rs`:

```rust
use bevy_ecs::prelude::*;
use crate::layer1::ecology::EcologyConfig;
use crate::layer1::flora::Flora;

#[derive(Resource, Debug, Clone, Copy)]
pub struct BiomeAggression {
    pub current_level: f32, // 0.0 to 1.0
    pub decay_rate: f32,    // per tick
    pub growth_modifier: f32, // Multiplier for Ecology
    pub spread_modifier: f32, // Multiplier for Flora
}

impl Default for BiomeAggression {
    fn default() -> Self {
        Self {
            current_level: 0.0,
            decay_rate: 0.0005,
            growth_modifier: 2.0, // Double growth at max aggression
            spread_modifier: 3.0, // Triple spread at max aggression
        }
    }
}

impl BiomeAggression {
    pub fn add_aggression(&mut self, amount: f32) {
        self.current_level = (self.current_level + amount).clamp(0.0, 1.0);
    }
}
```

### 2. Implement Systems

```rust
pub fn update_biome_aggression_system(mut aggression: ResMut<BiomeAggression>) {
    // Natural decay
    aggression.current_level = (aggression.current_level - aggression.decay_rate).max(0.0);
}

pub fn apply_aggression_effects_system(
    aggression: Res<BiomeAggression>,
    mut ecology: ResMut<EcologyConfig>,
    mut flora_query: Query<&mut Flora>,
) {
    let level = aggression.current_level;

    // Base values (hardcoded for MVP, ideally stored in a BaseConfig)
    let base_growth = 0.05;
    let base_spread = 0.1;

    // Apply modifiers: base + (base * modifier * level)
    ecology.growth_rate = base_growth + (base_growth * aggression.growth_modifier * level);

    for mut flora in flora_query.iter_mut() {
        flora.spread_chance = base_spread + (base_spread * aggression.spread_modifier * level);
    }
}
```

### 3. Integration Hooks (Conceptual)

- **Building Construction**: Trigger `aggression.add_aggression(0.05)` when a building completes.
- **Flora Destruction**: Trigger `aggression.add_aggression(0.01)` when flora is cleared.
- **Pollution**: Trigger `aggression.add_aggression(pollution_level * 0.001)` per tick.

## REFACTOR Phase: Quality & Design

- **Spatial Aggression**: Instead of a global value, use a `AggressionGrid` (heatmap) so aggression is localized to industrial zones.
- **Aggression Tiers**: Define thresholds (e.g., >0.8 triggers "Titan Spawn" or "Mass Bloom").
- **UI**: Add a "Biome Threat" meter to the UI.
- **Events**: Use `EventReader<BuildingCompletedEvent>` to decouple the trigger logic.

## Acceptance Criteria

- [ ] `BiomeAggression` resource implemented.
- [ ] Aggression decays over time.
- [ ] High aggression increases `EcologyConfig` growth rate.
- [ ] High aggression increases `Flora` spread chance.
- [ ] Tests pass.

## Technical Guidance

- Ensure `apply_aggression_effects_system` runs *before* `process_ecological_succession` and `flora_spread_system` to ensure the modifiers are active for the current tick.
- Be careful with `clamp(0.0, 1.0)` to avoid runaway values.
- `EcologyConfig` might need a `reset()` method to restore base values if we change them every tick, OR simply overwrite them every tick based on `base + modifier`. The latter is stateless and safer.

## Questions

*Builder: add questions here if spec is unclear.*

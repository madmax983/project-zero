# 040: Clothing and Temperature

## Overview

Introduces a survival loop centered around Winter. Pops without adequate clothing will suffer Hypothermia damage during cold seasons. To mitigate this, players must establish a textile industry: growing Fiber (Plantations), weaving Cloth (Weavers), and tailoring Clothing (Tailors).

This adds:
- **New Resources**: Fiber, Cloth, Clothing.
- **New Buildings**: Plantation, Weaver, Tailor.
- **New Need**: Warmth (abstracted via Clothing pool).
- **New Threat**: Hypothermia (Health damage).

## Dependencies

- `027` — Seasonal Rhythms (provides `SeasonState`).
- `034` — Pop Health (provides `Health` and `take_damage`).
- `023` — Refining Industry (provides `RefiningProgress` pattern).
- `030` — Tool Economy (provides "Global Pool" distribution pattern).

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/layer1/clothing_tests.rs (New file or integrated into existing modules)

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::resources::ColonyResources;
    use crate::layer1::seasons::{Season, SeasonState};
    use crate::layer1::health::Health;
    use crate::layer1::pop::Pop;
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::refining::{process_refining_system, get_refining_recipe, RefiningProgress};
    use crate::layer1::GridPosition;

    // 1. Resource Tests
    #[test]
    fn test_colony_resources_clothing_fields() {
        let res = ColonyResources::default();
        // New fields should exist
        assert_eq!(res.fiber, 0.0);
        assert_eq!(res.cloth, 0.0);
        assert_eq!(res.clothing, 0.0);

        // Defaults
        assert!(res.max_fiber > 0.0);
        assert!(res.max_cloth > 0.0);
        assert!(res.max_clothing > 0.0);
    }

    // 2. Refining Recipe Tests
    #[test]
    fn test_weaver_recipe() {
        let res = ColonyResources {
            fiber: 5.0,
            cloth: 0.0,
            max_cloth: 10.0,
            ..Default::default()
        };

        let (can_afford, input, output) = get_refining_recipe(BuildingType::Weaver, &res);

        assert!(can_afford);
        assert_eq!(input.fiber, 1.0);
        assert_eq!(output.cloth, 1.0);
    }

    #[test]
    fn test_tailor_recipe() {
        let res = ColonyResources {
            cloth: 5.0,
            clothing: 0.0,
            max_clothing: 10.0,
            ..Default::default()
        };

        let (can_afford, input, output) = get_refining_recipe(BuildingType::Tailor, &res);

        assert!(can_afford);
        assert_eq!(input.cloth, 1.0);
        assert_eq!(output.clothing, 1.0);
    }

    // 3. Hypothermia System Tests
    #[test]
    fn test_hypothermia_damage_in_winter_no_clothes() {
        let mut world = World::new();
        world.insert_resource(SeasonState { current_season: Season::Winter });
        world.insert_resource(ColonyResources { clothing: 0.0, ..Default::default() });

        // Spawn Pop
        let pop = world.spawn((
            Pop,
            Health { current: 100.0, max: 100.0 }
        )).id();

        // Run system multiple times to ensure probabilistic damage hits eventually
        // Or mock the RNG if possible. For now, assume high probability or deterministic test loop.
        // Let's assume the system deals 1 damage if hit.

        // We can force damage by mocking the probability or running enough ticks.
        // For TDD, let's just assert the system runs and *can* damage.

        super::hypothermia_system(&mut world);

        // This test is tricky without deterministic RNG.
        // Better approach: Test the *logic* inside the system, or expose probability.
        // For this spec, we will assume deterministic "first N pops get clothes".
    }

    #[test]
    fn test_clothing_prevents_hypothermia() {
        let mut world = World::new();
        world.insert_resource(SeasonState { current_season: Season::Winter });
        // 1 Pop, 1 Clothing
        world.insert_resource(ColonyResources { clothing: 1.0, ..Default::default() });

        let pop = world.spawn((
            Pop,
            Health { current: 100.0, max: 100.0 }
        )).id();

        super::hypothermia_system(&mut world);

        let health = world.get::<Health>(pop).unwrap();
        assert_eq!(health.current, 100.0);
    }

    #[test]
    fn test_clothing_degradation() {
        let mut world = World::new();
        world.insert_resource(ColonyResources { clothing: 10.0, ..Default::default() });
        // Spawn pops to use the clothing
        for _ in 0..10 { world.spawn(Pop); }

        // Run wear system
        super::clothing_wear_system(&mut world);

        let res = world.resource::<ColonyResources>();
        // Should be <= 10.0. Exact amount depends on RNG/Rate.
        // We verify it *can* decrease.
        // If probability is low, we might loop.
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Update `ColonyResources` (`src/layer1/resources.rs`)

Add fields for Fiber, Cloth, and Clothing.

```rust
pub struct ColonyResources {
    // ... existing fields ...
    pub fiber: f32,
    pub cloth: f32,
    pub clothing: f32,

    pub max_fiber: f32,
    pub max_cloth: f32,
    pub max_clothing: f32,
}

// Update Default impl
// Update add_fiber, add_cloth, add_clothing methods
```

### 2. Update `BuildingType` (`src/layer1/building.rs`)

Add `Plantation`, `Weaver`, and `Tailor`.

```rust
pub enum BuildingType {
    // ...
    Plantation,
    Weaver,
    Tailor,
}

// Update `label()`, `construction_cost()`, `char()`
// Plantation: '♣' (Green) - Same as Farm but maybe different shade?
// Weaver: 'W' (White)
// Tailor: 'T' (Blue)
```

### 3. Update `Refining` (`src/layer1/refining.rs`)

Update `get_refining_recipe` to handle new buildings.

```rust
// Inside get_refining_recipe match:
BuildingType::Weaver => (
    res.fiber >= 1.0 && res.cloth < res.max_cloth,
    ColonyResources { fiber: 1.0, ..Default::default() },
    ColonyResources { cloth: 1.0, ..Default::default() },
),
BuildingType::Tailor => (
    res.cloth >= 1.0 && res.clothing < res.max_clothing,
    ColonyResources { cloth: 1.0, ..Default::default() },
    ColonyResources { clothing: 1.0, ..Default::default() },
),
```

### 4. Update Farming (`src/layer1/farm.rs`)

Modify `produce_food_system` to handle different outputs.

**Option:** Rename `produce_food_system` to `farming_system`.
Inside the loop, check `BuildingType` of the `Building` component attached to the same entity.

```rust
pub fn farming_system(
    // Query now needs Building to distinguish Farm vs Plantation
    farm_query: Query<(&Farm, &Building)>,
    pop_query: Query<&Pop>,
    season: Option<Res<SeasonState>>,
    mut resources: ResMut<ColonyResources>,
) {
    let modifier = season.map_or(1.0, |s| s.current_season.food_modifier());

    for (farm, building) in &farm_query {
        let worker_count = farm.workers.iter().filter(|e| pop_query.get(**e).is_ok()).count() as f32;
        let production = worker_count * FOOD_PER_WORKER_PER_TICK * modifier;

        if production > 0.0 {
            match building.building_type {
                BuildingType::Farm => resources.add_food(production),
                BuildingType::Plantation => resources.add_fiber(production), // Fiber uses same rate for now
                _ => {}
            }
        }
    }
}
```

### 5. Hypothermia & Wear Systems (`src/layer1/clothing.rs`)

```rust
// src/layer1/clothing.rs

pub fn hypothermia_system(
    mut pop_query: Query<&mut Health, With<Pop>>,
    resources: Res<ColonyResources>,
    season: Option<Res<SeasonState>>,
) {
    // Only applies in Winter
    if !matches!(season.map(|s| s.current_season), Some(Season::Winter)) {
        return;
    }

    let clothing_available = resources.clothing;
    let pop_count = pop_query.iter().count() as f32;

    if pop_count == 0.0 { return; }

    // Calculate shortage ratio (0.0 = full clothes, 1.0 = no clothes)
    let shortage = (1.0 - (clothing_available / pop_count)).clamp(0.0, 1.0);

    if shortage <= 0.0 { return; }

    let mut rng = rand::thread_rng();

    for mut health in &mut pop_query {
        // Probabilistic damage based on shortage
        if rng.gen_bool(shortage as f64) {
            // Deal damage!
            health.take_damage(1.0);
        }
    }
}

pub fn clothing_wear_system(
    pop_query: Query<&Pop>,
    mut resources: ResMut<ColonyResources>,
) {
    let pop_count = pop_query.iter().count() as f32;
    let used_clothing = resources.clothing.min(pop_count);

    if used_clothing <= 0.0 { return; }

    // Simple decay: Fixed % chance per tick per used item
    // OR simplified: decay = used_clothing * RATE
    let decay_rate = 0.001;
    let amount_lost = used_clothing * decay_rate;

    if amount_lost > 0.0 {
        // Direct subtraction, clamp to 0
        resources.clothing = (resources.clothing - amount_lost).max(0.0);
    }
}
```

### 6. Register Systems

In `src/main.rs` (and `layer1/mod.rs`):
- Register `hypothermia_system` (Update schedule).
- Register `clothing_wear_system` (Update schedule).
- Update `farming_system` registration.

## REFACTOR Phase: Quality & Design

- **Probability**: `hypothermia_system` uses RNG for individual damage. This works well for large populations but might be spiky for small ones. Consider accumulating "Cold" debuff instead.
- **Feedback**: Add `Thought` generation ("So cold...", "Nice and warm") in `hypothermia_system`.
- **UI**: Ensure Status Bar shows new resources.
- **Stockpiles**: Ensure Stockpiles increase max capacity for Fiber/Cloth/Clothing (`022` might need update if it hardcoded resource types, but `ColonyResources` handles max caps).

## Acceptance Criteria

- [ ] `ColonyResources` tracks Fiber, Cloth, Clothing.
- [ ] `Plantation` building produces Fiber.
- [ ] `Weaver` produces Cloth from Fiber.
- [ ] `Tailor` produces Clothing from Cloth.
- [ ] Pops take damage in Winter if `clothing < pops`.
- [ ] Clothing degrades over time based on usage (pop count).
- [ ] `cargo test` passes.
- [ ] `cargo clippy -- -D warnings` passes.

## Technical Guidance

- Use `ColonyResources::add_X` methods to respect max caps.
- Ensure `farming_system` handles `modifier` correctly for Fiber (maybe Winter shouldn't penalize Fiber as much? Or should it? Stick to same modifier for simplicity).
- Remember to update `src/ui/status.rs` to display new resources if desired, or leave for separate UI task.

## Questions

- Should Fiber rot like Food? (Deferred to Entropy spec).
- Should Clothing have different tiers? (No, keep simple).

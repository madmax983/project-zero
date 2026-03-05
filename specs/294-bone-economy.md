# Specification: The Bone Economy (Layer 1)

## 1. Overview
The dead are your most precious resource. Advanced construction materials and potentially other high-tier recipes require "Calcium-Alloys" derived from Pop corpses or mega-fauna bones. "Grave-Robber" jobs (or a specific building/action) extract this resource, but engaging in this desecration generates massive Unrest or Stress.

## 2. Dependencies
- `018` Mining and Resources (for resource definitions and yields).
- `009` Job Assignment System (for the Grave-Robber or extraction job).
- `050` Civil Unrest (for Unrest generation).
- `057` Funeral Rites (for interaction with Corpses/Graves).

## 3. RED Phase: Tests First

```rust
// src/layer1/bone_economy_tests.rs

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;
    use crate::layer1::inventory::Inventory;
    use crate::layer1::needs::StressTracker;
    use crate::layer1::pop::Pop;
    use crate::layer1::resources::ResourceType;

    fn setup_world() -> World {
        let mut world = World::new();
        // Insert necessary resources (e.g. UnrestTracker)
        world.insert_resource(UnrestTracker { level: 0.0 });
        world
    }

    #[test]
    fn test_extract_calcium_from_corpse() {
        let mut world = setup_world();

        let corpse_entity = world.spawn(Corpse { bone_yield: 10.0 }).id();
        let extractor_entity = world.spawn((
            Pop,
            Inventory::default(),
            StressTracker::default(),
            BoneExtractor,
            CurrentAction::ExtractingBone { target: corpse_entity },
        )).id();

        world.run_system_once(extract_bone_system);

        // Assert corpse is gone or depleted
        assert!(world.get_entity(corpse_entity).is_err() || world.get::<Corpse>(corpse_entity).unwrap().bone_yield == 0.0);

        // Assert inventory gained CalciumAlloy
        let inventory = world.get::<Inventory>(extractor_entity).unwrap();
        assert_eq!(inventory.get_amount(ResourceType::CalciumAlloy), 10.0);
    }

    #[test]
    fn test_bone_extraction_causes_unrest() {
        let mut world = setup_world();

        let corpse_entity = world.spawn(Corpse { bone_yield: 5.0 }).id();
        world.spawn((
            Pop,
            Inventory::default(),
            StressTracker::default(),
            BoneExtractor,
            CurrentAction::ExtractingBone { target: corpse_entity },
        ));

        world.run_system_once(extract_bone_system);

        // Unrest should spike globally
        let unrest = world.get_resource::<UnrestTracker>().unwrap();
        assert!(unrest.level > 0.0);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/layer1/industry/bone_economy.rs

use bevy_ecs::prelude::*;
use crate::layer1::inventory::Inventory;
use crate::layer1::needs::StressTracker;
use crate::layer1::resources::ResourceType;

#[derive(Component)]
pub struct Corpse {
    pub bone_yield: f32,
}

#[derive(Component)]
pub struct BoneExtractor;

pub enum CurrentAction {
    ExtractingBone { target: Entity },
    Idle,
}

impl Default for CurrentAction {
    fn default() -> Self {
        CurrentAction::Idle
    }
}

#[derive(Resource, Default)]
pub struct UnrestTracker {
    pub level: f32,
}

pub fn extract_bone_system(
    mut commands: Commands,
    mut extractors: Query<(Entity, &mut Inventory, &mut StressTracker, &CurrentAction), With<BoneExtractor>>,
    mut corpses: Query<&mut Corpse>,
    mut unrest: ResMut<UnrestTracker>,
) {
    for (entity, mut inventory, mut stress, action) in extractors.iter_mut() {
        if let CurrentAction::ExtractingBone { target } = action {
            if let Ok(mut corpse) = corpses.get_mut(*target) {
                if corpse.bone_yield > 0.0 {
                    let amount = corpse.bone_yield;
                    inventory.add(ResourceType::CalciumAlloy, amount);
                    corpse.bone_yield = 0.0;
                    commands.entity(*target).despawn();

                    // Generate unrest
                    unrest.level += amount * 2.0;

                    // Stress the extractor
                    stress.accumulated_stress += 20.0;
                }
            }
            commands.entity(entity).remove::<CurrentAction>();
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design

- **Integration with Utility AI:** Register `ExtractBone` as a valid `ActionType` in the Utility AI if the colony is desperate (e.g., specific Edicts active or low on building materials).
- **Corpse State:** Allow extracting bones from existing `Grave` tiles, not just raw `Corpse` items on the ground.
- **Resource Types:** Add `CalciumAlloy` to the `ResourceType` enum.
- **Modifiers:** Pops with specific traits (e.g., "Pragmatist", "Cannibal", "Psychopath") should suffer less stress or cause less global unrest when performing this job.

## 6. Acceptance Criteria

- [ ] All tests pass.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >= 85%.
- [ ] Extracting bone from a corpse adds `CalciumAlloy` to the Pop's inventory and destroys the corpse.
- [ ] Extracting bone increases global unrest and local pop stress.

## 7. Technical Guidance
- **Resource Definition:** Ensure `CalciumAlloy` is properly registered in your `ResourceType` enum in `src/layer1/resources.rs` so it can be used in building costs.
- **ECS Best Practices:** Check `world.get_entity(entity).is_err()` properly as Bevy version requires it.

## 8. Questions
*Builder: add questions here if spec is unclear.*
*Architect: I will answer your questions as they come up.*
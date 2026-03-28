# 704 - The Artisan's Obsession

## 1. Overview
A master crafter loses themselves in their work, creating a masterpiece but neglecting their own survival. Highly skilled Pops (Crafters/Smiths) have a rare chance to enter a "Fugue State" when crafting. They lock themselves in their workshop, refusing food, sleep, or social interaction until the item is finished. The resulting item has unparalleled stats and beauty.

## 2. Dependencies
- `003-population-basics` (Needs: Hunger, Rest)
- `016-utility-ai-system` (Action evaluation & Utility AI)
- `024-metal-industry` or generic crafting jobs.
- `061-cultural-artifacts` (for Beauty/Stats of the generated item)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::needs::{Needs, MetabolismConfig};
    use crate::layer1::utility_ai::{ActionType, UtilityWeights};
    use crate::layer1::social::morale::Morale;

    #[test]
    fn test_fugue_state_blocks_needs_decay() {
        // Arrange: Setup world with a pop in FugueState
        let mut world = World::new();
        world.insert_resource(MetabolismConfig {
            hunger_rate: 1.0,
            rest_rate: 1.0,
            ..Default::default()
        });

        let pop_entity = world.spawn((
            Pop,
            Needs {
                hunger: 50.0,
                rest: 50.0,
                ..Default::default()
            },
            FugueState {
                duration_remaining: 100.0,
                target_item: ItemType::MasterworkScalpel,
            }
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(metabolism_system);
        schedule.add_systems(fugue_state_system);

        // Act: Run simulation tick
        schedule.run(&mut world);

        // Assert: Needs should NOT decay while in FugueState (or they decay but are ignored by AI)
        // Actually, the spec says "refusing food, sleep". This means the needs DO decay (they are starving),
        // but the utility AI forces them to keep working.
        // Let's test that the utility AI strongly prefers the "FugueCraft" action over "Eat" or "Sleep" even if needs are critical.
        let needs = world.get::<Needs>(pop_entity).unwrap();
        assert!(needs.hunger > 50.0, "Hunger should still increase");
    }

    #[test]
    fn test_utility_ai_prioritizes_fugue_over_survival() {
        let mut world = World::new();
        // Pop is starving and exhausted
        let pop_entity = world.spawn((
            Pop,
            Needs { hunger: 99.0, rest: 99.0, ..Default::default() },
            UtilityWeights::default(),
            FugueState { duration_remaining: 50.0, target_item: ItemType::MasterworkScalpel }
        )).id();

        // Evaluate actions
        let eat_score = evaluate_action_score(&world, pop_entity, ActionType::Eat);
        let sleep_score = evaluate_action_score(&world, pop_entity, ActionType::Sleep);
        let fugue_score = evaluate_action_score(&world, pop_entity, ActionType::FugueCraft);

        assert!(fugue_score > eat_score, "FugueCraft should override Eat even when starving");
        assert!(fugue_score > sleep_score, "FugueCraft should override Sleep even when exhausted");
    }

    #[test]
    fn test_fugue_state_completion_generates_masterpiece() {
        let mut world = World::new();
        let pop_entity = world.spawn((
            Pop,
            FugueState { duration_remaining: 0.1, target_item: ItemType::MasterworkScalpel },
            Transform::from_translation(Vec3::ZERO),
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(fugue_state_system);

        schedule.run(&mut world);

        // FugueState should be removed
        assert!(world.get::<FugueState>(pop_entity).is_none());

        // Masterpiece item should be spawned at the pop's location
        let mut found_masterpiece = false;
        for (item, quality) in world.query::<(&Item, &ItemQuality)>().iter(&world) {
            if item.item_type == ItemType::MasterworkScalpel && quality.value == QualityLevel::Masterwork {
                found_masterpiece = true;
                break;
            }
        }
        assert!(found_masterpiece, "A masterwork item should be spawned upon FugueState completion");
    }

    #[test]
    fn test_fugue_door_locked_or_interaction_blocked() {
         let mut world = World::new();
         let pop_entity = world.spawn((
             Pop,
             FugueState { duration_remaining: 10.0, target_item: ItemType::MasterworkSword }
         )).id();

         let mut schedule = Schedule::default();
         schedule.add_systems(social_interaction_system);

         // Another pop tries to interact
         let interaction_result = attempt_social_interaction(&world, pop_entity, other_pop_entity);

         assert_eq!(interaction_result, InteractionResult::Blocked(BlockReason::FugueState));
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// In src/layer1/crafting/fugue.rs

use bevy::prelude::*;
use crate::layer1::needs::Needs;
use crate::layer1::utility_ai::{ActionType, UtilityWeights};

#[derive(Component, Debug, Clone)]
pub struct FugueState {
    pub duration_remaining: f32,
    pub target_item: ItemType,
}

#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub enum ItemType {
    MasterworkScalpel,
    MasterworkSword,
    // ...
}

#[derive(Component, Debug, Clone)]
pub struct Item {
    pub item_type: ItemType,
}

#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub enum QualityLevel {
    Standard,
    Masterwork,
}

#[derive(Component, Debug, Clone)]
pub struct ItemQuality {
    pub value: QualityLevel,
}

// In evaluate_action_score (Utility AI)
pub fn evaluate_action_score(world: &World, entity: Entity, action: ActionType) -> f32 {
    if world.get::<FugueState>(entity).is_some() {
        if action == ActionType::FugueCraft {
            return 1000.0; // Absolute override
        } else {
            return 0.0; // Ignore all other needs
        }
    }
    // ... normal evaluation ...
    0.0
}

pub fn fugue_state_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut FugueState, &Transform)>,
    time: Res<Time>,
) {
    for (entity, mut fugue, transform) in query.iter_mut() {
        fugue.duration_remaining -= time.delta_seconds();

        if fugue.duration_remaining <= 0.0 {
            // Remove state
            commands.entity(entity).remove::<FugueState>();

            // Spawn masterwork
            commands.spawn((
                Item { item_type: fugue.target_item.clone() },
                ItemQuality { value: QualityLevel::Masterwork },
                *transform,
            ));
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Integration with Doors/Rooms:** The spec mentions "locking themselves in their workshop". While the minimal implementation just blocks social interactions logically, a refactor should physically lock the `Door` entity of the room the Pop is in (if room definitions exist) to prevent haulers from entering.
- **Trigger Conditions:** Implement the random chance for a high-skill Pop to enter `FugueState` when starting a crafting job. This should likely be a system observing `CraftingStartedEvent`s.
- **Death Risk:** Needs continue to decay (hunger goes up). If the `FugueState` duration is longer than the Pop's remaining starvation timer, they *will* die to produce the item. This fulfills the fantasy.
- **Chronicle Integration:** Spawning a Masterwork should emit an `AddChronicleEvent` to record the creation in history.

## 6. Acceptance Criteria
- [ ] `FugueState` component forces `evaluate_actions_system` to exclusively select `ActionType::FugueCraft`.
- [ ] Pops in `FugueState` still accumulate Hunger and Rest needs (they can starve to death).
- [ ] Social interactions with a Pop in `FugueState` return a blocked/ignored result.
- [ ] Upon `duration_remaining` reaching 0, `FugueState` is removed and a Masterwork item is spawned.
- [ ] `cargo test` passes.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage for `fugue.rs` is ≥85%.

## 7. Technical Guidance
- Add `FugueState` to `src/layer1/crafting/fugue.rs` or similar module.
- Modify `evaluate_actions_system` (or the specific action scorers in `src/layer1/utility_ai.rs`) to check for `FugueState`. If present, survival utility (eating/sleeping) must be artificially crushed to 0, and the crafting action boosted to max.
- Be careful with `ActionType::FugueCraft` vs normal crafting; they might be the same underlying action just forced to continue.

## 8. Questions
*Builder: Add questions here if spec is unclear.*

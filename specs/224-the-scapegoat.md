# 224: The Scapegoat

## Overview

Introduces a **Global Unrest** metric and a dark mechanism to manage it: **The Scapegoat**. When Unrest is high (due to low colony morale), the mob demands someone to blame. The player can "Denounce" a specific Pop (selected by the system as a suitable target based on traits like `Outsider`, `Mutant`, or low social standing).

Denouncing a Scapegoat dramatically lowers Global Unrest but inflicts severe penalties on the target (`Traumatized` trait, `Banishment`, or death) and causes "Guilt" among `Compassionate` pops.

## Dependencies

- `031` — Pop Morale (Unrest is the inverse of average Morale)
- `084` — Pop Traits (For target selection: `Outsider`, `Mutant`)
- `072` — Justice System (For `Banished` status reference)

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/layer1/scapegoat_tests.rs

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::pop::{Pop, Trait, MentalState};
    use crate::layer1::needs::Needs;
    use crate::layer1::unrest::{Unrest, calculate_unrest_system, identify_scapegoat_system, denounce_scapegoat_system, ScapegoatTarget, ScapegoatAction};

    #[test]
    fn test_unrest_calculation() {
        let mut world = World::new();
        world.insert_resource(Unrest::default());

        // Spawn pops with varying morale
        // Pop 1: High needs (Morale ~1.0)
        world.spawn((Pop, Needs { hunger: 1.0, rest: 1.0, leisure: 1.0 }));
        // Pop 2: Low needs (Morale ~0.0)
        world.spawn((Pop, Needs { hunger: 0.0, rest: 0.0, leisure: 0.0 }));

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(calculate_unrest_system);
        schedule.run(&mut world);

        let unrest = world.resource::<Unrest>();
        // Avg Morale = 0.5. Unrest = 1.0 - 0.5 = 0.5.
        assert!((unrest.level - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_scapegoat_identification_preference() {
        let mut world = World::new();
        // Insert Unrest high enough to trigger search
        world.insert_resource(Unrest { level: 0.8, ..Default::default() });

        // Normal Pop
        let normal = world.spawn((Pop, Trait::default())).id();
        // Outsider Pop (Should be preferred)
        let outsider = world.spawn((Pop, Trait::Outsider)).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(identify_scapegoat_system);
        schedule.run(&mut world);

        // Check if Outsider was tagged
        assert!(world.get::<ScapegoatTarget>(outsider).is_some());
        assert!(world.get::<ScapegoatTarget>(normal).is_none());
    }

    #[test]
    fn test_denounce_lowers_unrest_and_punishes_target() {
        let mut world = World::new();
        world.insert_resource(Unrest { level: 0.9, ..Default::default() });

        let target = world.spawn((
            Pop,
            Trait::Outsider,
            ScapegoatTarget,
        )).id();

        // Perform Denounce action (Exile)
        denounce_scapegoat_system(&mut world, target, ScapegoatAction::Exile);

        // Verify Unrest dropped
        let unrest = world.resource::<Unrest>();
        assert!(unrest.level < 0.5); // Significant drop

        // Verify Target is gone (Exiled/Despawned)
        assert!(world.get_entity(target).is_none());
    }

    #[test]
    fn test_denounce_punishment_trauma() {
        let mut world = World::new();
        world.insert_resource(Unrest { level: 0.9, ..Default::default() });

        let target = world.spawn((
            Pop,
            Trait::Outsider,
            ScapegoatTarget,
            MentalState::Normal,
        )).id();

        // Perform Denounce action (Shame/Punish but keep)
        denounce_scapegoat_system(&mut world, target, ScapegoatAction::PublicShame);

        // Verify Unrest dropped less than Exile
        let unrest = world.resource::<Unrest>();
        assert!(unrest.level < 0.9);

        // Verify Target gained Traumatized trait or breakdown
        let state = world.get::<MentalState>(target).unwrap();
        // Assuming Traumatized is a trait or state
        // For this test, let's assume it triggers a breakdown
        assert!(matches!(state, MentalState::Broken(_)));
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. `Unrest` Resource

```rust
// src/layer1/unrest.rs

use bevy_ecs::prelude::*;

#[derive(Resource, Default, Debug)]
pub struct Unrest {
    pub level: f32, // 0.0 to 1.0
}

#[derive(Component, Debug)]
pub struct ScapegoatTarget;

#[derive(Debug, Clone, Copy)]
pub enum ScapegoatAction {
    PublicShame,
    Exile,
    Execute, // Future scope
}
```

### 2. Unrest Calculation

```rust
// src/layer1/unrest.rs

use crate::layer1::needs::Needs;
use crate::layer1::pop::Pop;

pub fn calculate_unrest_system(
    mut unrest: ResMut<Unrest>,
    query: Query<&Needs, With<Pop>>,
) {
    let mut total_morale = 0.0;
    let mut count = 0;

    for needs in query.iter() {
        total_morale += needs.morale();
        count += 1;
    }

    if count > 0 {
        let avg_morale = total_morale / count as f32;
        unrest.level = (1.0 - avg_morale).clamp(0.0, 1.0);
    } else {
        unrest.level = 0.0;
    }
}
```

### 3. Identify Scapegoat

```rust
// src/layer1/unrest.rs

use crate::layer1::pop::Trait;

pub fn identify_scapegoat_system(
    mut commands: Commands,
    unrest: Res<Unrest>,
    query: Query<(Entity, Option<&Trait>), (With<Pop>, Without<ScapegoatTarget>)>,
) {
    if unrest.level < 0.6 {
        return; // No scapegoat needed if unrest is low
    }

    // Simple priority: Outsider -> Mutant -> Random
    let mut best_candidate = None;
    let mut best_score = 0;

    for (entity, trait_opt) in query.iter() {
        let score = match trait_opt {
            Some(Trait::Outsider) => 100,
            Some(Trait::Mutant) => 50, // Assuming Trait::Mutant exists or similar
            _ => 10,
        };

        if score > best_score {
            best_score = score;
            best_candidate = Some(entity);
        }
    }

    if let Some(entity) = best_candidate {
        commands.entity(entity).insert(ScapegoatTarget);
    }
}
```

### 4. Denounce Action

```rust
// src/layer1/unrest.rs

use crate::layer1::pop::{MentalState, MentalBreakType};

pub fn denounce_scapegoat_system(
    world: &mut World,
    target: Entity,
    action: ScapegoatAction,
) {
    // Apply Unrest reduction
    let reduction = match action {
        ScapegoatAction::Exile => 0.5,
        ScapegoatAction::PublicShame => 0.2,
        ScapegoatAction::Execute => 0.8,
    };

    if let Some(mut unrest) = world.get_resource_mut::<Unrest>() {
        unrest.level = (unrest.level - reduction).max(0.0);
    }

    // Apply Penalty
    match action {
        ScapegoatAction::Exile => {
            world.despawn(target);
        },
        ScapegoatAction::PublicShame => {
            if let Some(mut state) = world.get_mut::<MentalState>(target) {
                *state = MentalState::Broken(MentalBreakType::Daze); // Represent trauma/shame
            }
            world.entity_mut(target).remove::<ScapegoatTarget>();
        },
        _ => {}
    }
}
```

## REFACTOR Phase: Quality & Design

- **Trait Integration**: Ensure `Trait` enum actually has `Outsider` and `Mutant`. If not, add them or use generic `Trait::Unpopular` placeholder.
- **Cooldown**: Add a `last_denounce_tick` to `Unrest` to prevent spamming.
- **Side Effects**: Add logic to apply `MoodModifier::Guilt` to all pops with `Trait::Compassionate` after a denounce action.
- **UI**: Needs a "Colony Status" dashboard to show Unrest and the current Scapegoat candidate button.

## Acceptance Criteria

- [ ] `Unrest` resource tracks inverse morale.
- [ ] `ScapegoatTarget` is applied to vulnerable pops when unrest is high.
- [ ] `denounce_scapegoat_system` reduces unrest.
- [ ] `Exile` removes the entity.
- [ ] `PublicShame` triggers a mental break or debuff.
- [ ] Tests pass.

## Technical Guidance

- Use `world.despawn(entity)` carefully; ensure any links (jobs, beds) are cleaned up (handled by `on_despawn` hooks if they exist, or standard cleanup).
- `identify_scapegoat_system` should probably limit to 1 target at a time. Clear previous targets if they become invalid or a better one is found.

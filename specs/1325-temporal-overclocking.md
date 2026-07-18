# 1325: Temporal Overclocking

## 1. Overview
**Layer:** 1

**Fantasy:** Borrowing time from tomorrow.

**Mechanic:** "Chronal Fields" accelerate time in a specific room (Work speed x2). However, Pops inside age x2 faster and machines degrade x2 faster.

**Emergence:** You put your best researcher in the Chronal Lab to cure the plague. He finds the cure in a week (to you), but he emerges as an old man who missed his children growing up.

**Tension:** Crisis management (Speed) vs. The human cost of time.

## 2. Dependencies
- Base Layer 1 Population System (`crate::layer1::entities::pop::Pop`)
- Age System (`crate::layer1::lifecycle::Age`)
- Grid/Map System (`crate::layer1::map::GridPosition`)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;
    use crate::layer1::lifecycle::{Age, LifeStage};
    use crate::layer1::map::GridPosition;
    use crate::layer1::entities::pop::Pop;

    // Stub component representing a machine that can degrade
    #[derive(Component)]
    struct Machine {
        degradation: f32,
    }

    fn setup_app() -> World {
        let mut world = World::new();
        world
    }

    #[test]
    fn test_chronal_field_applies_temporal_modifier() {
        let mut world = setup_app();

        let mut schedule = bevy_ecs::schedule::Schedule::default();
        schedule.add_systems(apply_chronal_field_system);

        let chronal_field_pos = GridPosition { x: 5, y: 5 };

        // Spawn Chronal Field
        world.spawn((
            ChronalField { time_multiplier: 2.0 },
            chronal_field_pos,
        ));

        // Spawn Pop in the field
        let pop = world.spawn((
            Pop,
            Age { ticks_alive: 100, stage: LifeStage::Adult },
            chronal_field_pos,
        )).id();

        // Spawn Machine in the field
        let machine = world.spawn((
            Machine { degradation: 10.0 },
            chronal_field_pos,
        )).id();

        // Spawn Pop OUTSIDE the field
        let normal_pop = world.spawn((
            Pop,
            Age { ticks_alive: 100, stage: LifeStage::Adult },
            GridPosition { x: 0, y: 0 },
        )).id();

        schedule.run(&mut world);

        // Assert Pop in field has modified aging/speed
        let temporal_modifier = world.get::<TemporalModifier>(pop).unwrap();
        assert_eq!(temporal_modifier.multiplier, 2.0);

        // Assert Machine in field has modifier
        let machine_temporal = world.get::<TemporalModifier>(machine).unwrap();
        assert_eq!(machine_temporal.multiplier, 2.0);

        // Assert Normal Pop is unaffected
        assert!(world.get::<TemporalModifier>(normal_pop).is_none());
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy_ecs::prelude::*;
use crate::layer1::map::GridPosition;

#[derive(Component)]
pub struct ChronalField {
    pub time_multiplier: f32,
}

#[derive(Component)]
pub struct TemporalModifier {
    pub multiplier: f32,
}

pub fn apply_chronal_field_system(
    mut commands: Commands,
    fields: Query<(&ChronalField, &GridPosition)>,
    entities: Query<(Entity, &GridPosition), Without<ChronalField>>,
) {
    for (entity, pos) in entities.iter() {
        let mut in_field = false;
        let mut max_multiplier = 1.0;

        for (field, field_pos) in fields.iter() {
            if pos.x == field_pos.x && pos.y == field_pos.y {
                in_field = true;
                if field.time_multiplier > max_multiplier {
                    max_multiplier = field.time_multiplier;
                }
            }
        }

        if in_field {
            commands.entity(entity).insert(TemporalModifier { multiplier: max_multiplier });
        } else {
            commands.entity(entity).remove::<TemporalModifier>();
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Integration with Aging System:** The `TemporalModifier` needs to be consumed by `aging_system` (in `src/layer1/lifecycle.rs`) and machine degradation systems. Alternatively, we could directly modify the `fractional_age` accumulator (like `InsideChamber` does for `TemporalChamber`).
- **Integration with Work Speed:** Modify `src/layer1/execution/general_work.rs` to read `TemporalModifier` and multiply the base work speed.
- **Performance:** Iterating over all entities and checking against all fields is O(N*M). We should use a spatial grid or `HashMap` (e.g. `ChronalFieldGrid`) to look up fields by position in O(1) time.

## 6. Acceptance Criteria (Testable!)
- [ ] All RED phase tests pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >=85% for new code

## 7. Technical Guidance
- Look at `src/layer1/temporal_chamber.rs` and `aging_system` in `src/layer1/lifecycle.rs`. `TemporalChamber` handles slowing down time, you are doing the opposite (speeding it up).
- Ensure that the actual work efficiency system in `src/layer1/execution/general_work.rs` reads the `TemporalModifier` and multiplies its results.
- Ensure that `Machine` degradation (e.g., `StructureHealth` or `Maintenance`) also reads `TemporalModifier`.

## 8. Questions
*Builder: add questions here if spec is unclear.*

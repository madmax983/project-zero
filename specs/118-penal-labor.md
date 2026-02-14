# 118: Penal Labor

## 1. Overview

Currently, Pops arrested by the Justice System (`072`) become `Inmate`s and sit idle in Jail zones until their sentence expires. This is a waste of labor.

This feature allows the player to designate **Penal Zones** (e.g., a mine or a farm). Inmates can be assigned to jobs within these zones. They work with high efficiency but ignore happiness needs (Leisure/Social) and have a risk of revolt (future feature).

## 2. Dependencies

- `specs/072-justice-system.md` (Inmate component)
- `specs/056-designated-zones.md` (Zone system)
- `specs/016-utility-ai-system.md` (AI evaluation)

## 3. RED Phase: Tests First

These tests should be added to a new file `src/layer1/justice_labor_tests.rs`.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::justice::Inmate;
    use crate::layer1::zone::{ZoneGrid, ZoneType};
    use crate::layer1::utility_ai::{ActionType, evaluate_actions_system, PopAction};
    use crate::layer1::pop::Pop;
    use crate::layer1::map::GridPosition;
    use crate::layer1::designation::{Designation, DesignationType};

    #[test]
    fn test_zone_type_penal_exists() {
        // Simple compilation check that the enum variant exists
        let zone = ZoneType::Penal;
        assert_eq!(zone, ZoneType::Penal);
    }

    #[test]
    fn test_inmate_ignores_normal_work() {
        let mut world = World::new();
        // Setup systems/resources... (simplified)
        world.insert_resource(crate::layer1::utility_ai::UtilityConfig::default());
        world.insert_resource(crate::layer1::resources::ColonyResources::default());
        world.insert_resource(crate::shared::time::SimulationTime::default());
        world.insert_resource(crate::layer1::day_night::DayNightCycle::default());
        world.insert_resource(crate::layer1::taboo::TabooState::default());
        world.insert_resource(ZoneGrid::new(10, 10));

        // Spawn Inmate
        let inmate = world.spawn((
            Pop,
            Inmate { sentence_ticks: 100 },
            GridPosition { x: 0, y: 0 },
            PopAction::default(),
            crate::layer1::needs::Needs::default(),
            crate::layer1::utility_types::UtilityWeights::default(),
        )).id();

        // Designate mining in Normal Zone (None) at (1,0)
        world.spawn((
            Designation { designation_type: DesignationType::Mine },
            GridPosition { x: 1, y: 0 },
        ));

        // Run AI
        evaluate_actions_system(&mut world);

        // Assert Inmate is Idle
        let action = world.get::<PopAction>(inmate).unwrap();
        assert_eq!(action.current, ActionType::Idle, "Inmates should not work in non-penal zones");
    }

    #[test]
    fn test_inmate_works_in_penal_zone() {
        let mut world = World::new();
        // Setup...
        world.insert_resource(crate::layer1::utility_ai::UtilityConfig::default());
        world.insert_resource(crate::layer1::resources::ColonyResources::default());
        world.insert_resource(crate::shared::time::SimulationTime::default());
        world.insert_resource(crate::layer1::day_night::DayNightCycle::default());
        world.insert_resource(crate::layer1::taboo::TabooState::default());

        let mut zone_grid = ZoneGrid::new(10, 10);
        zone_grid.set(1, 0, ZoneType::Penal); // Set mining spot to Penal
        world.insert_resource(zone_grid);

        // Spawn Inmate
        let inmate = world.spawn((
            Pop,
            Inmate { sentence_ticks: 100 },
            GridPosition { x: 0, y: 0 },
            PopAction::default(),
            crate::layer1::needs::Needs::default(),
            crate::layer1::utility_types::UtilityWeights::default(),
        )).id();

        // Designate mining in Penal Zone
        world.spawn((
            Designation { designation_type: DesignationType::Mine },
            GridPosition { x: 1, y: 0 },
        ));

        // Run AI
        evaluate_actions_system(&mut world);

        // Assert Inmate works
        let action = world.get::<PopAction>(inmate).unwrap();
        assert_eq!(action.current, ActionType::Work, "Inmates MUST work in Penal zones");
    }

    #[test]
    fn test_free_pop_avoids_penal_zone() {
        let mut world = World::new();
        // Setup...
        world.insert_resource(crate::layer1::utility_ai::UtilityConfig::default());
        world.insert_resource(crate::layer1::resources::ColonyResources::default());
        world.insert_resource(crate::shared::time::SimulationTime::default());
        world.insert_resource(crate::layer1::day_night::DayNightCycle::default());
        world.insert_resource(crate::layer1::taboo::TabooState::default());

        let mut zone_grid = ZoneGrid::new(10, 10);
        zone_grid.set(1, 0, ZoneType::Penal);
        world.insert_resource(zone_grid);

        // Spawn Free Pop
        let pop = world.spawn((
            Pop,
            // No Inmate component
            GridPosition { x: 0, y: 0 },
            PopAction::default(),
            crate::layer1::needs::Needs::default(),
            crate::layer1::utility_types::UtilityWeights::default(),
        )).id();

        // Designate mining in Penal Zone
        world.spawn((
            Designation { designation_type: DesignationType::Mine },
            GridPosition { x: 1, y: 0 },
        ));

        // Run AI
        evaluate_actions_system(&mut world);

        // Assert Free Pop avoids Penal work
        let action = world.get::<PopAction>(pop).unwrap();
        assert_eq!(action.current, ActionType::Idle, "Free pops should not work in Penal zones");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

### 1. Update `ZoneType`

In `src/layer1/zone.rs`:

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ZoneType {
    // ... existing ...
    Jail,
    Pasture,
    Penal, // Add this
}
```

### 2. Update `evaluate_actions_system`

In `src/layer1/utility_ai.rs`:

1.  **Remove Inmate Filter**: Modify the query filter in `evaluate_actions_system` to allow entities with `Inmate` component to be processed.
    *   Change `.filter(|(_, _, _, _, action, _, _, _, inmate, _)| ...)` logic.
    *   Allow evaluation if `inmate.is_some()`.

2.  **Add ZoneGrid to Context**:
    *   Add `zone_grid: Option<Res<ZoneGrid>>` to `evaluate_actions_system` args (or fetch from world).
    *   Add `zone_grid: Option<&ZoneGrid>` to `WorldContext` struct.

3.  **Modify `evaluate_single_pop`**:
    *   Check if `data.inmate` is present.
    *   If Inmate:
        *   Block `Socialize`, `SatisfyLeisure`.
        *   For `Work` (and `Farm`, `Refine`, `Haul`, etc.), pass a filter or check the target position against `ZoneType::Penal`.
    *   If NOT Inmate:
        *   Block `Work` (etc.) if target position is `ZoneType::Penal`.

### 3. Implement Zone Check in Evaluation

Since `evaluate_work` iterates all designations, filtering inside `evaluate_work` might be cleaner.

Update `evaluate_work` signature:
```rust
pub fn evaluate_work<'a>(
    pop_pos: &GridPosition,
    weights: &UtilityWeights,
    designations: impl Iterator<Item = (Entity, &'a GridPosition, &'a Designation)>,
    zone_grid: Option<&ZoneGrid>, // New arg
    is_inmate: bool,              // New arg
) -> Option<(f32, Entity)>
```

Inside `evaluate_work`:
```rust
for (entity, pos, des) in designations {
    let zone = zone_grid.map(|z| z.get(pos.x, pos.y)).unwrap_or(ZoneType::None);

    if is_inmate {
        if zone != ZoneType::Penal { continue; }
    } else {
        if zone == ZoneType::Penal { continue; }
    }
    // ... rest of logic
}
```

Repeat for `evaluate_farm`, `evaluate_refine`, `evaluate_haul`.

## 5. REFACTOR Phase: Quality & Design

-   **Refactor Evaluation Functions**: Instead of passing `is_inmate` and `zone_grid` to every function, consider a `TargetFilter` closure or trait, but that might be complex for MVP. Passing explicit args is fine for now.
-   **Inmate AI Loop**: The `evaluate_single_pop` function is getting large. Grouping "Standard Actions" vs "Restricted Actions" might help.
-   **Performance**: Zone lookups are fast (array access).

## 6. Acceptance Criteria

- [ ] `ZoneType::Penal` exists.
- [ ] Inmates are evaluated by Utility AI.
- [ ] Inmates ONLY perform work/farm/haul actions if the target is in a Penal Zone.
- [ ] Free Pops NEVER perform work/farm/haul actions if the target is in a Penal Zone.
- [ ] Inmates do not attempt to Socialize or Satisfy Leisure.
- [ ] Tests in `justice_labor_tests.rs` pass.

## 7. Technical Guidance

-   Remember to update `WorldContext` definition in `src/layer1/utility_types.rs` if you add `zone_grid` to it.
-   Be careful with `evaluate_actions_system` filter. It currently *excludes* inmates. You must change it to *include* them.
-   Ensure `evaluate_mental_break` logic still applies to inmates (they can riot!).

## 8. Questions

-   *Builder:* Do inmates need tools?
    *   *Architect:* Yes, but for MVP let them work with bare hands (inefficient) or fetch tools if available in the zone.
-   *Builder:* Do inmates sleep?
    *   *Architect:* Yes, `SatisfyRest` is allowed. They should sleep in `ZoneType::Jail` beds if possible, but for MVP any bed or floor is fine.

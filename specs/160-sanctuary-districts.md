# 160: Sanctuary Districts

## 1. Overview

Sanctuary Districts are designated zones where the colony's law enforcement has no jurisdiction.
While crime (Vandalism, Assault) still occurs, it is ignored by the central authority.
Criminals and dissidents can hide here to avoid arrest, creating a "lawless" zone that serves as a pressure valve for the colony.

## 2. Dependencies

- `specs/072-justice-system.md` (Wanted, Justice System)
- `specs/056-designated-zones.md` (ZoneType, ZoneGrid)

## 3. RED Phase: Tests First

Write these tests in `src/layer1/justice_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::justice::{check_crime_system, evaluate_warden_action, Wanted};
    use crate::layer1::map::GridPosition;
    use crate::layer1::pop::Pop;
    use crate::layer1::unrest::{MentalBreakType, MentalState};
    use crate::layer1::zone::{ZoneGrid, ZoneType};
    use crate::layer1::utility_eval_types::PositionProxy;

    fn setup_world() -> World {
        let mut world = World::new();
        let mut zone_grid = ZoneGrid::new(10, 10);

        // Define Sanctuary Zone at (5,5)
        zone_grid.set(5, 5, ZoneType::Sanctuary);

        world.insert_resource(zone_grid);
        world
    }

    #[test]
    fn test_crime_in_sanctuary_ignored() {
        let mut world = setup_world();

        // Vandal inside Sanctuary
        let vandal = world.spawn((
            Pop,
            MentalState::Broken(MentalBreakType::Vandalize),
            GridPosition { x: 5, y: 5 }, // Inside Sanctuary
        )).id();

        // Run detection system
        let mut schedule = Schedule::default();
        schedule.add_systems(check_crime_system);
        schedule.run(&mut world);

        // Assert NOT Wanted
        assert!(world.get::<Wanted>(vandal).is_none(), "Pop in Sanctuary should not be marked Wanted");
    }

    #[test]
    fn test_crime_outside_sanctuary_punished() {
        let mut world = setup_world();

        // Vandal outside Sanctuary
        let vandal = world.spawn((
            Pop,
            MentalState::Broken(MentalBreakType::Vandalize),
            GridPosition { x: 0, y: 0 }, // Outside
        )).id();

        // Run detection system
        let mut schedule = Schedule::default();
        schedule.add_systems(check_crime_system);
        schedule.run(&mut world);

        // Assert Wanted
        assert!(world.get::<Wanted>(vandal).is_some(), "Pop outside Sanctuary SHOULD be marked Wanted");
    }

    #[test]
    fn test_warden_ignores_sanctuary_fugitive() {
        let mut world = setup_world();

        // Wanted criminal hiding in Sanctuary
        let fugitive = world.spawn((
            Pop,
            Wanted { severity: 1.0 },
            GridPosition { x: 5, y: 5 }, // Inside Sanctuary
        )).id();

        let criminals = vec![PositionProxy {
            entity: fugitive,
            pos: GridPosition { x: 5, y: 5 },
        }];

        // Warden outside
        let warden_pos = GridPosition { x: 4, y: 5 };

        // Evaluate action
        let result = evaluate_warden_action(&warden_pos, &criminals, &world.resource::<ZoneGrid>());

        // Assert NO target
        assert!(result.is_none(), "Warden should ignore fugitive in Sanctuary");
    }

    #[test]
    fn test_warden_pursues_outside_fugitive() {
        let mut world = setup_world();

        // Wanted criminal outside
        let fugitive = world.spawn((
            Pop,
            Wanted { severity: 1.0 },
            GridPosition { x: 0, y: 0 },
        )).id();

        let criminals = vec![PositionProxy {
            entity: fugitive,
            pos: GridPosition { x: 0, y: 0 },
        }];

        // Warden outside
        let warden_pos = GridPosition { x: 1, y: 0 };

        // Evaluate action
        let result = evaluate_warden_action(&warden_pos, &criminals, &world.resource::<ZoneGrid>());

        // Assert TARGET found
        assert!(result.is_some(), "Warden should pursue fugitive outside Sanctuary");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

### 1. Update `ZoneType` (`src/layer1/zone.rs`)

Add `Sanctuary` variant.

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ZoneType {
    // ... existing variants ...
    Sanctuary,
}
```

### 2. Update `check_crime_system` (`src/layer1/justice.rs`)

Inject `ZoneGrid` resource and check position.

```rust
pub fn check_crime_system(
    mut commands: Commands,
    query: Query<(Entity, &MentalState, &GridPosition), Without<Wanted>>,
    zone_grid: Res<ZoneGrid>,
) {
    for (entity, state, pos) in query.iter() {
        // Check if in Sanctuary
        if zone_grid.get(pos.x, pos.y) == ZoneType::Sanctuary {
            continue; // Ignore crime
        }

        if matches!(state, MentalState::Broken(MentalBreakType::Vandalize)) {
            commands.entity(entity).insert(Wanted { severity: 1.0 });
        }
    }
}
```

### 3. Update `evaluate_warden_action` (`src/layer1/justice.rs`)

Update signature to accept `ZoneGrid` reference.

```rust
pub fn evaluate_warden_action(
    guard_pos: &GridPosition,
    criminals: &[PositionProxy],
    zone_grid: &ZoneGrid, // New argument
) -> Option<(f32, Entity)> {
    let mut best_target = None;
    let mut min_dist = i32::MAX;

    for criminal in criminals {
        // Check if criminal is in Sanctuary
        if zone_grid.get(criminal.pos.x, criminal.pos.y) == ZoneType::Sanctuary {
            continue; // Cannot arrest in Sanctuary
        }

        let dist = manhattan_distance(guard_pos, &criminal.pos);
        if dist < min_dist {
            min_dist = dist;
            best_target = Some(criminal.entity);
        }
    }

    if let Some(target) = best_target {
        return Some((0.8, target));
    }
    None
}
```

### 4. Update Call Sites

Update `evaluate_actions_system` (or wherever `evaluate_warden_action` is called) to pass `zone_grid`.

## 5. REFACTOR Phase: Quality & Design

- **Sanctuary Mood**: Add a system that gives a temporary `MoodModifier` ("Safe in Sanctuary") to `Wanted` pops inside a Sanctuary.
- **Black Market**: Future integration where "Smugglers" spawn only in Sanctuary zones.
- **Vice**: Future integration where "Vice" needs are fulfilled more efficiently in Sanctuary zones.
- **UI**: Ensure Sanctuary zones are visualized distinctly (e.g., different border color).

## 6. Acceptance Criteria

- [ ] `ZoneType::Sanctuary` exists.
- [ ] `check_crime_system` ignores pops in Sanctuary.
- [ ] `evaluate_warden_action` ignores targets in Sanctuary.
- [ ] Tests pass.

## 7. Technical Guidance

- Modifying `evaluate_warden_action` signature will break existing tests/call sites. Update them all.
- Ensure `ZoneGrid` is available in the system that calls `evaluate_warden_action` (likely `evaluate_group_work`).
- Be careful with `PositionProxy` - ensure it has the correct position data.

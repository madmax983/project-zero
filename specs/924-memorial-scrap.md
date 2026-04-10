# 924 - Memorial Scrap

## 1. Overview

**Layer:** 2
**Fantasy:** Sacred reverence for destroyed ships preventing necessary recycling.
**Mechanic:** When a veteran fleet is destroyed, its wreckage is marked as "Memorial Scrap". Pops assigned to salvage it suffer massive morale penalties, potentially refusing the work order entirely.

## 2. Dependencies

- Ship Destruction events
- Salvage/Job assignment systems
- Pop Morale

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[derive(Component)]
    struct Wreckage { mass: f32 }

    #[derive(Component)]
    struct MemorialScrap;

    #[derive(Component)]
    struct SalvageJob { target: Entity }

    #[derive(Component)]
    struct PopMorale { value: f32 }

    #[derive(Event)]
    struct ShipDestroyedEvent { entity: Entity, is_veteran: bool }

    #[test]
    fn test_veteran_ship_leaves_memorial_scrap() {
        let mut app = App::new();
        app.add_event::<ShipDestroyedEvent>();
        app.add_systems(Update, handle_ship_destruction_system);

        let ship = app.world_mut().spawn_empty().id();
        app.world_mut().resource_mut::<Events<ShipDestroyedEvent>>().send(ShipDestroyedEvent { entity: ship, is_veteran: true });

        app.update();

        let mut query = app.world_mut().query::<(&Wreckage, &MemorialScrap)>();
        assert_eq!(query.iter(app.world()).count(), 1);
    }

    #[test]
    fn test_salvaging_memorial_scrap_reduces_morale() {
        let mut app = App::new();
        app.add_systems(Update, process_salvage_jobs_system);

        let scrap = app.world_mut().spawn((
            Wreckage { mass: 100.0 },
            MemorialScrap,
        )).id();

        let pop = app.world_mut().spawn((
            PopMorale { value: 80.0 },
            SalvageJob { target: scrap },
        )).id();

        app.update();

        let morale = app.world().get::<PopMorale>(pop).unwrap();
        assert!(morale.value < 80.0);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

// pub fn handle_ship_destruction_system(...) { ... }
// pub fn process_salvage_jobs_system(...) { ... }
```

## 5. REFACTOR Phase: Quality & Design

- The morale penalty could be structured as a memory or trauma event for better integration with existing narrative systems.

## 6. Acceptance Criteria

- [ ] All tests in RED phase pass
- [ ] Veteran ships spawn `MemorialScrap`
- [ ] Pops lose morale when assigned to `MemorialScrap` jobs

## 7. Technical Guidance

- Hook into existing `ShipDestroyedEvent` or similar if it exists.

## 8. Questions
*Builder: add questions here if spec is unclear.*

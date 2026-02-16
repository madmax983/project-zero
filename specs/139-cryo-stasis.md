# 139: Cryo-Stasis Vaults

## Overview

As the colony expands or faces crises (famine, oxygen loss, radiation), the ability to put Pops into suspended animation becomes critical. **Cryo-Stasis Vaults** allow players to freeze Pops, halting their resource consumption (Food, O2, Water) and aging, but removing them from the workforce.

Thawing a Pop is not without risk; they suffer from **Cryo-Sickness**, a temporary debuff that reduces their stats and movement speed until they recover.

## Dependencies

- `004` — Pop Entity
- `005` — Pop Needs
- `034` — Pop Health
- `007` — Housing (Concept of assigning pops to buildings)

## RED Phase: Tests First

Write these tests in `src/layer1/cryo_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::pop::{Pop, Speed};
    use crate::layer1::needs::{Needs, decay_needs_system};
    use crate::layer1::cryo::{CryoStasis, CryoSickness, enter_cryo_system, exit_cryo_system, cryo_sickness_decay_system};
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::map::GridPosition;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_cryo_stasis_halts_need_decay() {
        let mut world = World::new();
        // Spawn normal pop
        let pop1 = world.spawn((
            Pop,
            Needs { hunger: 1.0, rest: 1.0, leisure: 1.0 },
        )).id();
        // Spawn frozen pop
        let pop2 = world.spawn((
            Pop,
            Needs { hunger: 1.0, rest: 1.0, leisure: 1.0 },
            CryoStasis,
        )).id();

        // Run decay system multiple times
        for _ in 0..100 {
            world.run_system_once(decay_needs_system);
        }

        let needs1 = world.get::<Needs>(pop1).unwrap();
        let needs2 = world.get::<Needs>(pop2).unwrap();

        assert!(needs1.hunger < 1.0, "Normal pop should decay");
        assert_eq!(needs2.hunger, 1.0, "Frozen pop should NOT decay");
    }

    #[test]
    fn test_enter_cryo_applies_component() {
        let mut world = World::new();
        let pop = world.spawn((Pop, GridPosition { x: 0, y: 0 })).id();
        let pod = world.spawn((
            Building { building_type: BuildingType::CryoPod },
            GridPosition { x: 0, y: 0 },
            // Needs a component to trigger entry, e.g., CryoOrder
            crate::layer1::cryo::CryoOrder { target: pop },
        )).id();

        world.run_system_once(enter_cryo_system);

        assert!(world.get::<CryoStasis>(pop).is_some());
        // Pop should be "hidden" or inside the building (logic to be defined)
    }

    #[test]
    fn test_exit_cryo_applies_sickness() {
        let mut world = World::new();
        let pop = world.spawn((
            Pop,
            CryoStasis,
            Speed::default(),
        )).id();

        // Trigger exit (e.g. remove CryoStasis or use a command)
        // For this test, we assume a system handles the transition if marked
        world.entity(pop).insert(crate::layer1::cryo::ThawOrder);

        world.run_system_once(exit_cryo_system);

        assert!(world.get::<CryoStasis>(pop).is_none());
        assert!(world.get::<CryoSickness>(pop).is_some());

        // Check speed penalty
        let speed = world.get::<Speed>(pop).unwrap();
        assert!(speed.current < 1.0);
    }

    #[test]
    fn test_cryo_sickness_decays() {
        let mut world = World::new();
        let pop = world.spawn((
            Pop,
            CryoSickness { duration: 10, severity: 0.5 },
            Speed { base: 1.0, current: 0.5, accumulator: 0.0 },
        )).id();

        world.run_system_once(cryo_sickness_decay_system);

        let sickness = world.get::<CryoSickness>(pop).unwrap();
        assert_eq!(sickness.duration, 9);

        // Run until expiration
        for _ in 0..10 {
            world.run_system_once(cryo_sickness_decay_system);
        }

        assert!(world.get::<CryoSickness>(pop).is_none());
        let speed = world.get::<Speed>(pop).unwrap();
        assert_eq!(speed.current, 1.0, "Speed should recover");
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Define Components (`src/layer1/cryo.rs`)

```rust
use bevy_ecs::prelude::*;
use crate::layer1::pop::Speed;
use crate::layer1::needs::Needs;

#[derive(Component, Default, Debug)]
pub struct CryoStasis;

#[derive(Component, Debug)]
pub struct CryoSickness {
    pub duration: u32,
    pub severity: f32, // 0.0 to 1.0 (percent penalty)
}

#[derive(Component)]
pub struct CryoOrder {
    pub target: Entity,
}

#[derive(Component)]
pub struct ThawOrder;
```

### 2. Update `decay_needs_system` (`src/layer1/needs.rs`)

Modify the query to exclude `CryoStasis`.

```rust
pub fn decay_needs_system(
    mut query: Query<(&mut Needs, Option<&Traits>), Without<crate::layer1::cryo::CryoStasis>>,
    // ...
) {
    // ... logic remains same, but frozen pops are skipped
}
```

### 3. Implement Systems (`src/layer1/cryo.rs`)

```rust
pub fn enter_cryo_system(
    mut commands: Commands,
    mut pods: Query<(Entity, &mut CryoOrder)>,
    mut pops: Query<&mut GridPosition, With<crate::layer1::pop::Pop>>,
) {
    for (pod_entity, order) in pods.iter_mut() {
        if let Ok(_) = pops.get_mut(order.target) {
            commands.entity(order.target).insert(CryoStasis);
            commands.entity(pod_entity).remove::<CryoOrder>();
            // Visuals: Hide pop or move to same tile as pod
        }
    }
}

pub fn exit_cryo_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Speed), (With<CryoStasis>, With<ThawOrder>)>,
) {
    for (entity, mut speed) in query.iter_mut() {
        commands.entity(entity)
            .remove::<CryoStasis>()
            .remove::<ThawOrder>()
            .insert(CryoSickness { duration: 500, severity: 0.5 }); // 50% slow

        speed.current *= 0.5;
    }
}

pub fn cryo_sickness_decay_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut CryoSickness, &mut Speed)>,
) {
    for (entity, mut sick, mut speed) in query.iter_mut() {
        if sick.duration > 0 {
            sick.duration -= 1;
        } else {
            // Recover
            speed.current = speed.base; // Simplified reset
            commands.entity(entity).remove::<CryoSickness>();
        }
    }
}
```

### 4. Register Building (`src/layer1/building.rs`)

Add `CryoPod` to `BuildingType`.
Cost: Metal + Glass (or Stone).
Tech: `Cryogenics` (New tech or existing `Medical`).

## REFACTOR Phase: Quality & Design

- **Visuals**: Pops in cryo should be rendered blue or semi-transparent, or hidden inside the building.
- **UI**: Inspector should show "Status: Frozen".
- **AI**: Utility AI must strictly ignore `CryoStasis` pops to prevent them from trying to work.
- **Tech Tree**: Add `Cryogenics` tech.

## Acceptance Criteria

- [ ] `CryoStasis` component prevents Need decay.
- [ ] `CryoPod` building can be constructed.
- [ ] Pops can enter/exit cryo.
- [ ] Thawed pops suffer `CryoSickness`.
- [ ] Tests pass.

## Technical Guidance

- Ensure `movement_system` also excludes `CryoStasis` or checks for it.
- Update `evaluate_actions_system` to exclude `CryoStasis` pops.

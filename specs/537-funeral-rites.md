# 537 - Funeral Rites

## 1. Overview
A society is defined by how it treats its dead. Dead pops create "Corpse" items that cause grief. A "Funeral" ceremony must be performed by friends/family to convert Grief into "Closure". Unburied bodies cause "Haunted" moods.

## 2. Dependencies
- 003 Population Basics
- 005 Pop Needs
- 034 Pop Health and Damage

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;

    #[test]
    fn test_corpse_causes_grief() {
        // Arrange
        let mut world = World::new();
        let entity = world.spawn((Corpse, Position { x: 0, y: 0 })).id();
        let pop = world.spawn((Pop, Position { x: 1, y: 0 }, NeedGrief(0.0))).id();

        // Act
        let mut schedule = Schedule::default();
        schedule.add_systems(apply_corpse_grief_system);
        schedule.run(&mut world);

        // Assert
        let pop_grief = world.get::<NeedGrief>(pop).unwrap();
        assert!(pop_grief.0 > 0.0, "Corpse should cause grief in nearby pops");
    }

    #[test]
    fn test_funeral_ceremony_gives_closure() {
        // Arrange
        let mut world = World::new();
        let corpse = world.spawn((Corpse, Position { x: 0, y: 0 })).id();
        let pop = world.spawn((Pop, Position { x: 1, y: 0 }, NeedGrief(1.0))).id();

        world.spawn((FuneralTask { corpse_entity: corpse }, AssignedTo(pop)));

        // Act
        let mut schedule = Schedule::default();
        schedule.add_systems(perform_funeral_system);
        schedule.run(&mut world);

        // Assert
        assert!(world.get::<Corpse>(corpse).is_none(), "Corpse should be removed after funeral");
        let pop_grief = world.get::<NeedGrief>(pop).unwrap();
        assert_eq!(pop_grief.0, 0.0, "Funeral should provide closure and clear grief");
        assert!(world.get::<ClosureMemory>(pop).is_some(), "Pop should gain closure memory");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct Corpse;

#[derive(Component)]
pub struct NeedGrief(pub f32);

#[derive(Component)]
pub struct Position { pub x: i32, pub y: i32 }

#[derive(Component)]
pub struct Pop;

#[derive(Component)]
pub struct FuneralTask {
    pub corpse_entity: Entity,
}

#[derive(Component)]
pub struct AssignedTo(pub Entity);

#[derive(Component)]
pub struct ClosureMemory;

pub fn apply_corpse_grief_system(
    corpses: Query<&Position, With<Corpse>>,
    mut pops: Query<(&Position, &mut NeedGrief), With<Pop>>,
) {
    for (pop_pos, mut grief) in pops.iter_mut() {
        for corpse_pos in corpses.iter() {
            let dx = pop_pos.x - corpse_pos.x;
            let dy = pop_pos.y - corpse_pos.y;
            let dist_sq = dx * dx + dy * dy;
            if dist_sq < 25 {
                grief.0 += 1.0;
            }
        }
    }
}

pub fn perform_funeral_system(
    mut commands: Commands,
    tasks: Query<(Entity, &FuneralTask, &AssignedTo)>,
    mut pops: Query<&mut NeedGrief>,
) {
    for (task_entity, task, assigned) in tasks.iter() {
        if let Ok(mut grief) = pops.get_mut(assigned.0) {
            grief.0 = 0.0;
            commands.entity(assigned.0).insert(ClosureMemory);
        }
        commands.entity(task.corpse_entity).despawn();
        commands.entity(task_entity).despawn();
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Create specific structures for graves or crematoriums rather than generic removal.
- Integrate with `Memory` system properly to add closure as a memory event.
- Use the standard `Need` component structure for `NeedGrief`.
- Tie into `Utility AI` for prioritizing funeral tasks.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >=85% for new code
- [ ] Corpses correctly generate grief for nearby pops.
- [ ] Funerals remove the corpse and provide closure.

## 7. Technical Guidance
- Integrate with `layer1::needs` for the grief component.
- Add `Funeral` to `ActionType` in `layer1::utility_ai`.
- The corpse might need to be an actual `Item` dropped when a pop dies.

## 8. Questions
*Builder: add questions here if spec is unclear.*

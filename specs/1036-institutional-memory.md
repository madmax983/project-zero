# 1036: Institutional Memory

## 1. Overview
The knowledge of the ancients is just a coffee-stained notebook left by the old Chief Engineer. High-skill Pops automatically write "Manuals" or "Logs" when working. These items provide XP buffs to lower-skill Pops reading them or working nearby. Manuals degrade over time or can be destroyed.

## 2. Dependencies
- Layer 1 `Inventory` or `Item` system.
- Layer 1 `Skills` or `Experience` system.
- Layer 1 `Pop` action system (reading/studying).

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::skills::{SkillLevel, PopSkills};
    use crate::layer1::items::{Inventory, ItemType};
    use crate::layer1::institutional_memory::{Manual, InstitutionalMemoryPlugin};

    #[test]
    fn test_high_skill_pop_creates_manual() {
        let mut app = App::new();
        app.add_plugins(InstitutionalMemoryPlugin);

        let pop = app.world_mut().spawn((
            PopSkills {
                engineering: SkillLevel(10), // High skill
                ..default()
            },
            Inventory::default(),
            WorkingOnTask { task_type: TaskType::Engineering },
        )).id();

        // Simulate time passing to trigger manual creation
        app.update();
        app.update();

        let inventory = app.world().get::<Inventory>(pop).unwrap();
        assert!(inventory.contains(&ItemType::Manual(ManualType::Engineering)), "High skill pop working should create a manual.");
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// src/layer1/institutional_memory.rs
use bevy::prelude::*;
use crate::layer1::skills::{SkillLevel, PopSkills};
use crate::layer1::items::{Inventory, ItemType};

#[derive(Component)]
pub struct WorkingOnTask {
    pub task_type: TaskType,
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum TaskType {
    Engineering,
    // ...
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum ManualType {
    Engineering,
    // ...
}

pub struct InstitutionalMemoryPlugin;

impl Plugin for InstitutionalMemoryPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, create_manuals_system);
    }
}

fn create_manuals_system(
    mut query: Query<(&PopSkills, &mut Inventory, &WorkingOnTask)>,
) {
    for (skills, mut inventory, task) in query.iter_mut() {
        if task.task_type == TaskType::Engineering && skills.engineering.0 >= 10 {
            let manual_item = ItemType::Manual(ManualType::Engineering);
            if !inventory.contains(&manual_item) {
                inventory.add(manual_item);
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Expand the conditions for manual creation to involve a dedicated "Write Manual" task or random chance while working.
- Implement the "reading" mechanic where low-skill pops gain experience buffs while a manual is in their inventory or nearby.
- Introduce manual degradation (durability).

## 6. Acceptance Criteria (Testable!)
- [ ] Test `test_high_skill_pop_creates_manual` passes.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >= 85%.

## 7. Technical Guidance
- The `Inventory` and `ItemType` should ideally be standard enum variants if not already defined, making it easy to add `Manual`.
- Keep the condition simple for the MVP (e.g. `engineering >= 10`).

## 8. Questions
*Builder: add questions here if spec is unclear.*

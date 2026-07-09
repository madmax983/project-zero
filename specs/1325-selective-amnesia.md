# 1325: Selective Amnesia

## 1. Overview
**Layer:** 1

**Fantasy:** Eternal Sunshine of the Spotless Mind.

**Mechanic:** "Memory Wiping" medical procedure. Removes "Trauma" traits/memories. However, it also deletes linked Skills or Relationships.

**Emergence:** You wipe the PTSD of your best soldier so he can fight again. He forgets his wife (also a soldier), and she leaves him, causing a depression spiral.

**Tension:** Mental Health vs. Identity/Skills.

## 2. Dependencies
- ECS (`bevy_ecs`)
- Pop Memory system (`src/layer1/memory.rs`)
- Traits/Skills system

## 3. RED Phase: Tests First

```rust
// tests/selective_amnesia_tests.rs
use bevy::prelude::*;

#[test]
fn test_memory_wipe_removes_trauma() {
    let mut app = App::new();
    app.add_systems(Update, memory_wipe_system);
    app.add_event::<MemoryWipeEvent>();

    let pop_id = app.world_mut().spawn(PopMemory {
        memories: vec![
            Memory { id: 1, is_trauma: true, linked_skill: None },
            Memory { id: 2, is_trauma: false, linked_skill: None },
        ]
    }).id();

    app.world_mut().resource_mut::<Events<MemoryWipeEvent>>().send(MemoryWipeEvent { target: pop_id });

    app.update();

    let memory = app.world().get::<PopMemory>(pop_id).unwrap();
    assert_eq!(memory.memories.len(), 1);
    assert_eq!(memory.memories[0].id, 2, "Trauma memory should be removed.");
}

#[test]
fn test_memory_wipe_removes_linked_skills() {
    let mut app = App::new();
    app.add_systems(Update, memory_wipe_system);
    app.add_event::<MemoryWipeEvent>();

    let pop_id = app.world_mut().spawn((
        PopMemory {
            memories: vec![
                Memory { id: 1, is_trauma: true, linked_skill: Some(Skill::Combat) },
            ]
        },
        PopSkills { combat: 10 },
    )).id();

    app.world_mut().resource_mut::<Events<MemoryWipeEvent>>().send(MemoryWipeEvent { target: pop_id });

    app.update();

    let skills = app.world().get::<PopSkills>(pop_id).unwrap();
    assert_eq!(skills.combat, 0, "Linked skill should be wiped when trauma is wiped.");
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/layer1/selective_amnesia.rs
use bevy::prelude::*;

#[derive(Component)]
pub struct PopMemory {
    pub memories: Vec<Memory>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Memory {
    pub id: u32,
    pub is_trauma: bool,
    pub linked_skill: Option<Skill>,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Skill {
    Combat,
    Mining,
}

#[derive(Component)]
pub struct PopSkills {
    pub combat: u32,
}

#[derive(Event)]
pub struct MemoryWipeEvent {
    pub target: Entity,
}

pub fn memory_wipe_system(
    mut events: EventReader<MemoryWipeEvent>,
    mut query: Query<(&mut PopMemory, Option<&mut PopSkills>)>,
) {
    for ev in events.read() {
        if let Ok((mut mem, skills_opt)) = query.get_mut(ev.target) {
            let mut skills_to_wipe = Vec::new();

            // Retain only non-trauma memories, gather linked skills of wiped ones
            mem.memories.retain(|m| {
                if m.is_trauma {
                    if let Some(s) = m.linked_skill {
                        skills_to_wipe.push(s);
                    }
                    false
                } else {
                    true
                }
            });

            // Wipe linked skills
            if let Some(mut skills) = skills_opt {
                for s in skills_to_wipe {
                    match s {
                        Skill::Combat => skills.combat = 0,
                        _ => {}
                    }
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Events:** Bevy 0.15 uses `.read()` on `EventReader`, which is implemented properly.
- **Relationships:** Implement wiping for relationships/spouses if they are linked to the trauma memory.
- **Cost:** Wiping should be a medical job that requires a clinic and consumes power/medicine.

## 6. Acceptance Criteria
- [ ] All RED tests pass.
- [ ] Coverage >= 85%.
- [ ] `MemoryWipeEvent` successfully deletes trauma tags from a Pop's memory array.
- [ ] Linked skills are zeroed out or reduced.

## 7. Technical Guidance
- Be sure to update `PopSkills` correctly depending on how the skill system is modeled (might be a map or individual fields).

## 8. Questions
*Builder: Add any questions here.*

# 504 - Inherited Grudges

## 1. Overview
The Hatfields and McCoys in space. An argument over a spilled ration that echoes through centuries. When a Pop suffers a severe wrong (e.g., unjust imprisonment, starving while the Governor feasts) from another specific Pop, they generate a "Vendetta." This Vendetta is passed down genetically or culturally to their descendants. Descendants will spontaneously refuse to work with, or actively sabotage, the descendants of the original transgressor.

## 2. Dependencies
- `047` Pop Relationships
- `036` Pop Memory
- `391` Pop Relationships

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use super::*;
    // Mock imports for existing modules
    // use crate::layer1::relationships::{Relationship, Vendetta};
    // use crate::layer1::pop::{Pop, Descendant};

    #[derive(Component)]
    struct Pop {
        pub name: String,
        pub is_alive: bool,
    }

    #[derive(Component)]
    struct Relationship {
        pub vendettas: Vec<Entity>, // Target entities
    }

    #[derive(Component)]
    struct Descendant {
        pub parent: Entity,
    }

    fn pass_down_vendetta_system(
        mut commands: Commands,
        parents: Query<(Entity, &Relationship)>,
        mut descendants: Query<(Entity, &Descendant, Option<&mut Relationship>)>,
    ) {
        // Implementation omitted for RED phase. Should fail tests.
    }

    #[test]
    fn test_vendetta_inherited_by_descendant() {
        let mut app = App::new();

        let target_entity = app.world_mut().spawn(Pop { name: "McCoy".to_string(), is_alive: true }).id();
        let parent_entity = app.world_mut().spawn((
            Pop { name: "Hatfield Sr.".to_string(), is_alive: false },
            Relationship { vendettas: vec![target_entity] }
        )).id();
        let descendant_entity = app.world_mut().spawn((
            Pop { name: "Hatfield Jr.".to_string(), is_alive: true },
            Descendant { parent: parent_entity }
        )).id();

        app.add_systems(Update, pass_down_vendetta_system);
        app.update();

        let rel = app.world().get::<Relationship>(descendant_entity).unwrap();
        assert!(rel.vendettas.contains(&target_entity), "Descendant should inherit the vendetta against the target");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Pop {
    pub name: String,
    pub is_alive: bool,
}

#[derive(Component)]
pub struct Relationship {
    pub vendettas: Vec<Entity>,
}

#[derive(Component)]
pub struct Descendant {
    pub parent: Entity,
}

pub fn pass_down_vendetta_system(
    mut commands: Commands,
    parents: Query<(Entity, &Relationship)>,
    mut descendants: Query<(Entity, &Descendant, Option<&mut Relationship>)>,
) {
    for (descendant_entity, descendant, rel_opt) in descendants.iter_mut() {
        if let Ok((parent_entity, parent_rel)) = parents.get(descendant.parent) {
            if !parent_rel.vendettas.is_empty() {
                if let Some(mut rel) = rel_opt {
                    for v in parent_rel.vendettas.iter() {
                        if !rel.vendettas.contains(v) {
                            rel.vendettas.push(*v);
                        }
                    }
                } else {
                    commands.entity(descendant_entity).insert(Relationship { vendettas: parent_rel.vendettas.clone() });
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Vendetta Resolution**: How does a vendetta end? Add logic for an eventual peace treaty or forgiveness, perhaps driven by `036 Pop Memory` fading over time.
- **Dynamic Entities**: Target entities of vendettas shouldn't just be individual Pops (since they die). Vendettas should probably target Family/Lineage IDs or Factions.
- **System Frequency**: `pass_down_vendetta_system` only needs to run on `SpawnPopEvent` or when a descendant is generated, not every tick.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Descendants inherit `Vendetta`s from their parent entities.

## 7. Technical Guidance
- Integrate closely with the new births system to ensure `Descendant` mapping is robust.
- The Utility AI will need a new filter: a Scorer that heavily penalizes (or totally vetoes) assigning a Pop to work alongside an entity on their Vendetta list.

## 8. Questions
*Builder: add questions here if spec is unclear. Architect will address.*

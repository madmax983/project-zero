# 355: Generational Knowledge

## 1. Overview

The son of the master smith grows up in the forge. Skills are passed down, creating lineages of experts.

**Generational Knowledge** modifies the pop creation/birth system. When a new child pop is born, they scan the skills of their assigned "parents" (or mentors). If the parents have exceptionally high skill levels (e.g., Mining > 50), the child starts with a "Potential" bonus or partial XP in that specific skill. This causes families or colony lineages to naturally specialize over time, creating a "Mining Caste" or "Science Dynasty."

## 2. Dependencies

- `051` Pop Skills & XP (for the underlying skill tracking)
- `047` Pop Relationships / Pop Lifecycle (for parent-child linkage)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;
    use crate::layer1::pop::{PopBundle, Skills, SkillType};
    use crate::layer1::social::Lineage;

    #[test]
    fn test_child_inherits_skill_from_master_parent() {
        let mut world = World::new();
        let mut schedule = Schedule::default();
        schedule.add_systems(apply_generational_knowledge_system);

        // Create Parent with high Mining skill
        let mut parent_skills = Skills::default();
        parent_skills.set_level(SkillType::Mining, 60); // Above threshold
        parent_skills.set_level(SkillType::Farming, 10); // Below threshold

        let parent = world.spawn(parent_skills).id();

        // Create Child
        let child = world.spawn((
            PopBundle::default(),
            Skills::default(),
            Lineage { parents: vec![parent] },
            NewbornMarker, // A marker for the system to process them exactly once
        )).id();

        schedule.run(&mut world);

        // Child should have bonus in Mining but NOT Farming
        let child_skills = world.get::<Skills>(child).unwrap();
        assert!(child_skills.get_xp(SkillType::Mining) > 0.0);
        assert_eq!(child_skills.get_xp(SkillType::Farming), 0.0);

        // Marker should be removed
        assert!(world.get::<NewbornMarker>(child).is_err());
    }

    #[test]
    fn test_child_inherits_highest_skill_from_multiple_parents() {
        let mut world = World::new();
        let mut schedule = Schedule::default();
        schedule.add_systems(apply_generational_knowledge_system);

        // Parent 1 is Master Farmer
        let mut p1_skills = Skills::default();
        p1_skills.set_level(SkillType::Farming, 80);
        let parent1 = world.spawn(p1_skills).id();

        // Parent 2 is Master Scientist
        let mut p2_skills = Skills::default();
        p2_skills.set_level(SkillType::Science, 70);
        let parent2 = world.spawn(p2_skills).id();

        let child = world.spawn((
            PopBundle::default(),
            Skills::default(),
            Lineage { parents: vec![parent1, parent2] },
            NewbornMarker,
        )).id();

        schedule.run(&mut world);

        let child_skills = world.get::<Skills>(child).unwrap();
        assert!(child_skills.get_xp(SkillType::Farming) > 0.0);
        assert!(child_skills.get_xp(SkillType::Science) > 0.0);
    }

    #[test]
    fn test_no_inheritance_from_low_skill_parents() {
        let mut world = World::new();
        let mut schedule = Schedule::default();
        schedule.add_systems(apply_generational_knowledge_system);

        // Parent is mediocre
        let mut p_skills = Skills::default();
        p_skills.set_level(SkillType::Mining, 20);
        let parent = world.spawn(p_skills).id();

        let child = world.spawn((
            PopBundle::default(),
            Skills::default(),
            Lineage { parents: vec![parent] },
            NewbornMarker,
        )).id();

        schedule.run(&mut world);

        // Child gets nothing
        let child_skills = world.get::<Skills>(child).unwrap();
        assert_eq!(child_skills.get_xp(SkillType::Mining), 0.0);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy_ecs::prelude::*;
use crate::layer1::pop::{Skills, SkillType};

// Placed in `src/layer1/social.rs` or similar
#[derive(Component)]
pub struct Lineage {
    pub parents: Vec<Entity>,
}

#[derive(Component)]
pub struct NewbornMarker;

const MASTER_SKILL_THRESHOLD: u32 = 50;
const INHERITED_XP_BONUS: f32 = 500.0; // E.g., starts at level 5 instead of 0

pub fn apply_generational_knowledge_system(
    mut commands: Commands,
    mut newborn_query: Query<(Entity, &mut Skills, &Lineage), With<NewbornMarker>>,
    parent_query: Query<&Skills, Without<NewbornMarker>>, // Without handles edge cases where child is parent
) {
    for (entity, mut child_skills, lineage) in newborn_query.iter_mut() {

        for parent_entity in &lineage.parents {
            if let Ok(parent_skills) = parent_query.get(*parent_entity) {
                // Iterate over all possible skills (assuming SkillType::iter() exists, or check a list)
                let all_skills = vec![
                    SkillType::Mining,
                    SkillType::Farming,
                    SkillType::Building,
                    SkillType::Science,
                    SkillType::Social,
                    // ... etc
                ];

                for skill in all_skills {
                    if parent_skills.get_level(skill.clone()) >= MASTER_SKILL_THRESHOLD {
                        let current_xp = child_skills.get_xp(skill.clone());
                        child_skills.add_xp(skill, INHERITED_XP_BONUS); // Assuming add_xp exists, or do it manually
                    }
                }
            }
        }

        // Remove marker so we only process this once per lifetime
        commands.entity(entity).remove::<NewbornMarker>();
    }
}
```

## 5. REFACTOR Phase: Quality & Design

- **Potential vs. Flat XP:** Instead of just giving flat XP at birth, it's mechanically more interesting to give them a `SkillPotential` component which acts as a multiplier (e.g., `1.5x`) to XP gain for that specific skill for their whole life. This represents "talent" rather than innate knowledge.
- **Lineage Depth:** If you want to get fancy, track grandparents. But for MVP, immediate parents are fine.
- **Orphans/Clones:** Ensure the system safely ignores Pops born from Clone Vats (Spec 240) who don't have parents, unless you specifically want clones to inherit traits from the DNA template.

## 6. Acceptance Criteria

- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code
- [ ] Children born to parents with skills `> 50` receive an initial XP or learning rate boost in those skills.
- [ ] The system only processes newborns exactly once.
- [ ] Low-skilled parents pass down nothing.

## 7. Technical Guidance

- You will need to hook into the existing reproduction or pop spawning logic. Whenever a Pop is created naturally, attach `NewbornMarker` and `Lineage` with the parents' entity IDs.
- Ensure the `Skills` API supports safe mutation (e.g., `add_xp` or exposing mutable internal values) and that `SkillType` is easy to iterate over if you use an enum.

## 8. Questions

*Builder: add questions here if spec is unclear.*

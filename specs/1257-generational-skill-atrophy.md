# 1257: Generational Skill Atrophy

## Overview
A highly automated society loses the basic ability to survive when the machines stop. As a colony's automation level increases (e.g. from drones or auto-farms), the physical and survival skills of new Pop generations naturally decay. If automation is disabled (e.g. via EMP or glitch), the Pops become extremely inefficient at manual labor.

## Dependencies
- None

## RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::skills::{Skills, SkillType};
    use crate::layer1::social::old_guard::Generation;
    use crate::layer1::skills::generational_atrophy::{apply_skill_atrophy_system, AtrophiedSkill, AutomationLevel};

    #[test]
    fn test_high_automation_causes_skill_atrophy_in_new_generation() {
        let mut world = World::new();
        // Set a high automation level for the colony
        world.insert_resource(AutomationLevel { level: 80.0 }); // High automation

        // Spawn a Pop of a new generation with some starting skills
        let mut skills = Skills::default();
        skills.add_xp(SkillType::Farming, 500.0); // Would normally be level 2

        let pop = world.spawn((
            Generation::Immigrant, // "New" generation
            skills,
        )).id();

        // Run the system
        let mut schedule = Schedule::default();
        schedule.add_systems(apply_skill_atrophy_system);
        schedule.run(&mut world);

        // Verify that the skill has been atrophied and a marker added
        let atrophied = world.get::<AtrophiedSkill>(pop);
        assert!(atrophied.is_some(), "Pop should have AtrophiedSkill marker");

        let updated_skills = world.get::<Skills>(pop).unwrap();
        // Since get_xp does not expose a mutable reference to reduce it, the system reduces the underlying xp map
        assert!(*updated_skills.xp.get(&SkillType::Farming).unwrap_or(&0.0) < 500.0, "Skill XP should be reduced due to atrophy");
    }

    #[test]
    fn test_low_automation_does_not_cause_atrophy() {
        let mut world = World::new();
        // Low automation
        world.insert_resource(AutomationLevel { level: 10.0 });

        let mut skills = Skills::default();
        skills.add_xp(SkillType::Farming, 500.0);

        let pop = world.spawn((
            Generation::Immigrant,
            skills,
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(apply_skill_atrophy_system);
        schedule.run(&mut world);

        assert!(world.get::<AtrophiedSkill>(pop).is_none(), "Pop should not have AtrophiedSkill marker in low automation");
    }
}
```

## GREEN Phase: Minimal Implementation

```rust
use bevy_ecs::prelude::*;
use crate::layer1::skills::{Skills, SkillType};
use crate::layer1::social::old_guard::Generation;

#[derive(Resource, Default)]
pub struct AutomationLevel {
    pub level: f32, // 0.0 to 100.0
}

#[derive(Component)]
pub struct AtrophiedSkill;

pub fn apply_skill_atrophy_system(
    automation_level: Option<Res<AutomationLevel>>,
    mut query: Query<(Entity, &Generation, &mut Skills), Without<AtrophiedSkill>>,
    mut commands: Commands,
) {
    let automation = match automation_level {
        Some(res) => res.level,
        None => return,
    };

    // If automation is high, apply atrophy to physical skills of newer generations
    if automation > 50.0 {
        for (entity, generation, mut skills) in query.iter_mut() {
            if *generation == Generation::Immigrant { // Applies to later generations
                // Manually reduce the XP for physical skills
                let current_farming = *skills.xp.get(&SkillType::Farming).unwrap_or(&0.0);
                if current_farming > 0.0 {
                    if let Some(xp) = skills.xp.get_mut(&SkillType::Farming) {
                        *xp *= 0.5; // 50% penalty
                    }
                }

                let current_mining = *skills.xp.get(&SkillType::Mining).unwrap_or(&0.0);
                if current_mining > 0.0 {
                    if let Some(xp) = skills.xp.get_mut(&SkillType::Mining) {
                        *xp *= 0.5;
                    }
                }

                commands.entity(entity).insert(AtrophiedSkill);
            }
        }
    }
}
```

## REFACTOR Phase: Quality & Design
- Create an `AutomationLevel` system that continuously updates the colony's overall automation score based on active drone hubs and auto-factories.
- Generalize the skill atrophy to affect all physical skills (Farming, Mining, Forestry) based on a configurable list instead of hardcoding.
- `AtrophiedSkill` could store the percentage of atrophy applied, enabling gradual recovery if the automation level falls and the pop is forced to do manual labor.

## Acceptance Criteria
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Pops of Generation::Immigrant suffer skill XP reduction when `AutomationLevel` > 50.0

## Technical Guidance
- Implement this inside `src/layer1/skills/generational_atrophy.rs`.
- Wire up `AutomationLevel` calculation based on Layer 1 structures in a separate system, so this system just consumes the resource.

## Questions
*Builder: add questions here if spec is unclear.*

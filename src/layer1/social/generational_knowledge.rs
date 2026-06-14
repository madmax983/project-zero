#![allow(clippy::type_complexity)]
use bevy_ecs::prelude::*;
use crate::layer1::pop::PopBorn;
use crate::layer1::skills::{Skills, SkillType};
use crate::layer1::social::inherited_grudges::Lineage;

const MASTER_SKILL_THRESHOLD: u32 = 5;
const INHERITED_XP_BONUS: f32 = 500.0;

pub fn apply_generational_knowledge_system(
    mut events: EventReader<PopBorn>,
    mut queries: ParamSet<(Query<(&Lineage, &mut Skills)>, Query<&Skills>)>,
) {
    for event in events.read() {
        let parent_entity = queries
            .p0()
            .get(event.entity)
            .ok()
            .and_then(|(lineage, _)| lineage.parent_entity);

        if let Some(parent_entity) = parent_entity {
            let parent_skills_clone = if let Ok(parent_skills) = queries.p1().get(parent_entity) {
                Some(parent_skills.clone())
            } else {
                None
            };

            if let Some(parent_skills) = parent_skills_clone {
                if let Ok((_, mut child_skills)) = queries.p0().get_mut(event.entity) {
                    let all_skills = [
                        SkillType::Mining,
                        SkillType::Forestry,
                        SkillType::Farming,
                        SkillType::Construction,
                        SkillType::Crafting,
                        SkillType::Husbandry,
                        SkillType::Engineering,
                    ];

                    for skill in all_skills {
                        if parent_skills.get_level(skill) >= MASTER_SKILL_THRESHOLD {
                            child_skills.add_xp(skill, INHERITED_XP_BONUS);
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::pop::Pop;

    #[test]
    fn test_child_inherits_skill_from_master_parent() {
        let mut app = bevy::app::App::new();
        app.add_event::<PopBorn>();
        app.add_systems(bevy::app::Update, apply_generational_knowledge_system);

        let mut parent_skills = Skills::default();
        parent_skills.add_xp(SkillType::Mining, 10000.0); // Level 10
        parent_skills.add_xp(SkillType::Farming, 100.0); // Level 1

        let parent = app.world_mut().spawn((Pop, parent_skills)).id();

        let child = app.world_mut().spawn((
            Pop,
            Skills::default(),
            Lineage { parent_entity: Some(parent) },
        )).id();

        app.world_mut().send_event(PopBorn {
            entity: child,
            name: "Child".to_string(),
            tick: 0,
            source: "Birth".to_string(),
        });

        app.update();

        let child_skills = app.world().get::<Skills>(child).unwrap();
        assert!(child_skills.get_xp(SkillType::Mining) > 0.0);
        assert_eq!(child_skills.get_xp(SkillType::Farming), 0.0);
    }

    #[test]
    fn test_no_inheritance_from_low_skill_parents() {
        let mut app = bevy::app::App::new();
        app.add_event::<PopBorn>();
        app.add_systems(bevy::app::Update, apply_generational_knowledge_system);

        let mut p_skills = Skills::default();
        p_skills.add_xp(SkillType::Mining, 400.0); // Level 2
        let parent = app.world_mut().spawn((Pop, p_skills)).id();

        let child = app.world_mut().spawn((
            Pop,
            Skills::default(),
            Lineage { parent_entity: Some(parent) },
        )).id();

        app.world_mut().send_event(PopBorn {
            entity: child,
            name: "Child".to_string(),
            tick: 0,
            source: "Birth".to_string(),
        });

        app.update();

        let child_skills = app.world().get::<Skills>(child).unwrap();
        assert_eq!(child_skills.get_xp(SkillType::Mining), 0.0);
    }
}

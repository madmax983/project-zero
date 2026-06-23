use bevy_ecs::prelude::*;
use crate::layer1::skills::{SkillType, Skills};
use crate::layer1::social::inherited_grudges::Lineage;
use crate::layer1::pop::PopBorn;

#[derive(Component)]
pub struct NewbornMarker;

const MASTER_SKILL_THRESHOLD: u32 = 5; // A skill level of 5 means they are a master (equivalent to XP >= 2500)
const INHERITED_XP_BONUS: f32 = 500.0;

pub fn mark_newborns_system(
    mut commands: Commands,
    mut events: EventReader<PopBorn>,
) {
    for event in events.read() {
        if let Some(mut entity_cmds) = commands.get_entity(event.entity) {
            entity_cmds.insert(NewbornMarker);
        }
    }
}

pub fn apply_generational_knowledge_system(
    mut commands: Commands,
    mut newborn_query: Query<(Entity, &mut Skills, &Lineage), With<NewbornMarker>>,
    parent_query: Query<&Skills, Without<NewbornMarker>>,
) {
    for (entity, mut child_skills, lineage) in newborn_query.iter_mut() {
        if let Some(parent_entity) = lineage.parent_entity {
            if let Ok(parent_skills) = parent_query.get(parent_entity) {
                // Check all possible skills for mastery
                for skill in [
                    SkillType::Mining,
                    SkillType::Forestry,
                    SkillType::Farming,
                    SkillType::Construction,
                    SkillType::Crafting,
                    SkillType::Husbandry,
                    SkillType::Engineering,
                ] {
                    if parent_skills.get_level(skill) >= MASTER_SKILL_THRESHOLD {
                        child_skills.add_xp(skill, INHERITED_XP_BONUS);
                    }
                }
            }
        }

        // Remove marker so we only process this once per lifetime
        commands.entity(entity).remove::<NewbornMarker>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::pop::PopBundle;

    #[test]
    fn test_child_inherits_skill_from_master_parent() {
        let mut app = bevy::app::App::new();
        app.add_systems(bevy::app::Update, apply_generational_knowledge_system);

        // Create Parent with high Mining skill
        let mut parent_skills = Skills::default();
        parent_skills.add_xp(SkillType::Mining, 2500.0); // Level 5
        parent_skills.add_xp(SkillType::Farming, 100.0); // Level 1 (Below threshold)

        let parent = app.world_mut().spawn(parent_skills).id();

        // Create Child
        let child = app.world_mut().spawn((
            PopBundle::random(0, 0, &mut rand::thread_rng()),
            Lineage { parent_entity: Some(parent) },
            NewbornMarker, // A marker for the system to process them exactly once
        )).id();

        app.update();

        // Child should have bonus in Mining but NOT Farming
        let child_skills = app.world().get::<Skills>(child).unwrap();
        assert!(child_skills.get_xp(SkillType::Mining) > 0.0);
        assert_eq!(child_skills.get_xp(SkillType::Farming), 0.0);

        // Marker should be removed
        assert!(app.world().get::<NewbornMarker>(child).is_none());
    }

    #[test]
    fn test_no_inheritance_from_low_skill_parents() {
        let mut app = bevy::app::App::new();
        app.add_systems(bevy::app::Update, apply_generational_knowledge_system);

        // Parent is mediocre
        let mut p_skills = Skills::default();
        p_skills.add_xp(SkillType::Mining, 400.0); // Level 2
        let parent = app.world_mut().spawn(p_skills).id();

        let child = app.world_mut().spawn((
            PopBundle::random(0, 0, &mut rand::thread_rng()),
            Lineage { parent_entity: Some(parent) },
            NewbornMarker,
        )).id();

        app.update();

        // Child gets nothing
        let child_skills = app.world().get::<Skills>(child).unwrap();
        assert_eq!(child_skills.get_xp(SkillType::Mining), 0.0);
    }

    #[test]
    fn test_mark_newborns_system() {
        let mut app = bevy::app::App::new();
        app.add_event::<PopBorn>();
        app.add_systems(bevy::app::Update, mark_newborns_system);

        let child = app.world_mut().spawn(PopBundle::random(0, 0, &mut rand::thread_rng())).id();

        app.world_mut().send_event(PopBorn {
            entity: child,
            name: "Child".to_string(),
            tick: 0,
            source: "Birth".to_string(),
        });

        app.update();

        assert!(app.world().get::<NewbornMarker>(child).is_some());
    }
}

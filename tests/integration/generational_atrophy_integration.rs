use bevy::prelude::*;
use scale::layer1::skills::generational_atrophy::{
    apply_skill_atrophy_system, AtrophiedSkill, AutomationLevel,
};
use scale::layer1::skills::{SkillType, Skills};
use scale::layer1::social::old_guard::Generation;

#[test]
fn test_generational_atrophy_integration() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.world_mut()
        .insert_resource(AutomationLevel { level: 80.0 });
    app.add_systems(Update, apply_skill_atrophy_system);

    let mut skills = Skills::default();
    skills.add_xp(SkillType::Farming, 100.0);

    let pop = app.world_mut().spawn((Generation::Immigrant, skills)).id();

    app.update();

    assert!(app.world().get::<AtrophiedSkill>(pop).is_some());
    let updated_skills = app.world().get::<Skills>(pop).unwrap();
    assert_eq!(
        *updated_skills.xp.get(&SkillType::Farming).unwrap_or(&0.0),
        50.0
    );
}

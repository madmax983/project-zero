use bevy_ecs::prelude::*;
use scale::layer1::core::integration::update_automation_level_system;
use scale::layer1::drone::DroneHub;
use scale::layer1::skills::generational_atrophy::{apply_skill_atrophy_system, AtrophiedSkill, AutomationLevel};
use scale::layer1::skills::{SkillType, Skills};
use scale::layer1::social::old_guard::Generation;

#[test]
fn test_automation_level_and_atrophy_integration() {
    let mut world = World::new();

    world.insert_resource(AutomationLevel { level: 0.0 });

    // Spawn enough DroneHubs to trigger high automation (>50.0)
    world.spawn(DroneHub);
    world.spawn(DroneHub);
    world.spawn(DroneHub); // 3 * 20.0 = 60.0

    let mut skills = Skills::default();
    skills.add_xp(SkillType::Farming, 500.0);

    let pop = world.spawn((
        Generation::Immigrant,
        skills,
    )).id();

    let mut schedule = Schedule::default();
    schedule.add_systems((
        update_automation_level_system,
        apply_skill_atrophy_system.after(update_automation_level_system),
    ));
    schedule.run(&mut world);

    let automation_level = world.resource::<AutomationLevel>();
    assert_eq!(automation_level.level, 60.0, "AutomationLevel should be updated by DroneHubs");

    let atrophied = world.get::<AtrophiedSkill>(pop);
    assert!(atrophied.is_some(), "Pop should have AtrophiedSkill marker due to high automation");

    let updated_skills = world.get::<Skills>(pop).unwrap();
    assert!(*updated_skills.xp.get(&SkillType::Farming).unwrap_or(&0.0) < 500.0, "Skill XP should be reduced due to atrophy");
}

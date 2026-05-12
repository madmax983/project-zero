//! Generational Atrophy.
//!
//! Explores how high automation affects the physical skills of newer generations.
//! When automation exceeds a certain threshold, physical skills like Farming and Mining
//! begin to atrophy in immigrants and newer generations who rely on the automated systems.

use crate::layer1::skills::{SkillType, Skills};
use crate::layer1::social::old_guard::Generation;
use bevy_ecs::prelude::*;

/// Event triggered when generational atrophy is applied.
#[derive(Event, Debug, Clone)]
pub struct AtrophyAppliedEvent;

/// Resource tracking the colony's overall level of automation.
///
/// ## Examples
///
/// ```rust
/// use scale::layer1::skills::generational_atrophy::AutomationLevel;
///
/// let automation = AutomationLevel { level: 85.0 };
/// assert!(automation.level > 50.0); // High automation
/// ```
#[derive(Resource, Default)]
pub struct AutomationLevel {
    pub level: f32, // 0.0 to 100.0
}

/// Marker component added to Pops whose skills have been reduced by automation.
///
/// ## Examples
///
/// ```rust
/// use bevy_ecs::prelude::*;

/// use scale::layer1::skills::generational_atrophy::AtrophiedSkill;
///
/// let mut world = World::new();
/// let entity = world.spawn(AtrophiedSkill).id();
/// assert!(world.get::<AtrophiedSkill>(entity).is_some());
/// ```
#[derive(Component)]
pub struct AtrophiedSkill;

/// System that applies skill atrophy to newer generations in highly automated colonies.
///
/// If [`AutomationLevel`] is > 50.0, physical skills (Farming, Mining) for `Immigrant` Pops are halved.
///
/// ## Examples
///
/// ```rust
/// use bevy_ecs::prelude::*;

/// use scale::layer1::skills::{Skills, SkillType};
/// use scale::layer1::social::old_guard::Generation;
/// use scale::layer1::skills::generational_atrophy::{AutomationLevel, AtrophiedSkill, apply_skill_atrophy_system};
///
/// let mut world = World::new();
/// world.init_resource::<Events<scale::layer1::skills::generational_atrophy::AtrophyAppliedEvent>>();
/// world.insert_resource(AutomationLevel { level: 80.0 });
///
/// let mut skills = Skills::default();
/// skills.add_xp(SkillType::Farming, 100.0);
///
/// let pop = world.spawn((Generation::Immigrant, skills)).id();
///
/// let mut schedule = Schedule::default();
/// schedule.add_systems(apply_skill_atrophy_system);
/// schedule.run(&mut world);
///
/// assert!(world.get::<AtrophiedSkill>(pop).is_some());
/// let updated_skills = world.get::<Skills>(pop).unwrap();
/// assert_eq!(updated_skills.get_xp(SkillType::Farming), 50.0); // XP was halved
/// ```
pub fn apply_skill_atrophy_system(
    automation_level: Option<Res<AutomationLevel>>,
    mut query: Query<(Entity, &Generation, &mut Skills), Without<AtrophiedSkill>>,
    mut commands: Commands,
    mut atrophy_events: EventWriter<AtrophyAppliedEvent>,
) {
    let automation = match automation_level {
        Some(res) => res.level,
        None => return,
    };

    // If automation is high, apply atrophy to physical skills of newer generations
    if automation > 50.0 {
        for (entity, generation, mut skills) in query.iter_mut() {
            if *generation == Generation::Immigrant {
                // Applies to later generations
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
                atrophy_events.send(AtrophyAppliedEvent);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::layer1::skills::generational_atrophy::{
        apply_skill_atrophy_system, AtrophiedSkill, AutomationLevel,
    };
    use crate::layer1::skills::{SkillType, Skills};
    use crate::layer1::social::old_guard::Generation;
    use bevy_ecs::prelude::*;

    #[test]
    fn test_high_automation_causes_skill_atrophy_in_new_generation() {
        let mut world = World::new();
        // Set a high automation level for the colony
        world.insert_resource(AutomationLevel { level: 80.0 }); // High automation
        world.init_resource::<Events<super::AtrophyAppliedEvent>>();

        // Spawn a Pop of a new generation with some starting skills
        let mut skills = Skills::default();
        skills.add_xp(SkillType::Farming, 500.0); // Would normally be level 2

        let pop = world
            .spawn((
                Generation::Immigrant, // "New" generation
                skills,
            ))
            .id();

        // Run the system
        let mut schedule = Schedule::default();
        schedule.add_systems(apply_skill_atrophy_system);
        schedule.run(&mut world);

        // Verify that the skill has been atrophied and a marker added
        let atrophied = world.get::<AtrophiedSkill>(pop);
        assert!(atrophied.is_some(), "Pop should have AtrophiedSkill marker");

        let updated_skills = world.get::<Skills>(pop).unwrap();
        // Since get_xp does not expose a mutable reference to reduce it, the system reduces the underlying xp map
        assert!(
            *updated_skills.xp.get(&SkillType::Farming).unwrap_or(&0.0) < 500.0,
            "Skill XP should be reduced due to atrophy"
        );
    }

    #[test]
    fn test_low_automation_does_not_cause_atrophy() {
        let mut world = World::new();
        // Low automation
        world.insert_resource(AutomationLevel { level: 10.0 });
        world.init_resource::<Events<super::AtrophyAppliedEvent>>();

        let mut skills = Skills::default();
        skills.add_xp(SkillType::Farming, 500.0);

        let pop = world.spawn((Generation::Immigrant, skills)).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(apply_skill_atrophy_system);
        schedule.run(&mut world);

        assert!(
            world.get::<AtrophiedSkill>(pop).is_none(),
            "Pop should not have AtrophiedSkill marker in low automation"
        );
    }
}

#![allow(clippy::float_cmp)]
use bevy_ecs::prelude::*;
use std::collections::HashMap;

/// Types of skills a Pop can have.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SkillType {
    /// Mining rock.
    Mining,
    /// Chopping trees.
    Forestry,
    /// Producing food/fiber.
    Farming,
    /// Building and repairing structures.
    Construction,
    /// Refining resources.
    Crafting,
    /// Taming and caring for animals.
    Husbandry,
}

/// Component storing experience points for various skills.
#[derive(Component, Default, Clone, Debug)]
pub struct Skills {
    /// Map of skill type to accumulated XP.
    pub xp: HashMap<SkillType, f32>,
}

/// Event triggered when a Pop gains XP.
#[derive(Event, Debug, Clone)]
pub struct XpGainEvent {
    /// The entity gaining XP.
    pub entity: Entity,
    /// The skill type.
    pub skill: SkillType,
    /// The amount gained.
    pub amount: f32,
    /// The source of the XP gain (to prevent loops).
    pub source: XpSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct XpSource;

impl Skills {
    /// Adds XP to a specific skill.
    ///
    /// Note: This method does NOT emit `XpGainEvent` automatically to avoid
    /// circular dependencies with `EventWriter`. Systems should emit the event manually
    /// if they want to trigger side effects (like Quantum Twins).
    pub fn add_xp(&mut self, skill: SkillType, amount: f32) {
        let current = self.xp.entry(skill).or_insert(0.0);
        *current += amount;
    }

    /// Returns the current XP for a skill.
    #[must_use]
    pub fn get_xp(&self, skill: SkillType) -> f32 {
        *self.xp.get(&skill).unwrap_or(&0.0)
    }

    /// Calculates the level based on XP.
    /// Formula: Level = floor(sqrt(XP / 100))
    /// 0-99 XP = Lvl 0
    /// 100-399 XP = Lvl 1
    /// 400-899 XP = Lvl 2
    #[must_use]
    pub fn get_level(&self, skill: SkillType) -> u32 {
        let xp = self.get_xp(skill);
        // Using f32::sqrt requires the input to be non-negative, which XP is.
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let level = (xp / 100.0).sqrt().floor() as u32;
        level
    }

    /// Calculates efficiency multiplier based on skill level.
    /// Base 1.0 + 0.1 per level.
    #[must_use]
    pub fn get_efficiency(&self, skill: SkillType) -> f32 {
        let level = self.get_level(skill);
        #[allow(clippy::cast_precision_loss)]
        let level_f32 = level as f32;
        // Optimization: Use mul_add if appropriate, though simplistic here
        level_f32.mul_add(0.1, 1.0)
    }
}

/// Helper to get skill efficiency from an optional Skills component.
/// Returns 1.0 if no component exists.
#[must_use]
pub fn get_skill_efficiency(skills: Option<&Skills>, skill: SkillType) -> f32 {
    skills.map_or(1.0, |s| s.get_efficiency(skill))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skills_default_empty() {
        let skills = Skills::default();
        assert_eq!(skills.get_xp(SkillType::Mining), 0.0);
        assert_eq!(skills.get_level(SkillType::Mining), 0);
    }

    #[test]
    fn test_gain_xp_increases_level() {
        let mut skills = Skills::default();

        // Level 0 -> 1 requires 100 XP
        skills.add_xp(SkillType::Mining, 50.0);
        assert_eq!(skills.get_level(SkillType::Mining), 0);

        skills.add_xp(SkillType::Mining, 50.0);
        assert_eq!(skills.get_level(SkillType::Mining), 1);
    }

    #[test]
    fn test_efficiency_scaling() {
        let mut skills = Skills::default();

        // Level 0 efficiency = 1.0
        assert!((skills.get_efficiency(SkillType::Mining) - 1.0).abs() < f32::EPSILON);

        // Level 1 efficiency = 1.1 (10% bonus)
        skills.add_xp(SkillType::Mining, 100.0);
        assert!((skills.get_efficiency(SkillType::Mining) - 1.1).abs() < f32::EPSILON);
    }

    #[test]
    fn test_skill_types_exist() {
        let _ = SkillType::Mining;
        let _ = SkillType::Forestry;
        let _ = SkillType::Farming;
        let _ = SkillType::Construction;
        let _ = SkillType::Crafting;
    }
}

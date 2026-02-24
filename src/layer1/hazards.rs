use crate::layer1::health::Health;
use crate::layer1::skills::{SkillType, Skills};
use crate::layer1::structure::Structure;
use crate::layer1::utility_types::ActionType;
use crate::shared::log::MessageLog;
use bevy_ecs::prelude::*;
use rand::Rng;

/// Severity of a workplace accident.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum AccidentSeverity {
    /// Minor injury (10 damage).
    Minor,
    /// Major injury (40 damage).
    Major,
    /// Critical injury (80 damage + Amputation).
    Critical,
}

/// Event fired when a critical accident occurs.
#[derive(Event, Debug, Clone)]
pub struct AmputationEvent {
    /// The entity that lost a limb.
    pub entity: Entity,
}

/// Checks for workplace accidents and applies damage if one occurs.
pub fn handle_workplace_hazards(
    world: &mut World,
    pop_entity: Entity,
    action_type: ActionType,
    structure: Option<&Structure>,
    skills: &Skills,
    hazard_modifier: f64,
) {
    let base_risk = action_type.danger_level();

    // Danger is probability 0.0 to 1.0
    if base_risk <= 0.0 {
        return;
    }

    let skill_type = match action_type {
        ActionType::Tame => SkillType::Husbandry,
        // Default assumption for Work, Repair, etc.
        _ => SkillType::Construction,
    };

    let structure_ref = structure.unwrap_or(&Structure {
        current_hp: 100.0,
        max_hp: 100.0,
    }); // Dummy "Perfect" structure if none

    let risk = calculate_risk(base_risk, skills, skill_type, structure_ref, hazard_modifier);
    let mut rng = rand::thread_rng();

    if rng.gen_bool(risk.min(1.0)) {
        let severity_roll = rng.r#gen::<f32>();
        let severity = determine_severity(severity_roll);
        trigger_accident(world, pop_entity, severity);
    }
}

/// Calculates the probability of an accident occurring.
///
/// # Formula
/// `Risk = Base * MaintenanceFactor / SkillFactor`
///
/// *   **Maintenance Factor**: 1.0 (100% HP) to 3.0 (0% HP).
/// *   **Skill Factor**: 1.0 (Lvl 0) to 2.0 (Lvl 10).
#[must_use]
pub fn calculate_risk(
    base_risk: f64,
    skills: &Skills,
    skill_type: SkillType,
    structure: &Structure,
    hazard_modifier: f64,
) -> f64 {
    // 1. Maintenance Factor
    // 0% HP = 3.0x risk. 100% HP = 1.0x risk.
    // Formula: 1.0 + (1.0 - (current / max)) * 2.0
    let hp_percent = (structure.current_hp / structure.max_hp.max(1.0)).clamp(0.0, 1.0);
    let maintenance_factor = (1.0 - hp_percent).mul_add(2.0, 1.0);

    // 2. Skill Factor
    // Level 0 = 1.0x. Level 10 = 2.0x denominator (0.5x risk).
    let level = skills.get_level(skill_type);
    let skill_factor = f64::from(level).mul_add(0.1, 1.0);

    (base_risk * f64::from(maintenance_factor) / skill_factor) * hazard_modifier
}

/// Determines the severity of an accident based on a random roll (0.0 - 1.0).
///
/// *   0.00 - 0.80: Minor
/// *   0.80 - 0.95: Major
/// *   0.95 - 1.00: Critical
#[must_use]
pub fn determine_severity(roll: f32) -> AccidentSeverity {
    if roll < 0.80 {
        AccidentSeverity::Minor
    } else if roll < 0.95 {
        AccidentSeverity::Major
    } else {
        AccidentSeverity::Critical
    }
}

/// Applies accident consequences to the entity.
pub fn trigger_accident(world: &mut World, entity: Entity, severity: AccidentSeverity) {
    let damage = match severity {
        AccidentSeverity::Minor => 10.0,
        AccidentSeverity::Major => 40.0,
        AccidentSeverity::Critical => 80.0,
    };

    if let Some(mut health) = world.get_mut::<Health>(entity) {
        health.take_damage(damage);
    }

    if severity == AccidentSeverity::Critical {
        world.send_event(AmputationEvent { entity });
        if let Some(mut log) = world.get_resource_mut::<MessageLog>() {
            log.add_colored(
                "CRITICAL ACCIDENT: A limb was lost!",
                ratatui::style::Color::Red,
            );
        }
    } else if let Some(mut log) = world.get_resource_mut::<MessageLog>() {
        log.add(format!(
            "ACCIDENT: Worker injured! ({severity:?}, -{damage} HP)"
        ));
    }
}

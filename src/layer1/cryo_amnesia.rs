use bevy_ecs::prelude::*;
use crate::layer1::skills::{Skills, SkillType};
use crate::layer1::cryo::{ThawOrder, CryoStasis};
use crate::layer1::memory::{Memories, MemoryType};
use crate::shared::time::SimulationTime;

#[derive(Component, Default)]
pub struct CryoAmnesia {
    pub severity: f32, // Decays over time
    pub false_memory: Option<String>,
}

/// Applies CryoAmnesia component when a Pop is thawing.
pub fn cryo_thaw_system(
    mut commands: Commands,
    mut query: Query<(Entity, Option<&mut Memories>), (With<ThawOrder>, With<CryoStasis>)>,
    time: Option<Res<SimulationTime>>,
) {
    let current_tick = time.map_or(0, |t| t.tick);
    for (entity, mut memories) in &mut query {
        commands.entity(entity).insert(CryoAmnesia {
            severity: 0.5, // Start with a default severity of 50%
            false_memory: Some("I remember a different sky...".to_string()),
        });

        if let Some(ref mut mems) = memories {
            mems.add(MemoryType::CryoFalseMemory, current_tick);
        } else {
            let mut new_mems = Memories::default();
            new_mems.add(MemoryType::CryoFalseMemory, current_tick);
            commands.entity(entity).insert(new_mems);
        }
    }
}

/// Computes the effective skill considering the penalty from CryoAmnesia.
#[must_use]
pub fn get_effective_skill(skills: &Skills, skill: SkillType, amnesia: Option<&CryoAmnesia>) -> f32 {
    let base_skill = skills.get_xp(skill);
    if let Some(amnesia) = amnesia {
        // Lower the effective XP by the severity percentage
        let penalty = base_skill * amnesia.severity;
        (base_skill - penalty).max(0.0)
    } else {
        base_skill
    }
}

/// Empty stub as the get_effective_skill handles the penalty for now per spec.
pub fn amnesia_skill_penalty_system() {}

/// Decays the severity of CryoAmnesia over time.
pub fn amnesia_recovery_system(
    mut commands: Commands,
    mut amnesiacs: Query<(Entity, &mut CryoAmnesia)>,
) {
    for (entity, mut amnesia) in &mut amnesiacs {
        // Simple linear decay
        amnesia.severity -= 0.001;
        if amnesia.severity <= 0.0 {
            commands.entity(entity).remove::<CryoAmnesia>();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::App;
    use bevy_app::Update;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_systems(Update, (cryo_thaw_system, amnesia_recovery_system));
        app
    }

    #[test]
    fn test_thawing_pop_gains_amnesia() {
        let mut app = setup_app();
        let pop = app.world_mut().spawn((ThawOrder, CryoStasis)).id();

        app.update();

        assert!(app.world().get::<CryoAmnesia>(pop).is_some());
        let amnesia = app.world().get::<CryoAmnesia>(pop).unwrap();
        assert!(amnesia.severity > 0.0);
    }

    #[test]
    fn test_amnesia_reduces_effective_skill() {
        let mut skills = Skills::default();
        skills.add_xp(SkillType::Mining, 100.0); // Level 1 equivalent

        let amnesia = CryoAmnesia { severity: 0.5, false_memory: None };

        let effective = get_effective_skill(&skills, SkillType::Mining, Some(&amnesia));
        assert!(effective < 100.0);
        assert_eq!(effective, 50.0); // 50% penalty
    }

    #[test]
    fn test_amnesia_recovery_system() {
        let mut app = setup_app();
        let pop = app.world_mut().spawn(CryoAmnesia { severity: 0.0005, false_memory: None }).id();

        // 1 tick, decays by 0.001 -> severity becomes -0.0005 -> removed
        app.update();

        assert!(app.world().get::<CryoAmnesia>(pop).is_none());
    }
}

use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct TinkeringTarget {
    pub target: Entity,
}

#[derive(Component)]
pub struct ForceTinkerOutcome {
    pub success: bool,
}

#[derive(Component)]
pub struct EfficiencyMultiplier {
    pub value: f32,
}

use crate::layer1::architecture::structure::Structure;
use crate::layer1::skills::{SkillType, Skills};
use rand::Rng;

pub fn obsessive_optimization_system(
    mut commands: Commands,
    mut pops_query: Query<(
        Entity,
        &TinkeringTarget,
        &Skills,
        Option<&ForceTinkerOutcome>,
    )>,
    mut target_query: Query<(&mut Structure, &mut EfficiencyMultiplier)>,
) {
    for (pop_entity, tinkering_target, skills, force_outcome) in pops_query.iter_mut() {
        if skills.get_xp(SkillType::Engineering) > 80.0 {
            if let Ok((mut structure, mut efficiency)) =
                target_query.get_mut(tinkering_target.target)
            {
                let success = if let Some(outcome) = force_outcome {
                    outcome.success
                } else {
                    let mut rng = rand::thread_rng();
                    rng.gen_bool(0.5) // Example probability
                };

                if success {
                    // Refactor Phase: Cap efficiency at 2.0 to prevent infinite stacking.
                    efficiency.value = (efficiency.value + 0.1).min(2.0);
                } else {
                    structure.current_hp = (structure.current_hp - 10.0).max(0.0);
                }
            }
        }
        commands.entity(pop_entity).remove::<TinkeringTarget>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::architecture::building::Building;
    use crate::layer1::architecture::structure::Structure;
    use crate::layer1::pop::Pop;
    use crate::layer1::skills::{SkillType, Skills};

    fn setup_app() -> bevy_app::App {
        let mut app = bevy_app::App::new();
        app.add_systems(bevy_app::Update, obsessive_optimization_system);
        app
    }

    #[test]
    fn test_tinker_success_boosts_efficiency() {
        let mut app = setup_app();

        // Target machine
        let machine = app
            .world_mut()
            .spawn((
                Building::default(),
                Structure {
                    current_hp: 100.0,
                    max_hp: 100.0,
                },
                EfficiencyMultiplier { value: 1.0 },
            ))
            .id();

        // Engineer pop with high skill
        let mut skills = Skills::default();
        skills.add_xp(SkillType::Engineering, 10000.0); // Very high skill

        let _engineer = app
            .world_mut()
            .spawn((
                Pop::default(),
                skills,
                TinkeringTarget { target: machine },
                ForceTinkerOutcome { success: true }, // Test harness component
            ))
            .id();

        app.update();

        let efficiency = app.world().get::<EfficiencyMultiplier>(machine).expect("Component should exist or System should run");
        assert!(
            efficiency.value > 1.0,
            "Successful tinker should boost efficiency"
        );
    }

    #[test]
    fn test_tinker_success_capped_efficiency() {
        let mut app = setup_app();

        // Target machine near cap
        let machine = app
            .world_mut()
            .spawn((
                Building::default(),
                Structure {
                    current_hp: 100.0,
                    max_hp: 100.0,
                },
                EfficiencyMultiplier { value: 1.95 },
            ))
            .id();

        let mut skills = Skills::default();
        skills.add_xp(SkillType::Engineering, 10000.0);

        let _engineer = app
            .world_mut()
            .spawn((
                Pop::default(),
                skills,
                TinkeringTarget { target: machine },
                ForceTinkerOutcome { success: true },
            ))
            .id();

        app.update();

        let efficiency = app.world().get::<EfficiencyMultiplier>(machine).expect("Component should exist or System should run");
        assert!(
            (efficiency.value - 2.0).abs() < f32::EPSILON,
            "Efficiency should cap at 2.0"
        );
    }

    #[test]
    fn test_tinker_failure_damages_machine() {
        let mut app = setup_app();

        // Target machine
        let machine = app
            .world_mut()
            .spawn((
                Building::default(),
                Structure {
                    current_hp: 100.0,
                    max_hp: 100.0,
                },
                EfficiencyMultiplier { value: 1.0 },
            ))
            .id();

        // Engineer pop with high skill but fails
        let mut skills = Skills::default();
        skills.add_xp(SkillType::Engineering, 8000.0); // High skill

        let _engineer = app
            .world_mut()
            .spawn((
                Pop::default(),
                skills,
                TinkeringTarget { target: machine },
                ForceTinkerOutcome { success: false }, // Test harness component
            ))
            .id();

        app.update();

        let condition = app.world().get::<Structure>(machine).expect("Component should exist or System should run");
        assert!(
            condition.current_hp < 100.0,
            "Failed tinker should damage the machine"
        );
    }
}

use crate::layer1::artifacts::{ArtifactAura, AuraEffect};
use crate::layer1::building::Building;
use crate::layer1::mind::utility_types::{ActionType, PopAction};
use crate::layer1::pop::Pop;
use crate::layer1::social::unrest::Unrest;
use bevy_ecs::prelude::*;

/// Component indicating a structure has been vandalized.
#[derive(Component)]
pub struct Vandalized;

pub const VANDALISM_AURA_MULTIPLIER: f32 = 1.5;

/// AI Evaluation: When global Unrest is high, Pops with nothing else to do
/// may decide to vandalize cultural structures.
pub fn evaluate_vandalism_targets(
    q_structures: Query<Entity, (With<Building>, Without<Vandalized>)>,
    unrest: Res<Unrest>,
    mut q_pops: Query<&mut PopAction, With<Pop>>,
) {
    if unrest.level > 0.5 {
        for mut action in q_pops.iter_mut() {
            if action.current == ActionType::Idle {
                if let Some(_target) = q_structures.iter().next() {
                    action.current = ActionType::Vandalize;
                }
            }
        }
    }
}

/// Execution: Pops that have the Vandalize action currently target an official structure and deface it.
pub fn process_vandalism(
    mut commands: Commands,
    mut q_pops: Query<&mut PopAction>,
    mut q_structures: Query<(Entity, &mut ArtifactAura), With<Building>>,
) {
    for mut action in q_pops.iter_mut() {
        if action.current == ActionType::Vandalize {
            if let Some((entity, mut aura)) = q_structures.iter_mut().next() {
                if let AuraEffect::StressModifier(amount) = aura.effect {
                    if amount < 0.0 {
                        aura.effect = AuraEffect::StressModifier(-amount * VANDALISM_AURA_MULTIPLIER);
                        commands.entity(entity).insert(Vandalized);
                    }
                }
            }
            action.current = ActionType::Idle;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::building::BuildingType;


    #[test]
    fn test_high_unrest_triggers_vandalism_action() {
        let mut app = App::new();
        app.insert_resource(Unrest { level: 0.8, modifiers: vec![] });
        app.add_systems(Update, evaluate_vandalism_targets);

        let _structure = app.world_mut().spawn(Building { building_type: BuildingType::Statue }).id();
        let pop = app.world_mut().spawn((
            Pop,
            PopAction { current: ActionType::Idle, current_utility: 0.0, ticks_committed: 0 }
        )).id();

        app.update();

        let action = app.world().get::<PopAction>(pop).unwrap();
        assert_eq!(action.current, ActionType::Vandalize);
    }

    #[test]
    fn test_vandalism_inverts_morale_aura() {
        let mut app = App::new();
        app.add_systems(Update, process_vandalism);

        let structure = app.world_mut().spawn((
            Building { building_type: BuildingType::Statue },
            ArtifactAura { effect: AuraEffect::StressModifier(-10.0), radius: 5.0 },
        )).id();

        app.world_mut().spawn(PopAction {
            current: ActionType::Vandalize,
            current_utility: 0.0,
            ticks_committed: 0
        });

        app.update();

        let aura = app.world().get::<ArtifactAura>(structure).unwrap();
        if let AuraEffect::StressModifier(effect) = aura.effect {
            assert!(effect > 0.0, "Morale effect should be inverted after vandalism");
        } else {
            panic!("Wrong effect type");
        }
        assert!(app.world().get::<Vandalized>(structure).is_some(), "Structure should be marked as defaced");
    }
}

use crate::layer1::mind::utility_types::ActionType;
use crate::layer1::mind::utility_types::PopAction;
use crate::layer1::mind::utility_types::StartPlan;
use crate::layer1::social::unrest::Unrest;
use bevy_ecs::prelude::*;

#[derive(Component, Clone)]
pub struct MoraleAura {
    pub effect: f32,
}

#[derive(Component)]
pub struct OfficialStructure;

#[derive(Component)]
pub struct Defaced;

pub fn evaluate_vandalism_targets(
    unrest: Option<Res<Unrest>>,
    q_structures: Query<Entity, (With<OfficialStructure>, Without<Defaced>)>,
    mut commands: Commands,
    mut q_pops: Query<(Entity, &PopAction), Without<StartPlan>>,
) {
    let current_unrest = match unrest {
        Some(u) => u.level,
        None => 0.0,
    };

    if current_unrest > 0.5 {
        for (pop_entity, action) in q_pops.iter_mut() {
            if action.current == ActionType::Idle {
                if let Some(target) = q_structures.iter().next() {
                    commands.entity(pop_entity).insert(StartPlan {
                        action: ActionType::Vandalize,
                        target: Some(target),
                    });
                }
            }
        }
    }
}

pub fn process_vandalism(
    mut commands: Commands,
    q_pops: Query<(Entity, &StartPlan)>,
    mut q_structures: Query<(Entity, &mut MoraleAura), With<OfficialStructure>>,
) {
    for (pop_entity, plan) in q_pops.iter() {
        if plan.action == ActionType::Vandalize {
            if let Some(target_entity) = plan.target {
                if let Ok((entity, mut aura)) = q_structures.get_mut(target_entity) {
                    aura.effect = -aura.effect;
                    commands.entity(entity).insert(Defaced);
                }
            }
            // Remove the StartPlan as the action is executed
            commands.entity(pop_entity).remove::<StartPlan>();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_high_unrest_triggers_vandalism_action() {
        let mut app = App::new();
        app.add_systems(Update, evaluate_vandalism_targets);

        app.world_mut().insert_resource(Unrest {
            level: 0.8,
            ..Default::default()
        });

        let structure = app.world_mut().spawn(OfficialStructure).id();
        let pop = app
            .world_mut()
            .spawn((PopAction {
                current: ActionType::Idle,
                ..Default::default()
            },))
            .id();

        app.update();

        // The StartPlan should be inserted
        let plan = app.world().get::<StartPlan>(pop).unwrap();
        assert_eq!(plan.action, ActionType::Vandalize);
        assert_eq!(plan.target, Some(structure));
    }

    #[test]
    fn test_low_unrest_no_vandalism() {
        let mut app = App::new();
        app.add_systems(Update, evaluate_vandalism_targets);

        app.world_mut().insert_resource(Unrest {
            level: 0.2,
            ..Default::default()
        });

        let _structure = app.world_mut().spawn(OfficialStructure).id();
        let pop = app
            .world_mut()
            .spawn((PopAction {
                current: ActionType::Idle,
                ..Default::default()
            },))
            .id();

        app.update();

        // No StartPlan should be inserted
        assert!(app.world().get::<StartPlan>(pop).is_none());
    }

    #[test]
    fn test_vandalism_inverts_morale_aura() {
        let mut app = App::new();
        app.add_systems(Update, process_vandalism);

        let structure = app
            .world_mut()
            .spawn((OfficialStructure, MoraleAura { effect: 10.0 }))
            .id();

        let pop = app
            .world_mut()
            .spawn(StartPlan {
                target: Some(structure),
                action: ActionType::Vandalize,
            })
            .id();

        app.update();

        let aura = app.world().get::<MoraleAura>(structure).unwrap();
        assert!(
            aura.effect < 0.0,
            "Morale effect should be inverted after vandalism"
        );
        assert!(
            app.world().get::<Defaced>(structure).is_some(),
            "Structure should be marked as defaced"
        );

        // StartPlan should be removed
        assert!(app.world().get::<StartPlan>(pop).is_none());
    }
}

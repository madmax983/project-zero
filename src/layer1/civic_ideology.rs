use crate::layer1::needs::Needs;
use crate::layer1::utility_types::ActionType;
use bevy_ecs::prelude::*;

#[derive(Resource, Default)]
pub struct ActiveIdeology(pub IdeologyType);

#[derive(Default, PartialEq, Eq, Clone, Copy, Debug)]
pub enum IdeologyType {
    #[default]
    None,
    Survivalist,
    Profit,
    Knowledge,
}

#[derive(Component)]
pub struct RecentAction {
    pub action_type: ActionType,
    pub duration: f32,
}

pub fn apply_ideological_modifiers_system(
    ideology: Option<Res<ActiveIdeology>>,
    mut query: Query<(&mut Needs, &RecentAction)>,
) {
    let Some(active_ideology) = ideology else {
        return;
    };
    if active_ideology.0 == IdeologyType::None {
        return;
    }

    for (mut needs, action) in query.iter_mut() {
        match active_ideology.0 {
            IdeologyType::Survivalist => {
                if action.action_type == ActionType::SatisfyHunger {
                    needs.leisure += 0.05; // Bonus for aligned action
                } else if action.action_type == ActionType::Vandalize {
                    needs.leisure -= 0.05; // Penalty for waste
                }
            }
            IdeologyType::Profit => {
                if action.action_type == ActionType::Refine {
                    needs.leisure += 0.05;
                } else if action.action_type == ActionType::Idle {
                    needs.leisure -= 0.05;
                }
            }
            IdeologyType::Knowledge => {
                if action.action_type == ActionType::Research {
                    needs.leisure += 0.05;
                } else if action.action_type == ActionType::Binge {
                    needs.leisure -= 0.05;
                }
            }
            IdeologyType::None => {}
        }

        needs.leisure = needs.leisure.clamp(0.0, 1.0);
    }
}

pub fn decay_recent_action_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut RecentAction)>,
) {
    for (entity, mut action) in query.iter_mut() {
        action.duration -= 1.0;
        if action.duration <= 0.0 {
            commands.entity(entity).remove::<RecentAction>();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::App;

    #[test]
    fn test_civic_ideology_aligned_action_grants_morale() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(bevy::MinimalPlugins);
        app.init_resource::<ActiveIdeology>();
        app.add_systems(bevy_app::Update, apply_ideological_modifiers_system);

        app.world_mut()
            .insert_resource(ActiveIdeology(IdeologyType::Survivalist));

        let pop = app
            .world_mut()
            .spawn((
                Needs {
                    leisure: 0.5,
                    ..Default::default()
                },
                RecentAction {
                    action_type: ActionType::SatisfyHunger,
                    duration: 10.0,
                },
            ))
            .id();

        // Act
        app.update();

        // Assert
        let needs = app.world().get::<Needs>(pop).unwrap();
        assert!(
            needs.leisure > 0.5,
            "Leisure should increase when performing aligned actions"
        );
    }

    #[test]
    fn test_civic_ideology_opposed_action_reduces_morale() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(bevy::MinimalPlugins);
        app.init_resource::<ActiveIdeology>();
        app.add_systems(bevy_app::Update, apply_ideological_modifiers_system);

        app.world_mut()
            .insert_resource(ActiveIdeology(IdeologyType::Survivalist));

        let pop = app
            .world_mut()
            .spawn((
                Needs {
                    leisure: 0.5,
                    ..Default::default()
                },
                RecentAction {
                    action_type: ActionType::Vandalize,
                    duration: 10.0,
                },
            ))
            .id();

        // Act
        app.update();

        // Assert
        let needs = app.world().get::<Needs>(pop).unwrap();
        assert!(
            needs.leisure < 0.5,
            "Leisure should decrease when performing opposed actions"
        );
    }

    #[test]
    fn test_decay_recent_action_system() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(bevy::MinimalPlugins);
        app.add_systems(bevy_app::Update, decay_recent_action_system);

        let pop = app
            .world_mut()
            .spawn(RecentAction {
                action_type: ActionType::SatisfyHunger,
                duration: 1.5,
            })
            .id();

        // Act - Tick 1
        app.update();
        let action = app.world().get::<RecentAction>(pop).unwrap();
        assert!(
            (action.duration - 0.5).abs() < f32::EPSILON,
            "Duration should decrement by 1.0"
        );

        // Act - Tick 2
        app.update();
        assert!(
            app.world().get::<RecentAction>(pop).is_none(),
            "RecentAction should be removed when duration <= 0"
        );
    }
}

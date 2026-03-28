use bevy_ecs::prelude::*;

use crate::layer1::pop::Pop;
use crate::layer1::security::BiometricProfile;
use crate::layer1::unrest::Unrest;

/// Event fired when a pop initiates a ghost shift.
#[derive(Event, Debug, Clone)]
pub struct GhostShiftStartedEvent {
    pub entity: Entity,
}

/// Configuration for the ghost shift strike mechanic.
#[derive(Resource)]
pub struct GhostShiftConfig {
    /// Global unrest threshold required to start a ghost shift (0.0 to 1.0).
    pub unrest_threshold: f32,
    /// Minimum personal accumulated biometric drift required to evade direct detection.
    /// In this case, we use drift as the security threshold analog since the spec mentions
    /// high security preventing open riots, leading to subtle ghost shifts.
    /// Biometric drift represents high security pressure.
    pub security_drift_threshold: f32,
}

impl Default for GhostShiftConfig {
    fn default() -> Self {
        Self {
            unrest_threshold: 0.80,
            security_drift_threshold: 0.80,
        }
    }
}

/// The state of a pop engaging in a ghost shift strike.
/// While this is active, the pop consumes resources normally but `calculate_work_amount`
/// outputs 0.0 work per tick.
#[derive(Component)]
pub struct GhostShiftState {
    pub active: bool,
}

/// Evaluates if pops should enter a ghost shift strike.
///
/// A ghost shift occurs when global unrest is very high, but local security pressure
/// (represented by biometric drift monitoring) is also high, forcing the pop to pretend
/// to work instead of openly rioting.
#[allow(clippy::needless_pass_by_value)]
#[allow(clippy::type_complexity)]
pub fn evaluate_ghost_shifts(
    mut commands: Commands,
    unrest: Option<Res<Unrest>>,
    config: Option<Res<GhostShiftConfig>>,
    mut events: EventWriter<GhostShiftStartedEvent>,
    query: Query<(Entity, Option<&BiometricProfile>, Option<&GhostShiftState>), With<Pop>>,
) {
    let default_config = GhostShiftConfig::default();
    let config = config.as_deref().unwrap_or(&default_config);

    if let Some(unrest) = unrest {
        for (entity, profile_opt, ghost_state_opt) in query.iter() {
            // Determine security pressure from the profile
            let security_pressure = profile_opt.map_or(0.0, |p| p.drift);

            let should_strike = unrest.level > config.unrest_threshold
                && security_pressure > config.security_drift_threshold;

            if should_strike {
                if ghost_state_opt.is_none() {
                    commands
                        .entity(entity)
                        .insert(GhostShiftState { active: true });
                    events.send(GhostShiftStartedEvent { entity });
                }
            } else if ghost_state_opt.is_some() {
                // Conditions improved, end the strike
                commands.entity(entity).remove::<GhostShiftState>();
            }
        }
    } else {
        // No unrest tracking means everything should be fine, end strikes.
        for (entity, _, ghost_state_opt) in query.iter() {
            if ghost_state_opt.is_some() {
                commands.entity(entity).remove::<GhostShiftState>();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_ghost_shift_triggered_by_high_unrest_and_high_security() {
        let mut app = App::new();
        app.add_systems(Update, evaluate_ghost_shifts);
        app.add_event::<GhostShiftStartedEvent>();

        app.insert_resource(Unrest {
            level: 0.85,
            modifiers: vec![],
        });
        app.insert_resource(GhostShiftConfig::default());

        let pop = app
            .world_mut()
            .spawn((
                Pop,
                BiometricProfile {
                    drift: 0.90,
                    ..default()
                },
            ))
            .id();

        app.update();

        let pop_state = app.world().get::<GhostShiftState>(pop);
        assert!(pop_state.is_some(), "Pop should initiate a ghost shift");
        assert!(pop_state.unwrap().active);
    }

    #[test]
    fn test_ghost_shift_not_triggered_if_security_low() {
        let mut app = App::new();
        app.add_systems(Update, evaluate_ghost_shifts);
        app.add_event::<GhostShiftStartedEvent>();

        app.insert_resource(Unrest {
            level: 0.85,
            modifiers: vec![],
        });
        app.insert_resource(GhostShiftConfig::default());

        let pop = app
            .world_mut()
            .spawn((
                Pop,
                BiometricProfile {
                    drift: 0.20,
                    ..default()
                },
            ))
            .id();

        app.update();

        let pop_state = app.world().get::<GhostShiftState>(pop);
        assert!(pop_state.is_none());
    }

    #[test]
    fn test_ghost_shift_consumes_resources_but_produces_nothing() {
        let mut app = App::new();
        let pop_ghosting = app
            .world_mut()
            .spawn((Pop, GhostShiftState { active: true }))
            .id();

        let pop_working = app.world_mut().spawn((Pop,)).id();

        let amount_ghosting = crate::layer1::execution::calculate_work_amount(
            app.world(),
            pop_ghosting,
            crate::layer1::designation::DesignationType::Mine,
            None,
            0.5,
            1.0,
            1.0,
        );

        let amount_working = crate::layer1::execution::calculate_work_amount(
            app.world(),
            pop_working,
            crate::layer1::designation::DesignationType::Mine,
            None,
            0.5,
            1.0,
            1.0,
        );

        assert_eq!(
            amount_ghosting, 0.0,
            "Ghost shift should produce 0 work output."
        );
        assert!(
            amount_working > 0.0,
            "Normal shift should produce work output."
        );
    }
}

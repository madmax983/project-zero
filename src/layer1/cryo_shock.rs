//! Cryo-Shock system (Spec 413).
//!
//! Implements the Cryo-Shock status effect applied to Pops that spawn from cryo,
//! severely debuffing their movement and work speed. The effect naturally decays over time.

use bevy_ecs::prelude::*;

use crate::layer1::pop::{Pop, Speed};

/// Applied to Pops that have just emerged from cryo suspension.
/// Causes severe debuffs to movement speed and efficiency that decay over time.
#[derive(Component, Debug)]
pub struct CryoShock {
    /// Remaining duration of the shock in simulation ticks.
    pub duration_ticks: u64,
    /// Severity multiplier (0.0 to 1.0), where 1.0 reduces speed to 0.
    pub severity: f32,
}

impl Pop {
    /// Spawns a new Pop with the CryoShock component applied.
    #[must_use]
    pub fn from_cryo() -> (Self, CryoShock) {
        (
            Self,
            CryoShock {
                duration_ticks: 1000,
                severity: 0.5,
            },
        )
    }
}

/// Applies the movement speed debuff from the `CryoShock` component to Pops.
pub fn apply_cryo_debuff_system(mut query: Query<(&mut Speed, &CryoShock)>) {
    for (mut speed, shock) in query.iter_mut() {
        speed.current *= 1.0 - shock.severity;
    }
}

/// Decays the duration of the `CryoShock` effect and removes it once it expires.
pub fn decay_cryo_shock_system(mut commands: Commands, mut query: Query<(Entity, &mut CryoShock)>) {
    for (entity, mut shock) in query.iter_mut() {
        if shock.duration_ticks <= 1 {
            commands.entity(entity).remove::<CryoShock>();
        } else {
            shock.duration_ticks -= 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::{App, Update};

    #[test]
    fn test_cryo_shock_applied_on_spawn() {
        let mut app = App::new();

        let pop = app.world_mut().spawn(Pop::from_cryo()).id();
        app.update();

        assert!(
            app.world().get::<CryoShock>(pop).is_some(),
            "Pops spawned from cryo must have CryoShock"
        );
    }

    #[test]
    fn test_cryo_shock_applies_debuffs() {
        let mut app = App::new();

        let pop = app
            .world_mut()
            .spawn((
                Pop,
                Speed {
                    base: 10.0,
                    current: 10.0,
                    accumulator: 0.0,
                },
                CryoShock {
                    duration_ticks: 100,
                    severity: 0.5,
                },
            ))
            .id();

        app.add_systems(Update, apply_cryo_debuff_system);
        app.update();

        let speed = app.world().get::<Speed>(pop).unwrap().current;
        assert!(
            speed < 10.0,
            "Cryo shock must reduce effective movement speed"
        );
        assert_eq!(speed, 5.0, "Cryo shock severity 0.5 halves speed");
    }

    #[test]
    fn test_cryo_shock_duration_decays() {
        let mut app = App::new();

        let pop = app
            .world_mut()
            .spawn((
                Pop,
                CryoShock {
                    duration_ticks: 10,
                    severity: 0.5,
                },
            ))
            .id();

        app.add_systems(Update, decay_cryo_shock_system);

        app.update();

        let shock = app.world().get::<CryoShock>(pop).unwrap();
        assert!(
            shock.duration_ticks < 10,
            "Cryo shock duration must decay over time"
        );
        assert_eq!(shock.duration_ticks, 9);
    }

    #[test]
    fn test_cryo_shock_removed() {
        let mut app = App::new();

        let pop = app
            .world_mut()
            .spawn((
                Pop,
                CryoShock {
                    duration_ticks: 1,
                    severity: 0.5,
                },
            ))
            .id();

        app.add_systems(Update, decay_cryo_shock_system);

        app.update();

        assert!(
            app.world().get::<CryoShock>(pop).is_none(),
            "CryoShock should be removed when duration reaches 0"
        );
    }
}

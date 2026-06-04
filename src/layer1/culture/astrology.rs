//! Astrology System.
//!
//! This module implements astrological beliefs among the population.
//! Pops holding an `AstrologicalBelief` will have their productivity significantly
//! modified based on celestial alignments—receiving massive buffs during a
//! "lucky" alignment and debilitating debuffs during "unlucky" alignments.
//!
//! However, pops with the `Rationalist` trait are immune to these effects
//! and will maintain their baseline productivity.

use bevy_ecs::prelude::*;

/// Tracks whether the entity's sign is currently favored or cursed by the stars.
///
/// # Examples
/// ```rust
/// use scale::layer1::culture::astrology::AstrologicalBelief;
///
/// let belief = AstrologicalBelief {
///     lucky_alignment: true,
///     unlucky_alignment: false,
/// };
/// assert!(belief.lucky_alignment);
/// ```
#[derive(Component)]
pub struct AstrologicalBelief {
    pub lucky_alignment: bool,
    pub unlucky_alignment: bool,
}

/// A marker component for entities that reject astrology, granting immunity to its effects.
#[derive(Component)]
pub struct Rationalist;

/// Represents an entity's base productivity multiplier.
#[derive(Component)]
pub struct Productivity {
    pub multiplier: f32,
}

/// Applies massive productivity buffs or debuffs based on celestial alignments.
///
/// If an entity has a [`Rationalist`] component, they are entirely excluded from this system.
/// Otherwise, `Productivity::multiplier` is set to `1.5` during lucky alignments,
/// `0.5` during unlucky alignments, or reset to `1.0` during neutral periods.
///
/// # Examples
/// ```rust
/// use bevy_app::prelude::*;
/// use scale::layer1::culture::astrology::{AstrologicalBelief, Productivity, Rationalist, astrological_buff_system};
///
/// let mut app = App::new();
/// app.add_systems(Update, astrological_buff_system);
///
/// // A believer in a lucky alignment gets a buff.
/// let believer = app.world_mut().spawn((
///     AstrologicalBelief { lucky_alignment: true, unlucky_alignment: false },
///     Productivity { multiplier: 1.0 },
/// )).id();
///
/// // A rationalist is completely unaffected by the stars.
/// let skeptic = app.world_mut().spawn((
///     AstrologicalBelief { lucky_alignment: true, unlucky_alignment: false },
///     Productivity { multiplier: 1.0 },
///     Rationalist,
/// )).id();
///
/// app.update();
///
/// assert_eq!(app.world().get::<Productivity>(believer).unwrap().multiplier, 1.5);
/// assert_eq!(app.world().get::<Productivity>(skeptic).unwrap().multiplier, 1.0);
/// ```
pub fn astrological_buff_system(
    mut query: Query<(&AstrologicalBelief, &mut Productivity), Without<Rationalist>>,
) {
    for (belief, mut productivity) in query.iter_mut() {
        if belief.lucky_alignment {
            productivity.multiplier = 1.5; // Massive buff
        } else if belief.unlucky_alignment {
            productivity.multiplier = 0.5; // Massive debuff
        } else {
            productivity.multiplier = 1.0; // Normal
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::App;

    #[test]
    fn test_astrological_buff_applied_during_alignment() {
        let mut app = App::new();
        app.add_systems(bevy_app::Update, astrological_buff_system);

        let entity = app
            .world_mut()
            .spawn((
                AstrologicalBelief {
                    lucky_alignment: true,
                    unlucky_alignment: false,
                },
                Productivity { multiplier: 1.0 },
            ))
            .id();

        app.update();

        let productivity = app.world().get::<Productivity>(entity).unwrap();
        assert!(
            productivity.multiplier > 1.0,
            "Massive buff should be applied"
        );
    }

    #[test]
    fn test_astrological_debuff_applied_during_retrograde() {
        let mut app = App::new();
        app.add_systems(bevy_app::Update, astrological_buff_system);

        let entity = app
            .world_mut()
            .spawn((
                AstrologicalBelief {
                    lucky_alignment: false,
                    unlucky_alignment: true,
                },
                Productivity { multiplier: 1.0 },
            ))
            .id();

        app.update();

        let productivity = app.world().get::<Productivity>(entity).unwrap();
        assert!(
            productivity.multiplier < 1.0,
            "Massive debuff should be applied"
        );
    }

    #[test]
    fn test_rationalist_faction_ignores_astrology() {
        let mut app = App::new();
        app.add_systems(bevy_app::Update, astrological_buff_system);

        let entity = app
            .world_mut()
            .spawn((
                AstrologicalBelief {
                    lucky_alignment: true, // Should trigger buff, but ignored by Rationalist
                    unlucky_alignment: false,
                },
                Rationalist,
                Productivity { multiplier: 1.0 },
            ))
            .id();

        app.update();

        let productivity = app.world().get::<Productivity>(entity).unwrap();
        assert_eq!(
            productivity.multiplier, 1.0,
            "Rationalists should ignore astrology"
        );
    }

    #[test]
    fn test_astrological_normal() {
        let mut app = App::new();
        app.add_systems(bevy_app::Update, astrological_buff_system);

        let entity = app
            .world_mut()
            .spawn((
                AstrologicalBelief {
                    lucky_alignment: false,
                    unlucky_alignment: false,
                },
                Productivity { multiplier: 2.0 }, // Initial value different from normal
            ))
            .id();

        app.update();

        let productivity = app.world().get::<Productivity>(entity).unwrap();
        assert_eq!(
            productivity.multiplier, 1.0,
            "Multiplier should reset to normal 1.0"
        );
    }
}

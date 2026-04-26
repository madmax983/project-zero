//! Zodiac Talisman (Nova Feature)
//!
//! # The Spark
//! We have `AstrologicalBelief` which grants buffs/debuffs based on planetary alignments.
//! We also have a rich item system. What if there was a special "Zodiac Talisman" item?
//!
//! # The Feature
//! When a Pop with an `AstrologicalBelief` is equipped with a `ZodiacTalisman` (represented by a component),
//! the talisman *inverts* their current astrological effect, turning a retrograde (debuff) into an alignment (buff),
//! and vice-versa, acting as a bizarre cosmic battery.

use crate::layer1::religion::astrological_beliefs::{
    AstrologicalBelief, Productivity, Rationalist,
};
use bevy_ecs::prelude::*;

/// Component indicating a Pop is holding or equipped with a Zodiac Talisman.
#[derive(Component)]
pub struct ZodiacTalisman;

/// System that runs after `astrological_buff_system` to invert the effects if a Pop has a Zodiac Talisman.
pub fn zodiac_talisman_system(
    mut query: Query<
        (&AstrologicalBelief, &mut Productivity, &ZodiacTalisman),
        Without<Rationalist>,
    >,
) {
    for (belief, mut productivity, _) in query.iter_mut() {
        if belief.lucky_alignment {
            // The talisman absorbs the good luck
            productivity.multiplier = 0.5;
        } else if belief.unlucky_alignment {
            // The talisman repels the bad luck, inverting it to a buff
            productivity.multiplier = 1.5;
        } else {
            // Latent cosmic energy provides a slight static buzz
            productivity.multiplier = 1.1;
        }
    }
}

pub fn register(schedule: &mut Schedule) {
    schedule.add_systems(zodiac_talisman_system);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_talisman_inverts_lucky_alignment() {
        let mut app = bevy_app::App::new();
        // Register the base system then our experimental one
        app.add_systems(
            bevy_app::Update,
            (
                crate::layer1::religion::astrological_beliefs::astrological_buff_system,
                zodiac_talisman_system,
            )
                .chain(),
        );

        let entity = app
            .world_mut()
            .spawn((
                AstrologicalBelief {
                    lucky_alignment: true,
                    unlucky_alignment: false,
                },
                Productivity { multiplier: 1.0 },
                ZodiacTalisman,
            ))
            .id();

        app.update();

        let productivity = app.world().get::<Productivity>(entity).unwrap();
        // The base system sets it to 1.5, the talisman system overrides it to 0.5
        assert_eq!(productivity.multiplier, 0.5);
    }

    #[test]
    fn test_talisman_inverts_unlucky_alignment() {
        let mut app = bevy_app::App::new();
        app.add_systems(
            bevy_app::Update,
            (
                crate::layer1::religion::astrological_beliefs::astrological_buff_system,
                zodiac_talisman_system,
            )
                .chain(),
        );

        let entity = app
            .world_mut()
            .spawn((
                AstrologicalBelief {
                    lucky_alignment: false,
                    unlucky_alignment: true,
                },
                Productivity { multiplier: 1.0 },
                ZodiacTalisman,
            ))
            .id();

        app.update();

        let productivity = app.world().get::<Productivity>(entity).unwrap();
        // The base system sets it to 0.5, the talisman system overrides it to 1.5
        assert_eq!(productivity.multiplier, 1.5);
    }

    #[test]
    fn test_talisman_provides_latent_energy_normally() {
        let mut app = bevy_app::App::new();
        app.add_systems(
            bevy_app::Update,
            (
                crate::layer1::religion::astrological_beliefs::astrological_buff_system,
                zodiac_talisman_system,
            )
                .chain(),
        );

        let entity = app
            .world_mut()
            .spawn((
                AstrologicalBelief {
                    lucky_alignment: false,
                    unlucky_alignment: false,
                },
                Productivity { multiplier: 1.0 },
                ZodiacTalisman,
            ))
            .id();

        app.update();

        let productivity = app.world().get::<Productivity>(entity).unwrap();
        // Base sets to 1.0, talisman sets to 1.1
        assert_eq!(productivity.multiplier, 1.1);
    }
}

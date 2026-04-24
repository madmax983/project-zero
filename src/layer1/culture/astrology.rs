use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct AstrologicalBelief {
    pub lucky_alignment: bool,
    pub unlucky_alignment: bool,
}

#[derive(Component)]
pub struct Rationalist;

#[derive(Component)]
pub struct Productivity {
    pub multiplier: f32,
}

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

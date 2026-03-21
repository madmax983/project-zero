use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct GalacticCouncil {
    pub active_resolutions: Vec<Resolution>,
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum Resolution {
    BanStripMining,
    UniversalRights,
}

#[derive(Component)]
pub struct CouncilMember {
    pub in_breach: bool,
}

#[derive(Component)]
pub struct TradeSanctions {
    pub multiplier: f32, // 1.0 is normal, 0.5 is 50% penalty
}

pub fn enforce_resolutions_system(
    council: Res<GalacticCouncil>,
    mut commands: Commands,
    mut members: Query<(Entity, &CouncilMember, Option<&mut TradeSanctions>)>,
) {
    for (entity, member, sanctions_opt) in members.iter_mut() {
        if member.in_breach && !council.active_resolutions.is_empty() {
            // Apply severe sanctions (50% trade penalty)
            if let Some(mut sanctions) = sanctions_opt {
                sanctions.multiplier = 0.5;
            } else {
                commands
                    .entity(entity)
                    .insert(TradeSanctions { multiplier: 0.5 });
            }
        } else {
            // Remove sanctions if compliant
            if sanctions_opt.is_some() {
                commands.entity(entity).remove::<TradeSanctions>();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_in_breach_member_gets_trade_sanctions() {
        let mut app = App::new();
        app.insert_resource(GalacticCouncil {
            active_resolutions: vec![Resolution::BanStripMining],
        });

        let violator = app
            .world_mut()
            .spawn(CouncilMember { in_breach: true })
            .id();

        app.add_systems(Update, enforce_resolutions_system);
        app.update();

        let sanctions = app.world().get::<TradeSanctions>(violator);
        assert!(
            sanctions.is_some(),
            "Violating member should receive TradeSanctions"
        );
        assert_eq!(sanctions.unwrap().multiplier, 0.5);
    }

    #[test]
    fn test_compliant_member_no_sanctions() {
        let mut app = App::new();
        app.insert_resource(GalacticCouncil {
            active_resolutions: vec![Resolution::BanStripMining],
        });

        let compliant = app
            .world_mut()
            .spawn(CouncilMember { in_breach: false })
            .id();
        let formerly_breaching = app
            .world_mut()
            .spawn((
                CouncilMember { in_breach: false }, // Fixed their breach
                TradeSanctions { multiplier: 0.5 },
            ))
            .id();

        app.add_systems(Update, enforce_resolutions_system);
        app.update();

        assert!(
            app.world().get::<TradeSanctions>(compliant).is_none(),
            "Compliant member should not get sanctions"
        );
        assert!(
            app.world()
                .get::<TradeSanctions>(formerly_breaching)
                .is_none(),
            "Sanctions should be removed after compliance"
        );
    }

    #[test]
    fn test_in_breach_member_already_has_sanctions() {
        let mut app = App::new();
        app.insert_resource(GalacticCouncil {
            active_resolutions: vec![Resolution::BanStripMining],
        });

        let violator = app
            .world_mut()
            .spawn((
                CouncilMember { in_breach: true },
                TradeSanctions { multiplier: 1.0 }, // Has sanctions, but wrong multiplier
            ))
            .id();

        app.add_systems(Update, enforce_resolutions_system);
        app.update();

        let sanctions = app.world().get::<TradeSanctions>(violator);
        assert!(
            sanctions.is_some(),
            "Violating member should keep TradeSanctions"
        );
        assert_eq!(
            sanctions.unwrap().multiplier,
            0.5,
            "Multiplier should be updated to 0.5"
        );
    }

    #[test]
    fn test_in_breach_member_no_active_resolutions() {
        let mut app = App::new();
        app.insert_resource(GalacticCouncil {
            active_resolutions: vec![], // No active resolutions
        });

        let violator = app
            .world_mut()
            .spawn(CouncilMember { in_breach: true })
            .id();

        app.add_systems(Update, enforce_resolutions_system);
        app.update();

        let sanctions = app.world().get::<TradeSanctions>(violator);
        assert!(
            sanctions.is_none(),
            "Violating member should not receive sanctions if no active resolutions"
        );
    }
}

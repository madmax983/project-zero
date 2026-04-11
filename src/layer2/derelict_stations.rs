use crate::layer2::fleet::{Fleet, FleetOrder, InOrbit};
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct DerelictStation {
    pub claimed: bool,
}

#[derive(Component)]
pub struct HiddenQuirk(pub crate::layer1::social::rituals::QuirkType);

pub fn claim_station_system(
    mut commands: Commands,
    mut fleets: Query<(Entity, &FleetOrder, &InOrbit), With<Fleet>>,
    mut derelicts: Query<&mut DerelictStation>,
) {
    for (fleet_entity, order, in_orbit) in fleets.iter_mut() {
        if let FleetOrder::ClaimStation(target_station) = *order {
            if in_orbit.parent == target_station {
                if let Ok(mut derelict) = derelicts.get_mut(target_station) {
                    derelict.claimed = true;
                    // Claiming removes the order
                    commands.entity(fleet_entity).remove::<FleetOrder>();
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::architecture::structure::Structure;
    use crate::layer1::social::rituals::{Quirk, QuirkType};

    #[test]
    fn test_derelict_station_claiming() {
        let mut app = bevy::app::App::new();
        app.add_systems(bevy_app::Update, claim_station_system);

        let station = app
            .world_mut()
            .spawn(DerelictStation { claimed: false })
            .id();

        let fleet = app
            .world_mut()
            .spawn((
                Fleet,
                InOrbit { parent: station },
                FleetOrder::ClaimStation(station),
            ))
            .id();

        app.update();

        let derelict = app.world().get::<DerelictStation>(station).unwrap();
        assert!(derelict.claimed, "Station should be claimed");

        let order = app.world().get::<FleetOrder>(fleet);
        assert!(order.is_none(), "Claim order should be removed");
    }

    #[test]
    fn test_derelict_quirk_activation_on_repair() {
        let mut app = bevy::app::App::new();

        let structure_entity = app
            .world_mut()
            .spawn((
                Structure {
                    current_hp: 10.0,
                    max_hp: 100.0,
                },
                crate::layer1::GridPosition { x: 0, y: 0 },
                HiddenQuirk(QuirkType::GasLeak),
            ))
            .id();

        // Perform repair action
        crate::layer1::architecture::structure::process_repair(
            app.world_mut(),
            structure_entity,
            50.0,
        );

        let has_hidden = app.world().get::<HiddenQuirk>(structure_entity).is_some();
        let quirk = app.world().get::<Quirk>(structure_entity);

        assert!(!has_hidden, "HiddenQuirk should be removed upon repair");
        assert!(quirk.is_some(), "Quirk should be added upon repair");
        assert_eq!(quirk.unwrap().quirk_type, QuirkType::GasLeak);
    }
}

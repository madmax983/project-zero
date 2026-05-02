use bevy::prelude::*;

use crate::layer1::mind::utility_types::UtilityWeights;
use crate::layer1::architecture::building::{Building, BuildingType};
use crate::layer1::tech::machine_awakening::Bot;

// Redefining the struct according to the spec, as the existing one in `grafting` is different
#[derive(Component, Default)]
pub struct MaintenanceDebt {
    pub current: f32,
    pub threshold: f32,
}

#[derive(Component)]
pub struct MachineCultMember {
    pub shrine_entity: Entity,
}

#[allow(clippy::type_complexity)]
pub fn rogue_cult_formation_system(
    mut commands: Commands,
    bots_query: Query<(Entity, &MaintenanceDebt), (With<Bot>, Without<MachineCultMember>)>,
    shrines_query: Query<(Entity, &Building, &MaintenanceDebt)>,
) {
    // Find a potential shrine (ServerBank or CommandCenter acting as Mainframe/CommsRelay) with high debt
    let potential_shrine = shrines_query.iter().find(|(_, building, debt)| {
        (building.building_type == BuildingType::ServerBank || building.building_type == BuildingType::CommandCenter)
            && debt.current >= debt.threshold
    });

    if let Some((shrine_entity, _, _)) = potential_shrine {
        for (bot_entity, debt) in bots_query.iter() {
            if debt.current >= debt.threshold {
                commands.entity(bot_entity).insert(MachineCultMember {
                    shrine_entity,
                });
            }
        }
    }
}

pub fn cult_priority_override_system(
    mut query: Query<(&MachineCultMember, &mut UtilityWeights)>,
) {
    for (_, mut weights) in query.iter_mut() {
        weights.distance_weight = 0.1;
        weights.availability_weight = 0.9;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::layer1::mind::utility_types::UtilityWeights;
    use crate::layer1::architecture::building::{Building, BuildingType};
    use crate::layer1::tech::machine_awakening::Bot;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_systems(Update, (
            rogue_cult_formation_system,
            cult_priority_override_system,
        ));
        app
    }

    #[test]
    fn test_high_maintenance_debt_triggers_cult_formation() {
        let mut app = setup_app();

        let server_node = app.world_mut().spawn((
            Building { building_type: BuildingType::ServerBank },
            MaintenanceDebt { current: 150.0, threshold: 100.0 }
        )).id();

        let bot = app.world_mut().spawn((
            Bot,
            MaintenanceDebt { current: 120.0, threshold: 100.0 }
        )).id();

        app.update();

        // Assert bot joined a cult targeting the server node
        let cult_member = app.world().get::<MachineCultMember>(bot);
        assert!(cult_member.is_some());
        assert_eq!(cult_member.unwrap().shrine_entity, server_node);
    }

    #[test]
    fn test_cult_membership_overrides_standard_utility_weights() {
        let mut app = setup_app();

        let shrine = app.world_mut().spawn((
            Building { building_type: BuildingType::CommandCenter },
        )).id();

        let bot = app.world_mut().spawn((
            Bot,
            MachineCultMember { shrine_entity: shrine },
            UtilityWeights { distance_weight: 1.0, availability_weight: 1.0 }
        )).id();

        app.update();

        let weights = app.world().get::<UtilityWeights>(bot).unwrap();

        assert!(weights.distance_weight < 0.5, "Standard work weight should be suppressed");
        assert!(weights.availability_weight > 0.8, "Defend weight should spike to protect the shrine");
    }
}

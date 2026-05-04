use bevy_ecs::prelude::*;
use crate::layer1::mind::utility_types::UtilityWeights;
use crate::layer1::biology::grafting::MaintenanceDebt;
use crate::layer1::architecture::building::BuildingType;
use crate::layer1::tech::machine_awakening::Bot;

#[derive(Component)]
pub struct MachineCultMember {
    pub shrine_entity: Entity,
}

pub fn rogue_cult_formation_system(
    mut commands: Commands,
    bots_query: Query<(Entity, &MaintenanceDebt), (With<Bot>, Without<MachineCultMember>)>,
    shrines_query: Query<(Entity, &BuildingType, &MaintenanceDebt)>,
) {
    // Find a potential shrine (ServerBank or MediaStation with high debt)
    let potential_shrine = shrines_query.iter().find(|(_, b_type, debt)| {
        (*b_type == &BuildingType::ServerBank || *b_type == &BuildingType::MediaStation)
        && debt.amount >= 100.0
    });

    if let Some((shrine_entity, _, _)) = potential_shrine {
        for (bot_entity, debt) in bots_query.iter() {
            if debt.amount >= 100.0 {
                commands.entity(bot_entity).insert(MachineCultMember {
                    shrine_entity,
                });
            }
        }
    }
}

pub fn cult_priority_override_system(
    mut query: Query<(&MachineCultMember, &mut UtilityWeights)>
) {
    for (_, mut weights) in query.iter_mut() {
        // Override normal behavior to focus on worship/defense
        weights.work_weight = 0.1;
        weights.defend_weight = 0.9;
        // Cultists might also get a custom 'Worship' action weight here
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::{App, Update};

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
            BuildingType::ServerBank,
            MaintenanceDebt { amount: 150.0 }
        )).id();

        let bot = app.world_mut().spawn((
            Bot,
            MaintenanceDebt { amount: 120.0 }
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

        let shrine = app.world_mut().spawn(BuildingType::MediaStation).id();

        let bot = app.world_mut().spawn((
            Bot,
            MachineCultMember { shrine_entity: shrine },
            UtilityWeights { work_weight: 1.0, defend_weight: 0.0, ..Default::default() }
        )).id();

        app.update();

        let weights = app.world().get::<UtilityWeights>(bot).unwrap();
        // Standard work priority should drop, while defense of shrine increases
        assert!(weights.work_weight < 0.5, "Standard work weight should be suppressed");
        assert!(weights.defend_weight > 0.8, "Defend weight should spike to protect the shrine");
    }
}

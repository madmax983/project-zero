use crate::layer1::social::factions::{
    FactionDemand, FactionId, FactionMember, FactionState, Factions,
};
use bevy_ecs::prelude::*;

#[derive(Resource, Default)]
pub struct AtrocityScore {
    pub score: f32,
}

pub fn check_penitent_faction_formation_system(
    mut atrocity_score: ResMut<AtrocityScore>,
    mut pops: Query<&mut FactionMember, With<crate::layer1::pop::Pop>>,
    mut factions: ResMut<Factions>,
) {
    if atrocity_score.score >= 100.0 {
        atrocity_score.score = 0.0;
        // Convert some random pops
        for mut faction in pops.iter_mut() {
            if faction.faction_id == Some(FactionId::Unaligned) && rand::random::<f32>() < 0.1 {
                faction.faction_id = Some(FactionId::Penitent);
            }
        }

        // Ensure the Penitent faction data exists and has a demand
        if let Some(data) = factions.map.get_mut(&FactionId::Penitent) {
            if data.active_demand.is_none() {
                data.active_demand = Some(FactionDemand {
                    policy: None,
                    remaining_time: 2000.0,
                    description: "Reparations".to_string(),
                });
                data.state = FactionState::Unhappy;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::needs::Needs;
    use crate::layer1::pop::Pop;
    use crate::layer1::social::factions::{
        FactionDemand, FactionId, FactionMember, FactionState, Factions,
    };

    fn setup_world() -> World {
        let mut world = World::new();
        world.init_resource::<AtrocityScore>();

        let mut factions = Factions::default();
        factions.initialize();
        // Manually insert Penitent for test since initialize might not include it
        factions.map.insert(
            FactionId::Penitent,
            crate::layer1::social::factions::FactionData {
                name: "The Penitent".into(),
                ..Default::default()
            },
        );
        world.insert_resource(factions);
        world
    }

    #[test]
    fn test_penitent_faction_forms_on_high_atrocity() {
        let mut world = setup_world();

        let pop = world
            .spawn((
                Pop,
                FactionMember {
                    faction_id: Some(FactionId::Unaligned),
                },
                Needs {
                    ..Default::default()
                },
            ))
            .id();

        let mut atrocity = world.resource_mut::<AtrocityScore>();
        atrocity.score = 100.0;

        let mut schedule = Schedule::default();
        schedule.add_systems(check_penitent_faction_formation_system);
        // We need to loop since it's a 10% chance
        for _ in 0..100 {
            world.resource_mut::<AtrocityScore>().score = 100.0;
            schedule.run(&mut world);
            let faction = world.get::<FactionMember>(pop).unwrap();
            if faction.faction_id == Some(FactionId::Penitent) {
                break;
            }
        }

        let faction = world.get::<FactionMember>(pop).unwrap();
        assert_eq!(faction.faction_id, Some(FactionId::Penitent));
    }

    #[test]
    fn test_penitent_faction_strikes_if_ignored() {
        let mut world = setup_world();

        let mut factions = world.resource_mut::<Factions>();
        factions.map.insert(
            FactionId::Penitent,
            crate::layer1::social::factions::FactionData {
                active_demand: Some(FactionDemand {
                    remaining_time: 1.0,
                    policy: None,
                    description: "Reparations".to_string(),
                }),
                state: FactionState::Unhappy,
                ..Default::default()
            },
        );

        let mut schedule = Schedule::default();
        schedule.add_systems(crate::layer1::social::factions::update_faction_strikes_system);
        schedule.run(&mut world);

        let factions = world.resource::<Factions>();
        let penitent_data = factions.get(FactionId::Penitent).unwrap();
        assert_eq!(penitent_data.state, FactionState::Striking);
    }
}

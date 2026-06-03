use bevy_ecs::prelude::*;
// src/layer1/social/bureau_of_regrets.rs

use crate::layer1::social::factions::{FactionId, FactionMember};
use crate::layer1::pop::Pop;
use crate::layer1::core::chronicle::AtrocityScore;

/// Represents the demands of the Penitent faction to dismantle structures or technologies.
#[derive(Component)]
pub struct ReparationDemands {
    /// Number of ticks the demand has been ignored.
    pub ignored_ticks: u32,
    /// Whether the pop is currently on strike.
    pub is_striking: bool,
}

/// Checks if the AtrocityScore is high enough to form the Penitent faction.
/// Converts random unaligned pops to the Penitent faction.
pub fn check_penitent_faction_formation_system(
    atrocity_score: Res<AtrocityScore>,
    mut pops: Query<(Entity, &mut FactionMember), With<Pop>>,
    mut commands: Commands,
) {
    if atrocity_score.score >= 100.0 {
        // Convert some random pops or unhappy pops
        for (entity, mut faction) in pops.iter_mut() {
            if faction.faction_id == Some(FactionId::Unaligned) && rand::random::<f32>() < 0.1 {
                faction.faction_id = Some(FactionId::Penitent);
                commands.entity(entity).insert(ReparationDemands {
                    ignored_ticks: 0,
                    is_striking: false,
                });
            }
        }
    }
}

/// Processes the reparation demands, incrementing ignored ticks and triggering strikes if ignored for too long.
pub fn process_reparation_strikes_system(
    mut demands: Query<&mut ReparationDemands>,
) {
    for mut demand in demands.iter_mut() {
        demand.ignored_ticks += 1;
        if demand.ignored_ticks >= 100 {
            demand.is_striking = true;
            // Additional logic to halt work or sabotage
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::layer1::social::factions::{FactionId, FactionMember};
    use crate::layer1::pop::Pop;
    use crate::layer1::morale::Morale;
    use bevy_ecs::system::RunSystemOnce;

    fn setup_world() -> World {
        let mut world = World::new();
        world.init_resource::<AtrocityScore>();
        world
    }

    #[test]
    fn test_penitent_faction_forms_on_high_atrocity() {
        let mut world = setup_world();

        let pop = world.spawn((
            Pop,
            FactionMember { faction_id: Some(FactionId::Unaligned) },
            Morale { value: 0.5, modifiers: vec![] },
        )).id();

        let mut atrocity = world.resource_mut::<AtrocityScore>();
        atrocity.score = 100.0; // High score

        // Loop enough times to practically guarantee the 10% chance hits
        for _ in 0..100 {
            world.run_system_once(check_penitent_faction_formation_system).unwrap();
        }

        let faction = world.get::<FactionMember>(pop).unwrap();
        assert_eq!(faction.faction_id, Some(FactionId::Penitent)); // Pop converted
    }

    #[test]
    fn test_penitent_faction_strikes_if_ignored() {
        let mut world = setup_world();

        let pop = world.spawn((
            Pop,
            FactionMember { faction_id: Some(FactionId::Penitent) },
            ReparationDemands { ignored_ticks: 100, is_striking: false },
        )).id();

        world.run_system_once(process_reparation_strikes_system).unwrap();

        let demands = world.get::<ReparationDemands>(pop).unwrap();
        assert!(demands.is_striking); // Strike initiated
    }
}

use bevy_ecs::prelude::*;
use crate::layer1::social::factions::{FactionId, FactionMember};
use crate::layer1::entities::pop::Pop;
use crate::layer1::core::chronicle::AtrocityScore;

#[derive(Component)]
pub struct ReparationDemands {
    pub ignored_ticks: u32,
    pub is_striking: bool,
}

pub fn check_penitent_faction_formation_system(
    mut commands: Commands,
    atrocity_score: Res<AtrocityScore>,
    mut pops: Query<(Entity, &mut FactionMember), With<Pop>>,
) {
    if atrocity_score.score >= 100.0 {
        for (entity, mut faction) in pops.iter_mut() {
            if faction.faction_id != Some(FactionId::Penitent) {
                faction.faction_id = Some(FactionId::Penitent);
                commands.entity(entity).insert(ReparationDemands {
                    ignored_ticks: 0,
                    is_striking: false,
                });
            }
        }
    }
}

pub fn process_reparation_strikes_system(
    mut demands: Query<&mut ReparationDemands>,
) {
    for mut demand in demands.iter_mut() {
        demand.ignored_ticks += 1;
        if demand.ignored_ticks >= 100 {
            demand.is_striking = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_world() -> World {
        let mut world = World::new();
        world.init_resource::<AtrocityScore>();
        world
    }

    #[test]
    fn test_penitent_faction_forms_on_high_atrocity() {
        let mut world = setup_world();

        let pop = world.spawn((
            Pop::default(),
            FactionMember { faction_id: Some(FactionId::Unaligned) },
        )).id();

        let mut atrocity = world.resource_mut::<AtrocityScore>();
        atrocity.score = 100.0;

        let mut schedule = Schedule::default();
        schedule.add_systems(check_penitent_faction_formation_system);
        schedule.run(&mut world);

        let faction = world.get::<FactionMember>(pop).unwrap();
        assert_eq!(faction.faction_id, Some(FactionId::Penitent));
        assert!(world.get::<ReparationDemands>(pop).is_some());
    }

    #[test]
    fn test_penitent_faction_strikes_if_ignored() {
        let mut world = setup_world();

        let pop = world.spawn((
            Pop::default(),
            FactionMember { faction_id: Some(FactionId::Penitent) },
            ReparationDemands { ignored_ticks: 100, is_striking: false },
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(process_reparation_strikes_system);
        schedule.run(&mut world);

        let demands = world.get::<ReparationDemands>(pop).unwrap();
        assert!(demands.is_striking);
    }
}

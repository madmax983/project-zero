use crate::layer1::architecture::building::{Building, BuildingType};
use crate::layer1::core::map::GridPosition;
use crate::layer1::entities::pop::Pop;
use crate::layer1::social::{AffinityChange, Relationships};
use bevy_ecs::prelude::*;

const GOSSIP_RADIUS: u32 = 3;
const AFFINITY_BOOST: f32 = 0.5;

type PopQueryFilters = (With<Pop>, With<Relationships>);

#[allow(clippy::needless_range_loop)]
pub fn water_cooler_gossip_system(
    pops: Query<(Entity, &GridPosition), PopQueryFilters>,
    wells: Query<(&GridPosition, &Building)>,
    mut affinity_events: EventWriter<AffinityChange>,
) {
    let mut well_positions = Vec::new();
    for (pos, building) in wells.iter() {
        if building.building_type == BuildingType::Well {
            well_positions.push(*pos);
        }
    }

    if well_positions.is_empty() {
        return;
    }

    let pops_list: Vec<(Entity, GridPosition)> = pops.iter().map(|(e, pos)| (e, *pos)).collect();

    for i in 0..pops_list.len() {
        let (pop1_entity, pop1_pos) = pops_list[i];

        let mut is_near_well = false;
        for well_pos in &well_positions {
            if pop1_pos.distance_chebyshev(*well_pos) <= GOSSIP_RADIUS {
                is_near_well = true;
                break;
            }
        }

        if !is_near_well {
            continue;
        }

        for j in (i + 1)..pops_list.len() {
            let (pop2_entity, pop2_pos) = pops_list[j];

            if pop1_pos.distance_chebyshev(pop2_pos) <= GOSSIP_RADIUS {
                affinity_events.send(AffinityChange {
                    source: pop1_entity,
                    target: pop2_entity,
                    amount: AFFINITY_BOOST,
                });
                affinity_events.send(AffinityChange {
                    source: pop2_entity,
                    target: pop1_entity,
                    amount: AFFINITY_BOOST,
                });
            }
        }
    }
}

pub fn register(schedule: &mut Schedule) {
    schedule.add_systems(water_cooler_gossip_system);
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_water_cooler_gossip() {
        let mut world = World::new();
        world.insert_resource(Events::<AffinityChange>::default());

        world.spawn((
            GridPosition { x: 5, y: 5 },
            Building {
                building_type: BuildingType::Well,
            },
        ));

        let _pop1 = world
            .spawn((
                Pop,
                GridPosition { x: 6, y: 6 }, // Within radius 3
                Relationships::default(),
            ))
            .id();

        let _pop2 = world
            .spawn((
                Pop,
                GridPosition { x: 6, y: 7 }, // Within radius 3 of pop1 and well
                Relationships::default(),
            ))
            .id();

        world.run_system_once(water_cooler_gossip_system).unwrap();

        let events = world.resource::<Events<AffinityChange>>();
        let mut reader = events.get_cursor();
        let emitted: Vec<_> = reader.read(events).collect();

        assert_eq!(emitted.len(), 2);
        assert_eq!(emitted[0].amount, AFFINITY_BOOST);
    }
}

use crate::layer1::actions::AssignedTo;
use crate::layer1::actions::AssignmentType;
use crate::layer1::needs::Needs;
use crate::layer1::pop::Pop;
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct VoidSleep;

#[derive(Component)]
pub struct ZeroGSuspensionPod;

#[derive(Component)]
pub struct GravityNightmare;

/// Pops with VoidSleep sleeping in normal beds suffer nightmares and don't recover rest properly.
pub fn gravity_nightmares_system(
    mut commands: Commands,
    mut sleep_query: Query<(Entity, &mut Needs, &AssignedTo, Option<&VoidSleep>), With<Pop>>,
    pod_query: Query<&ZeroGSuspensionPod>,
    zone_grid: Option<Res<crate::layer1::zone::ZoneGrid>>,
    noise_map: Option<Res<crate::layer1::acoustic::NoiseMap>>,
    housing_query: Query<(
        &crate::layer1::building::Building,
        &crate::layer1::GridPosition,
    )>,
) {
    for (entity, mut needs, assigned, void_sleep) in sleep_query.iter_mut() {
        if assigned.assignment_type == AssignmentType::HousingResident {
            let mut in_pod = false;
            let mut zone_bonus = 0.0;
            let mut noise_penalty = 0.0;

            if pod_query.get(assigned.entity).is_ok() {
                in_pod = true;
            }

            if void_sleep.is_some() && !in_pod {
                // If they are in a normal bed, subtract the rest they gained (mostly)
                // restore_rest_in_housing_system gave them:
                // REST_RESTORE_PER_TICK * (1.0 + zone_bonus) * (1.0 - noise_penalty)

                if let Ok((building, pos)) = housing_query.get(assigned.entity) {
                    zone_bonus = zone_grid.as_ref().map_or(0.0, |grid| {
                        let zone = grid.get(pos.x, pos.y);
                        crate::layer1::zone::calculate_zone_bonus(zone, building.building_type)
                    });

                    noise_penalty = noise_map
                        .as_ref()
                        .map_or(0.0, |map| map.get(pos.x, pos.y) * 0.5);
                }

                // Normal amount recovered is ~0.05
                // We subtract most of it to make recovery minimal, let's say they recover only 5% of normal
                let normal_recovery = 0.05 * (1.0 + zone_bonus) * (1.0 - noise_penalty);
                let nightmare_penalty = normal_recovery * 0.95;

                // Ensure we don't drop below 0 if rest was already low
                needs.rest = (needs.rest - nightmare_penalty).max(0.0);
                commands.entity(entity).insert(GravityNightmare);
            } else {
                // They are normal or in a suspension pod
                commands.entity(entity).remove::<GravityNightmare>();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::housing::Housing;
    use crate::layer1::GridPosition;
    use bevy_ecs::system::RunSystemOnce;

    fn setup_app() -> World {
        let world = World::new();
        world
    }

    #[test]
    fn test_void_sleep_pop_in_regular_bed() {
        let mut world = setup_app();

        let house = world
            .spawn((
                Housing {
                    capacity: 1,
                    residents: vec![],
                },
                Building {
                    building_type: BuildingType::Housing,
                },
                GridPosition { x: 0, y: 0 },
            ))
            .id();

        let pop = world
            .spawn((
                Pop,
                Needs {
                    hunger: 1.0,
                    rest: 0.1,
                    leisure: 1.0,
                    hygiene: 1.0,
                },
                VoidSleep,
                AssignedTo {
                    entity: house,
                    assignment_type: AssignmentType::HousingResident,
                },
            ))
            .id();
        world.get_mut::<Housing>(house).unwrap().residents.push(pop);

        // Run rest system
        world
            .run_system_once(crate::layer1::housing::restore_rest_in_housing_system)
            .unwrap();
        // Run nightmares system
        world.run_system_once(gravity_nightmares_system).unwrap();

        // Assert: Rest should not recover significantly, and should have a nightmare modifier
        let rest = world.get::<Needs>(pop).unwrap().rest;
        assert!(
            rest < 0.12,
            "Rest recovered too much in regular bed: {}",
            rest
        );
        assert!(world.get::<GravityNightmare>(pop).is_some());
    }

    #[test]
    fn test_void_sleep_pop_in_suspension_pod() {
        let mut world = setup_app();

        let pod = world
            .spawn((
                Housing {
                    capacity: 1,
                    residents: vec![],
                },
                Building {
                    building_type: BuildingType::Housing,
                },
                GridPosition { x: 0, y: 0 },
                ZeroGSuspensionPod,
            ))
            .id();

        let pop = world
            .spawn((
                Pop,
                Needs {
                    hunger: 1.0,
                    rest: 0.1,
                    leisure: 1.0,
                    hygiene: 1.0,
                },
                VoidSleep,
                AssignedTo {
                    entity: pod,
                    assignment_type: AssignmentType::HousingResident,
                },
            ))
            .id();
        world.get_mut::<Housing>(pod).unwrap().residents.push(pop);

        world
            .run_system_once(crate::layer1::housing::restore_rest_in_housing_system)
            .unwrap();
        world.run_system_once(gravity_nightmares_system).unwrap();

        // Assert: Rest should recover normally, no nightmare
        let rest = world.get::<Needs>(pop).unwrap().rest;
        assert!(
            rest > 0.12,
            "Rest did not recover in suspension pod: {}",
            rest
        );
        assert!(world.get::<GravityNightmare>(pop).is_none());
    }

    #[test]
    fn test_normal_pop_in_regular_bed() {
        let mut world = setup_app();

        let house = world
            .spawn((
                Housing {
                    capacity: 1,
                    residents: vec![],
                },
                Building {
                    building_type: BuildingType::Housing,
                },
                GridPosition { x: 0, y: 0 },
            ))
            .id();

        let pop = world
            .spawn((
                Pop,
                Needs {
                    hunger: 1.0,
                    rest: 0.1,
                    leisure: 1.0,
                    hygiene: 1.0,
                },
                AssignedTo {
                    entity: house,
                    assignment_type: AssignmentType::HousingResident,
                },
            ))
            .id();
        world.get_mut::<Housing>(house).unwrap().residents.push(pop);

        world
            .run_system_once(crate::layer1::housing::restore_rest_in_housing_system)
            .unwrap();
        world.run_system_once(gravity_nightmares_system).unwrap();

        // Assert: Normal pop sleeps fine, no nightmare
        let rest = world.get::<Needs>(pop).unwrap().rest;
        assert!(rest > 0.12, "Normal pop did not recover rest: {}", rest);
        assert!(world.get::<GravityNightmare>(pop).is_none());
    }
}

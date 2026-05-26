use crate::layer1::biology::health::Health;
use crate::layer1::social::morale::Morale;
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct HabSeed {
    pub growth_stage: f32,
}

#[derive(Component)]
pub struct HabPlant;

#[derive(Component)]
pub struct ResidentOf {
    pub building: Entity,
}

#[derive(Component)]
pub struct SymbioticLink {
    pub strength: f32,
}

pub fn grow_hab_seed_system(mut commands: Commands, mut seeds: Query<(Entity, &mut HabSeed)>) {
    for (entity, mut seed) in seeds.iter_mut() {
        seed.growth_stage += 0.1;

        if seed.growth_stage >= 100.0 {
            commands.entity(entity).remove::<HabSeed>();
            commands.entity(entity).insert(HabPlant);
        }
    }
}

pub fn process_symbiotic_pain_system(
    plants: Query<(Entity, &Health), With<HabPlant>>,
    mut residents: Query<
        (&mut Health, &mut Morale, &ResidentOf, &SymbioticLink),
        Without<HabPlant>,
    >,
) {
    for (plant_entity, plant_health) in plants.iter() {
        let is_damaged = plant_health.current < plant_health.max;
        let pain_amount = plant_health.max - plant_health.current;

        if is_damaged {
            for (mut res_health, mut res_morale, resident, link) in residents.iter_mut() {
                if resident.building == plant_entity {
                    res_health.current -= pain_amount * 0.1 * link.strength;
                    res_morale.value -= pain_amount * 0.01 * link.strength;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::layer1::architecture::building::{Building, BuildingType};
    use crate::layer1::biology::health::Health;
    use crate::layer1::entities::pop::Pop;
    use crate::layer1::social::morale::Morale;

    fn setup_world() -> World {
        let world = World::new();
        world
    }

    #[test]
    fn test_hab_seed_grows_over_time() {
        let mut world = setup_world();

        let seed = world
            .spawn((
                Building {
                    building_type: BuildingType::Housing,
                }, // mock
                HabSeed { growth_stage: 0.0 },
                Health {
                    current: 100.0,
                    max: 100.0,
                    has_rust_lung: false,
                },
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(grow_hab_seed_system);
        schedule.run(&mut world);

        let hab_seed = world.get::<HabSeed>(seed).unwrap();
        assert!(hab_seed.growth_stage > 0.0);
    }

    #[test]
    fn test_hab_plant_damage_causes_symbiotic_pain() {
        let mut world = setup_world();

        let plant = world
            .spawn((
                Building {
                    building_type: BuildingType::Housing,
                }, // mock
                HabPlant,
                Health {
                    current: 50.0,
                    max: 100.0,
                    has_rust_lung: false,
                }, // Damaged
            ))
            .id();

        let pop = world
            .spawn((
                Pop,
                ResidentOf { building: plant },
                SymbioticLink { strength: 1.0 },
                Health {
                    current: 100.0,
                    max: 100.0,
                    has_rust_lung: false,
                },
                Morale {
                    value: 1.0,
                    modifiers: vec![],
                },
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(process_symbiotic_pain_system);
        schedule.run(&mut world);

        let health = world.get::<Health>(pop).unwrap();
        let morale = world.get::<Morale>(pop).unwrap();
        assert!(health.current < 100.0); // Took sympathetic damage
        assert!(morale.value < 1.0); // Took sympathetic morale hit
    }
}

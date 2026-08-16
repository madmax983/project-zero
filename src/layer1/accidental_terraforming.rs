use bevy_ecs::prelude::*;

#[allow(dead_code)]
#[derive(Component, PartialEq, Eq, Debug, Clone, Copy)]
pub enum BiomeType {
    Ice,
    Arid,
    Tundra,
    Ocean,
    Toxic,
}

#[derive(Component, Default)]
pub struct AtmosphericByproducts {
    pub heat: f32,
    pub toxins: f32,
}

#[derive(Component)]
pub struct TerraformEmissions {
    pub heat_per_tick: f32,
    pub toxins_per_tick: f32,
}

#[allow(dead_code)]
#[derive(Event)]
pub struct BiomeShiftEvent {
    pub planet: Entity,
    pub old_biome: BiomeType,
    pub new_biome: BiomeType,
}

pub fn accumulate_emissions_system(
    mut planets: Query<&mut AtmosphericByproducts>,
    buildings: Query<&TerraformEmissions, With<crate::layer1::building::Building>>,
) {
    let mut total_heat = 0.0;
    let mut total_toxins = 0.0;

    for emission in buildings.iter() {
        total_heat += emission.heat_per_tick;
        total_toxins += emission.toxins_per_tick;
    }

    for mut byproducts in planets.iter_mut() {
        byproducts.heat += total_heat;
        byproducts.toxins += total_toxins;
    }
}

pub fn accidental_terraforming_system(
    mut planets: Query<(Entity, &mut BiomeType, &mut AtmosphericByproducts)>,
    mut events: EventWriter<BiomeShiftEvent>,
) {
    for (entity, mut biome, mut byproducts) in planets.iter_mut() {
        let mut changed = false;
        let old_biome = *biome;

        if *biome == BiomeType::Ice && byproducts.heat >= 1000.0 {
            *biome = BiomeType::Ocean;
            byproducts.heat -= 1000.0;
            changed = true;
        } else if byproducts.toxins >= 1000.0 {
            *biome = BiomeType::Toxic;
            byproducts.toxins -= 1000.0;
            changed = true;
        }

        if changed {
            events.send(BiomeShiftEvent {
                planet: entity,
                old_biome,
                new_biome: *biome,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_emissions_accumulate_byproducts() {
        let mut world = World::new();

        let planet = world
            .spawn((
                BiomeType::Ice,
                AtmosphericByproducts {
                    heat: 0.0,
                    toxins: 0.0,
                },
            ))
            .id();

        world.spawn((
            crate::layer1::building::Building {
                building_type: crate::layer1::building::BuildingType::Housing,
            },
            TerraformEmissions {
                heat_per_tick: 10.0,
                toxins_per_tick: 5.0,
            },
        ));

        let _ = world.run_system_once(accumulate_emissions_system);

        let byproducts = world.get::<AtmosphericByproducts>(planet).unwrap();
        assert_eq!(
            byproducts.heat, 10.0,
            "Heat should accumulate from buildings"
        );
        assert_eq!(
            byproducts.toxins, 5.0,
            "Toxins should accumulate from buildings"
        );
    }

    #[test]
    fn test_terraforming_biome_shift() {
        let mut world = World::new();

        let planet = world
            .spawn((
                BiomeType::Ice,
                AtmosphericByproducts {
                    heat: 1000.0,
                    toxins: 0.0,
                },
            ))
            .id();

        world.init_resource::<Events<BiomeShiftEvent>>();

        let _ = world.run_system_once(accidental_terraforming_system);

        let biome = world.get::<BiomeType>(planet).unwrap();
        assert_eq!(
            *biome,
            BiomeType::Ocean,
            "High heat on Ice should melt it to Ocean"
        );

        let events = world.resource::<Events<BiomeShiftEvent>>();
        let mut reader = events.get_cursor();
        assert_eq!(
            reader.read(events).count(),
            1,
            "Should emit a BiomeShiftEvent"
        );
    }
}

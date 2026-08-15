//! Collective Guilt
//!
//! Tracks the sociological impact of past atrocities committed by a faction.
//! High guilt affects diplomatic standing and internal stability, requiring reparations or propaganda.

use crate::layer1::social::unrest::Unrest;
use bevy_ecs::prelude::*;

#[derive(Resource, Default)]
pub struct GuiltResource {
    pub amount: f32,
}

#[derive(Component)]
pub struct PsychicResonance {
    pub intensity: f32,
}

pub fn process_guilt_generation_system(world: &mut World) {
    let mut total_resonance = 0.0;

    let mut query = world.query::<&PsychicResonance>();
    for resonance in query.iter(world) {
        total_resonance += resonance.intensity;
    }

    if let Some(mut guilt) = world.get_resource_mut::<GuiltResource>() {
        guilt.amount += total_resonance;
    }
}

pub fn apply_guilt_unrest_system(world: &mut World) {
    let guilt_amount = world
        .get_resource::<GuiltResource>()
        .map_or(0.0, |g| g.amount);
    if guilt_amount == 0.0 {
        return;
    }

    let unrest_increase = guilt_amount * 0.1; // 10% conversion to unrest

    if let Some(mut unrest) = world.get_resource_mut::<Unrest>() {
        let label = "Psychic Resonance".to_string();
        if let Some(modifier) = unrest.modifiers.iter_mut().find(|m| m.label == label) {
            modifier.value = unrest_increase;
            modifier.duration = 10;
        } else {
            unrest
                .modifiers
                .push(crate::layer1::social::unrest::UnrestModifier {
                    value: unrest_increase,
                    duration: 10,
                    label,
                });
        }
    }
}

pub fn process_construction_on_ruins_system(world: &mut World) {
    use crate::layer1::architecture::building::Building;
    use crate::layer1::architecture::ruins::Ruin;
    use crate::layer1::map::GridPosition;

    // In a real implementation we'd check for newly completed buildings.
    // For this minimal MVP, we'll check if a Building and a Ruin exist at the same GridPosition,
    // and if the Building lacks PsychicResonance but the Ruin has it, we copy it over.

    // Extract ruin positions and resonances
    // ⚡ Bolt Optimization: Pre-allocated vector capacity based on query size to eliminate dynamic heap reallocation.
    let mut query = world.query::<(&Ruin, &GridPosition, &PsychicResonance)>();
    let mut ruin_data = Vec::with_capacity(query.iter(world).len());
    for (_, pos, res) in query.iter(world) {
        ruin_data.push((*pos, res.intensity));
    }

    let mut buildings_to_update = Vec::new();
    let mut b_query = world
        .query_filtered::<(Entity, &GridPosition), (With<Building>, Without<PsychicResonance>)>();
    for (entity, pos) in b_query.iter(world) {
        for (r_pos, intensity) in &ruin_data {
            if pos.x == r_pos.x && pos.y == r_pos.y {
                buildings_to_update.push((entity, *intensity));
            }
        }
    }

    for (entity, intensity) in buildings_to_update {
        world
            .entity_mut(entity)
            .insert(PsychicResonance { intensity });
    }
}

#[cfg(test)]
mod tests {
    use super::{
        apply_guilt_unrest_system, process_guilt_generation_system, GuiltResource, PsychicResonance,
    };
    use crate::layer1::architecture::building::{BuildingType, MaterialType};
    use crate::layer1::architecture::ruins::Ruin;
    use crate::layer1::map::GridPosition;
    use bevy_ecs::prelude::*;

    #[test]
    fn test_building_on_ruins_generates_psychic_resonance() {
        let mut world = World::new();
        // Setup ruin
        world.spawn((
            Ruin {
                original_type: BuildingType::Housing,
                material: MaterialType::Stone,
            },
            GridPosition { x: 5, y: 5 },
            PsychicResonance { intensity: 10.0 }, // Emits resonance
        ));

        // When a building is placed on the ruin, it absorbs the resonance
        let new_building = world
            .spawn((
                crate::layer1::architecture::building::Building {
                    building_type: BuildingType::Smelter,
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        // Act
        crate::layer3::guilt::process_construction_on_ruins_system(&mut world);

        // Assert
        let res = world.get::<PsychicResonance>(new_building);
        assert!(
            res.is_some(),
            "Building should absorb resonance from the ruin below it."
        );
        assert_eq!(res.unwrap().intensity, 10.0);
    }

    #[test]
    fn test_guilt_resource_generation() {
        let mut world = World::new();
        world.insert_resource(GuiltResource { amount: 0.0 });

        // Setup a resonant building
        world.spawn((
            crate::layer1::architecture::building::Building {
                building_type: BuildingType::Smelter,
            },
            PsychicResonance { intensity: 5.0 },
        ));

        process_guilt_generation_system(&mut world);

        // Assert guilt generated
        assert_eq!(world.resource::<GuiltResource>().amount, 5.0);
    }

    #[test]
    fn test_guilt_causes_unrest() {
        let mut world = World::new();
        world.insert_resource(GuiltResource { amount: 100.0 });

        world.insert_resource(crate::layer1::social::unrest::Unrest {
            level: 0.0,
            modifiers: vec![],
        });

        apply_guilt_unrest_system(&mut world);

        assert_eq!(
            world
                .resource::<crate::layer1::social::unrest::Unrest>()
                .modifiers[0]
                .value,
            10.0
        );
    }
}

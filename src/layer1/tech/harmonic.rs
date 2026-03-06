use crate::layer1::health::Health;
use crate::layer1::map::GridPosition;
use crate::layer1::resources::{ColonyResources, ResourceType};
use crate::layer1::Structure;
use bevy_ecs::prelude::*;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Frequency(pub u32);

#[derive(Component, Debug, Clone)]
pub struct HarmonicDrill {
    pub frequency: Frequency,
    pub radius: f32,
    pub active: bool,
}

#[derive(Component, Debug, Clone)]
pub struct ResonantMaterial {
    pub frequency: Frequency,
    pub material_type: ResourceType,
}

type MaterialQueryTuple<'a> = (
    Entity,
    &'a ResonantMaterial,
    &'a GridPosition,
    Option<&'a mut Health>,
    Option<&'a mut Structure>,
);

pub fn process_harmonic_mining(
    mut commands: Commands,
    drills: Query<(&HarmonicDrill, &GridPosition)>,
    mut materials: Query<MaterialQueryTuple<'_>>,
    mut resources: ResMut<ColonyResources>,
) {
    let mut processed = std::collections::HashSet::new();

    for (drill, drill_pos) in drills.iter() {
        if !drill.active {
            continue;
        }

        for (target_entity, material, target_pos, health, structure) in materials.iter_mut() {
            if processed.contains(&target_entity) {
                continue;
            }

            // Distance check (using Chebyshev distance as proposed)
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let distance = drill_pos.distance_chebyshev(*target_pos);

            if distance <= drill.radius as u32 {
                // Resonance Check
                if drill.frequency == material.frequency {
                    // It's a structure/unit -> Damage it
                    let mut damaged = false;

                    if let Some(mut h) = health {
                        h.current = 0.0;
                        damaged = true;
                    }
                    if let Some(mut s) = structure {
                        s.current_hp = 0.0;
                        damaged = true;
                    }

                    if damaged {
                        // Logic to handle destruction event if needed
                        // Wait for layer1::health despawn system or damage systems to process
                    } else {
                        // It's raw resource -> Mine it
                        match material.material_type {
                            ResourceType::Ore => resources.add_ore(10.0),
                            ResourceType::Wood => resources.add_wood(10.0),
                            ResourceType::Stone => resources.add_stone(10.0),
                            ResourceType::Planks => resources.add_planks(10.0),
                            ResourceType::Food => resources.add_food(10.0),
                            // Default case
                            _ => {}
                        }
                        commands.entity(target_entity).despawn();
                    }
                    processed.insert(target_entity);
                }
            }
        }
    }
}

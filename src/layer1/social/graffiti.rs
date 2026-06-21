use crate::layer1::architecture::Building;
use crate::layer1::map::GridPosition;
use crate::layer1::psychology::stress::StressTracker;
use crate::layer1::psychology::traits::{Trait, Traits};
use crate::layer1::social::morale::{MoodModifier, Morale};
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct RebelliousGraffiti {
    pub intensity: f32,
}

#[derive(Component)]
pub struct GraffitiEfficiency {
    pub current: f32,
}

pub fn propaganda_graffiti_system(world: &mut World) {
    let mut to_tag = Vec::new();

    // Find stressed creative pops and their positions
    let mut creative_positions = Vec::new();
    for (_, traits, stress, pos) in world
        .query::<(Entity, &Traits, &StressTracker, &GridPosition)>()
        .iter(world)
    {
        if stress.accumulated_stress > 80.0 && traits.has(Trait::Artistic) {
            creative_positions.push(*pos);
        }
    }

    // Find buildings at those positions
    for (entity, _, pos) in world
        .query::<(Entity, &Building, &GridPosition)>()
        .iter(world)
    {
        if creative_positions.contains(pos) && world.get::<RebelliousGraffiti>(entity).is_none() {
            to_tag.push(entity);
        }
    }

    for entity in to_tag {
        if let Ok(mut cmds) = world.get_entity_mut(entity) {
            cmds.insert(RebelliousGraffiti { intensity: 1.0 });
        }
    }
}

pub fn graffiti_aura_system(world: &mut World) {
    let mut graffiti_positions = Vec::new();
    for (graffiti, pos) in world
        .query::<(&RebelliousGraffiti, &GridPosition)>()
        .iter(world)
    {
        graffiti_positions.push((*pos, graffiti.intensity));
    }

    if graffiti_positions.is_empty() {
        return;
    }

    for (pop_pos, mut eff, mut morale) in world
        .query::<(&GridPosition, Option<&mut GraffitiEfficiency>, &mut Morale)>()
        .iter_mut(world)
    {
        let mut affected = false;
        let mut best_intensity = 0.0;

        for (g_pos, intensity) in &graffiti_positions {
            if pop_pos.distance_chebyshev(*g_pos) <= 2 {
                affected = true;
                if *intensity > best_intensity {
                    best_intensity = *intensity;
                }
            }
        }

        if affected {
            // Apply a one-time penalty for testing purposes (since Efficiency is stubbed) or maintain a steady state
            if let Some(ref mut eff_val) = eff {
                // To avoid infinite scaling, just set it if it's currently 1.0 (baseline)
                if eff_val.current > 0.95 {
                    eff_val.current *= 0.9;
                }
            }

            // Check if they already have the modifier to prevent infinite vector growth
            if !morale
                .modifiers
                .iter()
                .any(|m| m.label == "Venting: Rebellious Graffiti")
            {
                morale.modifiers.push(MoodModifier {
                    value: 0.2 * best_intensity,
                    duration: 50,
                    label: "Venting: Rebellious Graffiti".to_string(),
                });
            }
        }
    }
}

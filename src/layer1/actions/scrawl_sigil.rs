use bevy_ecs::prelude::*;
use crate::layer1::utility_types::ActionType;
use crate::layer1::utility_eval_types::PopEvalData;
use crate::layer1::memetic::MemeticCarrier;
use crate::layer1::building::{Building, BuildingType};
use crate::layer1::map::GridPosition;
use rand::seq::IteratorRandom;

/// Evaluates the desire to scrawl memetic sigils on walls.
///
/// This action is available only to Pops infected with [`crate::layer1::memetic::MemeticCarrier`].
/// It overrides normal priorities with a high utility score (2.0).
pub fn evaluate_scrawl_memetic_sigil(
    data: &PopEvalData,
    world: &mut World,
) -> Option<(ActionType, f32, Option<Entity>)> {
    // 1. Am I a carrier?
    if world.get::<MemeticCarrier>(data.entity).is_none() {
        return None;
    }

    // 2. Find a wall to deface
    let mut rng = rand::thread_rng();

    let mut query = world.query::<(Entity, &GridPosition, &Building)>();
    let candidates: Vec<(Entity, GridPosition)> = query.iter(world)
        .filter(|(_, _, b)| b.building_type == BuildingType::Wall)
        .map(|(e, p, _)| (e, *p))
        .collect();

    // Pick one
    if let Some((target, _pos)) = candidates.into_iter().choose(&mut rng) {
        // High utility to override everything else (2.0 vs normal 1.0 max)
        return Some((ActionType::ScrawlMemeticSigil, 2.0, Some(target)));
    }

    None
}

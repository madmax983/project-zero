use bevy_ecs::prelude::*;
use crate::layer1::utility_types::ActionType;
use crate::layer1::utility_eval_types::{PopEvalData, UtilityAIBuffer};
use rand::seq::SliceRandom;

/// Evaluates the desire to scrawl memetic sigils on walls.
///
/// This action is available only to Pops infected with [`crate::layer1::memetic::MemeticCarrier`].
/// It overrides normal priorities with a high utility score (2.0).
pub fn evaluate_scrawl_memetic_sigil(
    data: &PopEvalData,
    buffer: &UtilityAIBuffer,
) -> Option<(ActionType, f32, Option<Entity>)> {
    // 1. Am I a carrier?
    if !data.is_memetic_carrier {
        return None;
    }

    // 2. Find a wall to deface
    let mut rng = rand::thread_rng();

    // Pick one
    if let Some(proxy) = buffer.walls.choose(&mut rng) {
        // High utility to override everything else (2.0 vs normal 1.0 max)
        return Some((ActionType::ScrawlMemeticSigil, 2.0, Some(proxy.entity)));
    }

    None
}

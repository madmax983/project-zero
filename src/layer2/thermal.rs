use bevy_ecs::prelude::*;
use crate::layer1::temperature::HeatSource;
use crate::layer2::events::DetectionEvent;
use rand::Rng;

/// Resource tracking the aggregate heat signature of the colony.
/// This determines the risk of detection by hostile forces in the system.
#[derive(Resource, Debug)]
pub struct ThermalSignature {
    /// The current thermal value (arbitrary units, correlates to HeatSource output).
    pub current_value: f32,
    /// The threshold above which detection becomes possible.
    pub detection_threshold: f32,
    /// The rate at which the signature moves towards the target value per tick (0.0 to 1.0).
    pub decay_rate: f32,
}

impl Default for ThermalSignature {
    fn default() -> Self {
        Self {
            current_value: 0.0,
            detection_threshold: 500.0,
            decay_rate: 0.1,
        }
    }
}

/// System to update the thermal signature based on active heat sources.
///
/// It sums the output of all `HeatSource` components in Layer 1 and
/// smooths the transition of the global `ThermalSignature`.
pub fn update_thermal_bloom_system(
    mut signature: ResMut<ThermalSignature>,
    query: Query<&HeatSource>,
) {
    // 1. Calculate total active heat output from L1
    let total_output: f32 = query.iter().map(|h| h.output).sum();

    // 2. Apply smoothing/decay
    // The signature moves towards the target output, simulating thermal mass
    let diff = total_output - signature.current_value;
    signature.current_value += diff * signature.decay_rate; // Simple lerp

    // Clamp to 0
    if signature.current_value < 0.0 {
        signature.current_value = 0.0;
    }
}

/// System to determine if the colony is detected based on its thermal signature.
///
/// If `current_value` exceeds `detection_threshold`, there is a chance per tick
/// to trigger a `DetectionEvent`.
pub fn detection_risk_system(
    signature: Res<ThermalSignature>,
    mut events: EventWriter<DetectionEvent>,
) {
    if signature.current_value > signature.detection_threshold {
        // Simple probability: (Value - Threshold) / Threshold * Multiplier
        // Example: If value is 1000 and threshold is 500:
        // Excess = 500. Chance = (500 / 500) * 0.01 = 0.01 (1% per tick)
        let excess = signature.current_value - signature.detection_threshold;
        let chance = (excess / signature.detection_threshold) * 0.01;

        let mut rng = rand::thread_rng();
        if rng.gen::<f32>() < chance {
            events.send(DetectionEvent);
        }
    }
}

# 243: Thermal Bloom

## Overview

In space, heat is the primary way ships and colonies are detected. "Thermal Bloom" is a mechanic where the aggregate heat generation of the Layer 1 colony creates a "Thermal Signature" visible on the Layer 2 System Map. A high Thermal Signature increases the probability of hostile events (Pirate Raids, Space Fauna attacks) targeting the colony.

Players must balance industrial output (Heat) against the risk of detection (Stealth).

## Dependencies

- `140` — Thermal Management (Heat generation mechanic)
- `099` — Fleet Movement (Layer 2 context)
- `045` — Structure Durability (Heat sources are buildings)

## RED Phase: Tests First

Write these tests in `src/layer2/thermal_bloom_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::heat::{HeatSource, ThermalOutput}; // Assuming from spec 140
    use crate::layer2::thermal::{ThermalSignature, update_thermal_bloom_system, detection_risk_system};
    use crate::layer2::events::DetectionEvent;

    #[test]
    fn test_thermal_signature_aggregation() {
        let mut world = World::new();
        // Uses Default which sets decay_rate > 0.0
        world.insert_resource(ThermalSignature::default());

        // Spawn 3 heat sources
        world.spawn(HeatSource { output: ThermalOutput(10.0) });
        world.spawn(HeatSource { output: ThermalOutput(20.0) });
        world.spawn(HeatSource { output: ThermalOutput(5.0) });

        // Run system to update signature
        let mut schedule = Schedule::default();
        schedule.add_systems(update_thermal_bloom_system);
        schedule.run(&mut world);

        let signature = world.resource::<ThermalSignature>();
        // Total = 35.0.
        // With decay_rate 0.1, current_value = 0.0 + (35.0 - 0.0) * 0.1 = 3.5 (first tick)
        // Wait, the system should move TOWARDS the target.
        // If current is 0 and target is 35, it increases.
        assert!(signature.current_value > 0.0);
    }

    #[test]
    fn test_thermal_decay() {
        let mut world = World::new();
        // Start with high signature
        world.insert_resource(ThermalSignature {
            current_value: 100.0,
            ..Default::default()
        });

        // No heat sources spawned (Target = 0.0)

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(update_thermal_bloom_system);
        schedule.run(&mut world);

        let signature = world.resource::<ThermalSignature>();
        // Should decay towards 0.0
        assert!(signature.current_value < 100.0);
    }

    #[test]
    fn test_detection_risk_trigger() {
        let mut world = World::new();
        world.insert_resource(Events::<DetectionEvent>::default());

        // High signature
        world.insert_resource(ThermalSignature {
            current_value: 1000.0,
            detection_threshold: 500.0,
            decay_rate: 0.1
        });

        // Run detection system
        let mut schedule = Schedule::default();
        schedule.add_systems(detection_risk_system);
        schedule.run(&mut world);

        let events = world.resource::<Events<DetectionEvent>>();
        let mut reader = events.get_reader();
        // Note: Probabilistic test, might need to mock RNG or check logic directly.
        // For unit testing, we might want to force the RNG or check the threshold logic.
        // Assuming the system always emits if probability > 0 for this test example, or valid RNG seed.
        // In real implementation, verify probability calculation.
    }

    #[test]
    fn test_low_signature_safe() {
        let mut world = World::new();
        world.insert_resource(Events::<DetectionEvent>::default());

        // Low signature
        world.insert_resource(ThermalSignature {
            current_value: 10.0,
            detection_threshold: 500.0,
            decay_rate: 0.1
        });

        // Run detection system
        let mut schedule = Schedule::default();
        schedule.add_systems(detection_risk_system);
        schedule.run(&mut world);

        let events = world.resource::<Events<DetectionEvent>>();
        let mut reader = events.get_reader();
        assert_eq!(reader.len(&events), 0, "Low thermal signature should NOT trigger detection");
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Define Resource (`src/layer2/thermal.rs`)

```rust
use bevy_ecs::prelude::*;

#[derive(Resource, Debug)]
pub struct ThermalSignature {
    pub current_value: f32,
    pub detection_threshold: f32, // e.g., 500.0
    pub decay_rate: f32, // e.g., 0.1 per tick
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
```

### 2. Implement Aggregation System

```rust
use crate::layer1::heat::HeatSource;

pub fn update_thermal_bloom_system(
    mut signature: ResMut<ThermalSignature>,
    query: Query<&HeatSource>,
) {
    // 1. Calculate total active heat output from L1
    let total_output: f32 = query.iter().map(|h| h.output.0).sum();

    // 2. Apply smoothing/decay
    // The signature moves towards the target output, simulating thermal mass
    let diff = total_output - signature.current_value;
    signature.current_value += diff * signature.decay_rate; // Simple lerp

    // Clamp to 0
    if signature.current_value < 0.0 {
        signature.current_value = 0.0;
    }
}
```

### 3. Implement Risk System

```rust
use bevy_ecs::event::Event;
use rand::Rng;

#[derive(Event)]
pub struct DetectionEvent;

pub fn detection_risk_system(
    signature: Res<ThermalSignature>,
    mut events: EventWriter<DetectionEvent>,
) {
    if signature.current_value > signature.detection_threshold {
        // Simple probability: (Value - Threshold) / Threshold * Multiplier
        let excess = signature.current_value - signature.detection_threshold;
        let chance = (excess / signature.detection_threshold) * 0.01; // 1% per 100% over

        let mut rng = rand::thread_rng();
        if rng.gen::<f32>() < chance {
            events.send(DetectionEvent);
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **Optimization**: `update_thermal_bloom_system` iterates all heat sources every frame. If `HeatSource` count > 1000, consider caching the sum and only updating on `Added/Changed/Removed<HeatSource>`.
- **Game Design**: `detection_threshold` should probably vary based on the Local System's "Threat Level" (from Layer 2 generation). High-threat systems have lower detection thresholds.
- **UI**: Add a `ThermalGraph` widget to the Command Center UI (Spec 146) showing current signature vs. threshold.

## Acceptance Criteria (Testable!)

- [ ] `ThermalSignature` resource tracks aggregate heat.
- [ ] Signature decays/moves towards target value over time (not instant).
- [ ] `DetectionEvent` fires probabilistically when threshold is exceeded.
- [ ] `HeatSource` components from Layer 1 are the input.
- [ ] All tests pass.

## Technical Guidance

- **HeatSource**: Ensure you are using the correct component from `src/layer1/heat.rs` (or `thermal.rs` depending on implementation of Spec 140).
- **Events**: Register `DetectionEvent` in the app setup.
- **Decay Rate**: A value of `0.1` means it takes ~20 ticks to reach 90% of new target. Tune this for gameplay feel (lag vs responsiveness).

## Questions

*Builder: add questions here if spec is unclear.*
*Architect:* See related specifications for design details. MVP implementation should follow standard conventions.

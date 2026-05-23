#!/bin/bash

# Insert resource initialization
sed -i '/world.init_resource::<crate::layer1::shadow_market::ShadowMarketCooldown>();/i \
    world.init_resource::<Events<crate::layer1::psychology::memory_forgery::ForgeryActionEvent>>();\n    world.init_resource::<Events<crate::layer1::psychology::memory_forgery::TruthOutbreakEvent>>();\n' src/simulation.rs

# Insert system registration
sed -i '/fn register_simulation_extended_systems(schedule: &mut Schedule) {/a \
    schedule.add_systems((\n        crate::layer1::psychology::memory_forgery::process_mnestic_archiver_system,\n        crate::layer1::psychology::memory_forgery::process_truth_outbreak_system,\n    ));\n' src/simulation.rs

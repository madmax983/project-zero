#!/bin/bash

# Insert schedule builder test fixes in init_simulation_resources if they are part of test_schedule_runs_on_fresh_world missing events
sed -i '/world.init_resource::<crate::layer1::shadow_market::ShadowMarketCooldown>();/i \
    world.init_resource::<Events<crate::layer1::psychology::memory_forgery::ForgeryActionEvent>>();\n    world.init_resource::<Events<crate::layer1::psychology::memory_forgery::TruthOutbreakEvent>>();\n' src/simulation.rs

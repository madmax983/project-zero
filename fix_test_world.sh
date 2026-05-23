#!/bin/bash

# Insert into test_schedule_runs_on_fresh_world
sed -i '/world.init_resource::<Events<crate::layer3::diplomacy::fading_homeworld::PlayerDemandResponse>>();/a \
        world.init_resource::<Events<crate::layer1::psychology::memory_forgery::ForgeryActionEvent>>();\n        world.init_resource::<Events<crate::layer1::psychology::memory_forgery::TruthOutbreakEvent>>();\n' src/simulation.rs

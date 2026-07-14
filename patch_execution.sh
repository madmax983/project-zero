#!/bin/bash
sed -i '/pub fn register(schedule: &mut Schedule) {/a \
    schedule.add_systems(\
        (\
            crate::layer1::social::propaganda_graffitists::propaganda_graffiti_system,\
            crate::layer1::social::propaganda_graffitists::graffiti_aura_system,\
            crate::layer1::core::integration::graffiti_chronicle_bridge,\
        )\
            .chain()\
            .in_set(super::Layer1SystemSet::Execution),\
    );' src/layer1/systems/execution.rs

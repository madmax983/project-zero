#!/bin/bash
sed -i '/if !world.contains_resource::<Events<crate::layer1::grafting::GraftBuildingEvent>>()/!b;n;a\
    if !world.contains_resource::<Events<crate::layer3::market::quantum_famine::MarketPanicEvent>>() {\
        world.init_resource::<Events<crate::layer3::market::quantum_famine::MarketPanicEvent>>();\
        world.init_resource::<Events<crate::layer3::market::quantum_famine::ExportDumpEvent>>();\
    }' src/simulation.rs

sed -i '/world.init_resource::<Events<crate::layer1::grafting::GraftBuildingEvent>>();/,+3d' src/simulation.rs

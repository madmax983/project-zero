#!/bin/bash
sed -i 's/let mut resources = world.resource_mut::<ColonyResources>();/let mut resources = world.resource_mut::<ColonyResources>();\n        resources.max_scrap = 50000.0;/g' src/layer3/market/debt_collector.rs

#!/bin/bash
sed -i 's/resources.get/resources.get_amount/g' src/layer3/market/debt_collector.rs
sed -i 's/resources.add(/resources.add_credits(/g' src/layer3/market/debt_collector.rs
sed -i 's/ResourceType::Credits/ResourceType::Scrap/g' src/layer3/market/debt_collector.rs
sed -i 's/resources.add_credits(ResourceType::Scrap, 5000.0);/resources.add_scrap(5000.0);/g' src/layer3/market/debt_collector.rs
sed -i 's/resources.add_credits(ResourceType::Scrap, 1000.0);/resources.add_scrap(1000.0);/g' src/layer3/market/debt_collector.rs
sed -i 's/BuildingType::Turret/BuildingType::Office/g' src/layer3/market/debt_collector.rs

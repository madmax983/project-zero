#!/bin/bash
sed -i 's/resources.get(ev.resource)/resources.get_amount(ev.resource)/' src/layer3/market/debt_collector.rs
sed -i 's/AuditorRefusalEvent::default()/AuditorRefusalEvent/' src/layer3/market/debt_collector.rs
sed -i 's/BuildingType::Tavern | BuildingType::Statue | BuildingType::Museum/BuildingType::Tavern | BuildingType::Statue | BuildingType::Library/' src/layer3/market/debt_collector.rs

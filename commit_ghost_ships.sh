#!/bin/bash
git add src/layer3/integration.rs
git add tests/integration.rs
git add tests/integration/ghost_ships_bridge.rs
git add src/simulation.rs
git add src/setup.rs
git add design/SEAM_MAP.md
git add design/IN_PROGRESS.md
git add design/COMPLETED.md
git commit -m "feat(integration): connect InTransit to Ghost Ships evaluation" -m "INT-892: Bridges the layer2 fleet InTransit and layer3 Ghost Ships systems. Emits EvaluateTransitEvent and EvaluateLostShipReturnEvent."

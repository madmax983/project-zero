#!/bin/bash
for file in src/layer1/economy/inflation.rs src/layer1/economy/ideological_contraband.rs src/layer1/economy/smugglers_cove.rs src/layer1/skills/generational_atrophy.rs; do
  echo "--- $file ---"
  grep -Hn "^pub struct" $file
done

#!/bin/bash
# Find where DebrisRainChance is used and OrbitalDebris is used.
grep -rn "DebrisRainChance" src/
grep -rn "OrbitalDebris" src/

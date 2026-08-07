feat(integration): connect OrbitalDebris to DebrisRainChance

INT-317: Bridges the `OrbitalDebris` component from Layer 2 with the `DebrisRainChance` resource in Layer 1.

Glue added:
- `orbital_junkyard_bridge_system` calculates the total debris on Layer 2 and clamps it to a 0.0-1.0 range, modifying `DebrisRainChance`.
- Added tests in `tests/integration/orbital_junkyard_bridge.rs`.

import re

with open("design/SEAM_MAP.md", "r") as f:
    content = f.read()

new_seams = """
### INT-1081: Cryptid Sightings -> Chronicle
- **Date:** 2026-06-16
- **Systems connected:** `cryptid_observation_system` (Cryptids) -> `cryptid_chronicle_bridge_system` (Integration) -> `AddChronicleEvent`
- **Glue added:** Added `cryptid_chronicle_bridge_system` in `src/layer1/core/integration.rs` to emit an `AddChronicleEvent` when a pop's awe becomes > 0 due to seeing a cryptid. Registered in `Layer1SystemSet::Observation`.
- **Tests:** `tests/integration/cryptid_chronicle_bridge.rs`

### INT-1098: Psychoactive Weather -> Pop Mood & Work
- **Date:** 2026-04-30
- **Systems connected:** `WeatherState` and `RoofGrid` -> `apply_weather_moodlets_system` -> `process_euphoria_effects_system` (Integration) -> `Mood`, `CurrentTask`, `Morale`
- **Glue added:** Added `apply_weather_moodlets_system` and `process_euphoria_effects_system` to `src/layer1/systems/execution.rs` to apply moodlets and adjust task efficiency based on weather.
- **Tests:** `tests/integration/psychoactive_weather_bridge.rs`
"""

if "INT-1081" not in content:
    lines = content.split('\n')
    for i, line in enumerate(lines):
        if line.startswith("## Connected Seams"):
            lines.insert(i+1, new_seams)
            break
    with open("design/SEAM_MAP.md", "w") as f:
        f.write("\n".join(lines))

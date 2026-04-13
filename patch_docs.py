import sys
from datetime import datetime

def main():
    date_str = datetime.now().strftime("%Y-%m-%d")

    # 1. Update SEAM_MAP.md
    with open('design/SEAM_MAP.md', 'r') as f:
        seam_map = f.read()

    new_seam = f"""### INT-969: Predatory Weather -> Energy/Heat Systems
- **Date:** {date_str}
- **Systems connected:** `weather_movement_system` (Layer 2) -> `predatory_weather_emission_bridge_system` & `predatory_weather_impact_bridge_system` (Integration)
- **Glue added:**
    - `predatory_weather_emission_bridge_system`: Reads `PowerSource` and `HeatSource` in Layer 1 and updates the colony's `AggroTarget` in Layer 2.
    - `predatory_weather_impact_bridge_system`: Listens to `StormImpactEvent` and damages `Structure` components on Layer 1.
- **Tests:** `tests/integration/predatory_weather.rs`

"""
    if "INT-969: Predatory Weather" not in seam_map:
        seam_map = seam_map.replace("## Connected Seams\n", f"## Connected Seams\n\n{new_seam}")
        with open('design/SEAM_MAP.md', 'w') as f:
            f.write(seam_map)
        print("Updated SEAM_MAP.md")

    # 2. Update COMPLETED.md
    with open('design/COMPLETED.md', 'r') as f:
        completed = f.read()

    if "INT-969" not in completed:
        new_completed = f"- [x] `INT-969` Integration: Predatory Weather -> Energy/Heat Systems — completed {date_str}\n"
        completed = completed + new_completed
        with open('design/COMPLETED.md', 'w') as f:
            f.write(completed)
        print("Updated COMPLETED.md")

if __name__ == "__main__":
    main()

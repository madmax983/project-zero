import sys
from datetime import datetime

today = datetime.today().strftime('%Y-%m-%d')

# Update COMPLETED.md
with open("design/COMPLETED.md", "r") as f:
    completed = f.read()

completed += f"- [x] `INT-1306` Integration: Negative Events -> Architectural Superstition - completed {today}\n"

with open("design/COMPLETED.md", "w") as f:
    f.write(completed)

# Update IN_PROGRESS.md
with open("design/IN_PROGRESS.md", "r") as f:
    in_progress_lines = f.readlines()

with open("design/IN_PROGRESS.md", "w") as f:
    for line in in_progress_lines:
        if "INT-1306" not in line:
            f.write(line)

# Update SEAM_MAP.md
seam_entry = f"""
### INT-1306: Negative Events -> Architectural Superstition
- **Date:** {today}
- **Systems connected:** `PopDied`, `BuildingRemovedEvent`, `PopDiedInAccidentEvent` -> `track_negative_events_bridge_system` -> `NegativeEventHistory`
- **Glue added:** Added `track_negative_events_bridge_system` in `src/layer1/core/integration.rs` to add `NegativeEvent` entries to nearby buildings when catastrophic events occur.
- **Schedule:** Registered the system in `src/layer1/systems/observation.rs`.
- **Tests:** `tests/integration/architectural_superstition_bridge.rs`
"""

with open("design/SEAM_MAP.md", "a") as f:
    f.write(seam_entry)

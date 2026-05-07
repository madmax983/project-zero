import re

with open("design/IN_PROGRESS.md", "r") as f:
    content = f.read()

content = content.replace("- [ ] `623` The Nostalgia Plague — `specs/623-nostalgia-plague.md` — claimed 2026-02-01", "")

with open("design/IN_PROGRESS.md", "w") as f:
    f.write(content)

with open("design/COMPLETED.md", "r") as f:
    content = f.read()

content += "\n- [x] `INT-623` Integration: Nostalgia Plague -> Rumor Spread — completed 2026-02-01\n"

with open("design/COMPLETED.md", "w") as f:
    f.write(content)

with open("design/SEAM_MAP.md", "r") as f:
    content = f.read()

content += """
### INT-623: The Nostalgia Plague Integration
- **Date:** 2026-02-01
- **Systems connected:** `Nostalgia` -> `nostalgia_tavern_bridge_system` -> `RumorSpreadEvent`
- **Glue added:** Added `nostalgia_tavern_bridge_system` in `src/layer1/core/integration.rs` to allow nostalgic pops to proselytize to others in the same Tavern. Registered `RumorSpreadEvent` and nostalgia systems in `src/layer1/systems/observation.rs`.
- **Tests:** `tests/integration/nostalgia_plague_bridge.rs`
"""

with open("design/SEAM_MAP.md", "w") as f:
    f.write(content)

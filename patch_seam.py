import re

with open('design/SEAM_MAP.md', 'r') as f:
    content = f.read()

new_seam = """### INT-768: Dynastic Succession -> Chronicle
- **Date:** 2026-04-10
- **Systems connected:** `process_succession_system` (Diplomacy) -> `AddChronicleEvent` (Chronicle)
- **Glue added:**
    - `dynastic_succession_chronicle_bridge` in `src/layer3/integration.rs` tracking `Added<SuccessionCrisis>` and `Changed<CurrentLeader>`.
- **Tests:** `tests/integration/dynastic_succession_chronicle.rs` (Integration test verified)

"""

# It looks like the file does not have "## Connected Seams" header. Let's prepend it instead.
content = new_seam + content

with open('design/SEAM_MAP.md', 'w') as f:
    f.write(content)

print("Patched SEAM_MAP.md")

with open('design/SEAM_MAP.md', 'r') as f:
    c = f.read()

c = c.replace(
"""### INT-1152: Sub-light Arrival Shock -> Chronicle
- **Date:** 2026-08-19
- **Systems connected:**  -> Chronicle ()
- **Glue added:**  in
- **Tests:** """,
"""### INT-1152: Sub-light Arrival Shock -> Chronicle
- **Date:** 2026-08-19
- **Systems connected:** `SubLightArrivalEvent` -> Chronicle (`AddChronicleEvent`)
- **Glue added:** `sub_light_arrival_chronicle_bridge_system` in `src/layer2/integration.rs`
- **Tests:** `tests/integration/sub_light_arrival.rs`"""
)

with open('design/SEAM_MAP.md', 'w') as f:
    f.write(c)

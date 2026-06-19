## [Integration Bug]
**Bug:** Exclusive systems blocking parallel execution and improperly draining events.
**Fix:** Instead of consuming all events via an exclusive `&mut World` system or `SystemState`, define `PlayerDemandResponse` with `#[derive(Event, Clone)]` and use `EventReader<PlayerDemandResponse>` alongside standard resource queries in a normal system.
**Saved:** Fixes compile errors when using `.chain()` and prevents logic black holes by keeping events in Bevy's normal buffering lifecycle.

**2025-06-02 - Deserialization DoS in Layout Config**
**Threat:** Unbounded JSON deserialization in `decoded_persisted_layout` could allow a malformed, massive string to crash the game (DoS) via memory exhaustion or stack overflow.
**Defense:** Added a 1MB length limit to the layout JSON before passing it to `serde_json::from_str`.

**2025-02-09 - O(N) Inventory Remove Performance Vector**
**Threat:** Heavy usage of `Vec::remove()` on `Inventory::items` under simulation load leads to O(N) shifting of elements. This causes severe simulation lag and frame drops which functions as a DoS vector.
**Defense:** Replaced 10 usages of `Vec::remove()` with `Vec::swap_remove()`, turning an O(N) operation into an O(1) operation.

## [Integration Bug]
**Bug:** Exclusive systems blocking parallel execution and improperly draining events.
**Fix:** Instead of consuming all events via an exclusive `&mut World` system or `SystemState`, define `PlayerDemandResponse` with `#[derive(Event, Clone)]` and use `EventReader<PlayerDemandResponse>` alongside standard resource queries in a normal system.
**Saved:** Fixes compile errors when using `.chain()` and prevents logic black holes by keeping events in Bevy's normal buffering lifecycle.

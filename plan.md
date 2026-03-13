1. **Understand Goal**: Integrate Orbital Crossfire (`ShipDestroyedEvent` in Layer 2) with Layer 1 (`OrbitalEvent`).
2. **Missing Logic**: Write `orbital_crossfire_bridge` in `src/layer1/integration.rs` to consume `ShipDestroyedEvent` and emit `OrbitalEvent`.
    - Note that `OrbitalEvent` is a Component, so we `commands.spawn`.
    - We need `TerrainGrid` to determine grid dimensions (or just hardcode a max if it's not available, but getting it from `Option<Res<TerrainGrid>>` is safer).
    - Actually, `TerrainGrid` does not have a `new` method in test. We can create it manually `TerrainGrid { width: 20, height: 20, tiles: vec![TerrainType::Grass; 400] }`.
3. **Write the code**:
    - Update `src/layer1/integration.rs` with `pub fn orbital_crossfire_bridge(mut commands: Commands, mut events: EventReader<crate::layer2::events::ShipDestroyedEvent>, terrain: Option<Res<crate::layer1::terrain::TerrainGrid>>)`
    - Inside, `for event in events.read() { ... }`
    - Pick random x, y within terrain bounds. (e.g. `(rand::random::<usize>() % width) as i32`).
    - Spawn `OrbitalEvent { target: GridPosition { x, y }, damage: 100.0, heat: 100.0 }`.
4. **Register the system**:
    - In `src/layer1/systems/environment.rs`, register `orbital_crossfire_bridge` before `crate::layer1::orbital_crossfire::impact_system`. Or in `Observation` phase.
    - Since `ShipDestroyedEvent` is emitted in `Observation` phase (or where is fleet combat system registered?), maybe `Observation` or `EventCleanup` is better. Wait, Layer 2 combat system might be in `src/layer2/system.rs` running independently.
    - Let's put `orbital_crossfire_bridge` in `Layer1SystemSet::Environment` before `impact_system`. Or in `Observation` phase, which runs after `Consumption`. Let's use `Observation` just like other chronicle bridges, but it spawns a component that won't be processed until the *next* frame's `Environment` phase. That's perfectly fine. Let's register it in `src/layer1/systems/observation.rs` with `crate::layer1::integration::orbital_crossfire_bridge`. Wait, we just need to register it somewhere in `layer1`. `src/layer1/systems/observation.rs` is fine.
5. **Write tests**:
    - Add `tests/integration/orbital_crossfire_bridge.rs`.
    - Run the tests to verify.
    - Run `cargo fmt`, `cargo clippy`.
6. **Update docs**:
    - Update `design/SEAM_MAP.md` and `design/COMPLETED.md` / `IN_PROGRESS.md`.

1. **Add `Artifact` to `TerrainType`:**
   - Update `TerrainType` enum in `src/layer1/nature/terrain.rs` with `Artifact` variant.
   - Update `name`, `movement_cost`, `is_walkable`, and `heat_retention` matching methods to handle `Artifact`. `is_walkable` should return `false`.
   - Update `headless.rs` and `map.rs` to render the `Artifact` character (e.g. `Ω`).

2. **Implement `Artifact` Component & Auras (`src/layer1/artifacts.rs`):**
   - Note: The file `src/layer1/artifacts/mod.rs` already exists, but we need to ensure map generation integrates this.
   - Create `src/layer1/artifacts/map_gen.rs` (or modify `src/layer1/nature/terrain.rs` directly) to randomly spawn `Artifact` tiles on the map and attach the `Artifact` and `Aura` components to those grid coordinates. The spec says "Modify map generation to randomly spawn a small number of `Artifact` entities on the grid."
   - Wait, `TerrainType::Artifact` is requested, but also `Artifact` components. Let's make map generation place `TerrainType::Artifact` on the grid and spawn entities with `(TerrainType::Artifact, GridPosition, ArtifactAura)` as requested by the spec tests.

3. **Update Map Generation:**
   - Modify `generate_terrain` in `src/layer1/nature/terrain.rs` to spawn a few `Artifact` tiles (and maybe returning their positions to spawn entities, or spawning entities directly if `generate_terrain` takes a `&mut World`, but wait, `generate_terrain` currently returns a `TerrainGrid` and doesn't take `World`. Let's create a system that runs at startup to turn `TerrainType::Artifact` into entities, OR we change `generate_terrain` OR we spawn them in another startup system after `TerrainGrid` is inserted).
   - Ah, the spec test says:
     `world.spawn((TerrainType::Artifact, GridPosition { x: 5, y: 5 }));`
     This implies `TerrainType` is also a component, but currently `TerrainType` is an enum inside `TerrainGrid.tiles`. Wait, let's look at `TerrainType`.

4. **Verify `TerrainType` usage:**
   - Is `TerrainType` used as a component anywhere? Let's check `grep "impl Component for TerrainType"`. No, usually it's derived. We'll add `#[derive(Component)]` to `TerrainType`.
   - Update mining logic in `src/layer1/designation.rs` to ensure `TerrainType::Artifact` cannot be mined or destroyed.

5. **Aura System (`src/layer1/artifacts/mod.rs`):**
   - Use the existing `ArtifactAura` (or update existing `Aura` to `ArtifactAura` as named in spec).
   - Apply aura effects to pops. The spec mentions `AuraEffect::Insight` -> `+Science XP, +Stress`. We will implement this.
   - Wait, `ActiveAuras` already applies effects. Let's make sure it handles the specific ones requested by the spec (e.g. Stress). There's already `AuraEffect::StressModifier`. We just need to ensure the system that accumulates stress uses it, or we apply it directly in `aura_system` or a new system `apply_artifact_auras_system`.

6. **Refactor & Tests:**
   - Add the required RED phase tests to `src/layer1/artifacts/mod.rs` or `src/layer1/nature/terrain.rs`.
   - Pre-commit step.

1. **Define RED phase tests:**
   - In `src/layer1/artifacts/mod.rs` or a new module like `src/layer1/artifacts.rs`, add the RED phase tests given in `specs/541-xeno-artifacts.md`. Wait, there is already an `artifacts` module inside `src/layer1/` but it's a directory containing `mod.rs` and `vr_pod.rs`. I can just add tests to `src/layer1/artifacts/mod.rs` and implement the logic there. I'll need to update `src/layer1/nature/terrain.rs` to add `TerrainType::Artifact` and update map generation. I should actually just implement it in `src/layer1/artifacts.rs`? Wait, `src/layer1/mod.rs` has `pub mod artifacts;` which points to `src/layer1/artifacts/mod.rs` since `artifacts` is a directory. I'll add `xeno_artifacts.rs` inside `artifacts/` or add logic to `artifacts/mod.rs`. Wait, the spec says "Stumbling upon something ancient...".

2. **Add `Artifact` to `TerrainType` enum:**
   - Update `src/layer1/nature/terrain.rs` to add `Artifact` to `TerrainType` enum.
   - Update `name`, `movement_cost`, `is_walkable`, `heat_retention` in `TerrainType`. `is_walkable` should be `false`. `heat_retention` to 0.5 (same as Rock).

3. **Map Generation Update:**
   - Modify map generation logic. The spec says "Modify map generation to randomly spawn a small number of `Artifact` entities on the grid." BUT wait, `generate_terrain` returns a `TerrainGrid`, it doesn't take `World`. So maybe we add a startup system `spawn_artifacts_system` that spawns `(TerrainType::Artifact, GridPosition, ArtifactAura)` entities on the map where `TerrainGrid` has `TerrainType::Artifact`? Or we randomly select 1-3 spots on the grid, set them to `TerrainType::Artifact`, AND spawn entities for them in a startup system. Yes, there's `src/bin/headless.rs` or `src/simulation.rs` where startup systems are added. Let's see how `TerrainGrid` is populated. `generate_terrain` is called, then `world.insert_resource(TerrainGrid)`. Then `world.spawn((TerrainType, GridPosition))` usually? Wait, `TerrainGrid` is a resource containing a flat array. Entities are only spawned for things that need components (like pops, buildings). The spec explicitly says: `world.spawn((TerrainType::Artifact, GridPosition { x: 5, y: 5 }));` which means `TerrainType` should derive `Component`!
   - Wait, looking at `src/layer1/nature/terrain.rs`, `TerrainType` doesn't derive `Component`. I will add `#[derive(Component)]` to it. Wait, `#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]` is what it has. I will add `Component`. Wait, if `TerrainGrid` uses `TerrainType`, I don't need `TerrainType` to be a Component unless the spec demands it. The spec RED phase tests use `TerrainType::Artifact` as a component! I will add `#[derive(Component)]` to `TerrainType` in `src/layer1/nature/terrain.rs`.

4. **Implement the systems:**
   - In `src/layer1/artifacts/mod.rs`, I will update `AuraEffect` to have `Insight` and `Vitality` variants or just `Insight` and create `ArtifactAura` component (to match the spec, or rename `Aura` to `ArtifactAura`). Wait, the spec has `ArtifactAura` and `AuraEffect::Insight`. I will rename/add to match the spec exactly to make tests pass.
   - I'll add a system `apply_artifact_auras_system` to `src/layer1/artifacts/mod.rs`. The system queries `(&ArtifactAura, &GridPosition)` and then applies effects to pops.
   - For `Insight`, it should add Stress (e.g. modify `StressTracker` or `Stress`). Wait, the spec uses `Mood`, `Stress` components. I need to check what components are actually used for Stress. `src/layer1/stress.rs` has `StressTracker`. I'll adapt the test to use `StressTracker` since the spec says "Assume generic test world setup exists" and the exact component names might be slightly different.
   - Implement `generate_artifacts_system` as a `Startup` system that runs after terrain generation. It will randomly pick 1-3 spots on the `TerrainGrid`, set them to `TerrainType::Artifact`, and spawn an entity with `(TerrainType::Artifact, GridPosition, ArtifactAura)`.

5. **Indestructible Artifacts:**
   - In `src/layer1/designation.rs`, `can_designate` logic for `DesignationType::Mine` should explicitly deny `TerrainType::Artifact`. Since `Artifact` is not `Rock`, it will automatically be denied because `can_designate` for `Mine` only allows `TerrainType::Rock` and `ImpactSite`!
   - Wait, if it's already indestructible because it's not `Rock`, that test might pass out of the box if `Mine` only checks for `Rock`.

6. **Refactor Phase:**
   - Refactoring and fixing clippy.

7. **Pre-commit Instructions:**
   - Execute `pre_commit_instructions`.

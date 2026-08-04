Plan:
1. Create `src/layer2/ecophagy.rs` and add `pub mod ecophagy;` to `src/layer2/mod.rs`.
2. Define the RED phase tests in `src/layer2/ecophagy.rs`. Note: We will adapt the spec to match existing types:
   - Use `TerrainType::Mountain`? Wait, I saw `Rock` is the closest. But we can just use `TerrainType::Rock` instead of `TileType::Mountain`, and `TerrainType::Void` (if it exists) or `DeepRock` for `Bedrock`. Wait, `TerrainType::Void` exists! We can use that for "Bedrock".
   - Wait! What about `RawResources`? There is no `RawResources`. I will define `RawResources` within `src/layer2/ecophagy.rs` for MVP since the spec explicitly asks for it:
     ```rust
     #[derive(Resource)]
     pub struct RawResources { pub amount: u32 }
     ```
   - The spec uses `GridPosition { x: 5, y: 5, z: 0 }`. But `GridPosition` doesn't have `z`. We will just use `x: 5, y: 5`.
3. In `src/layer2/fleet.rs`:
   - Extend `FleetOrder` (we will use `FleetCommand` enum in `fleet.rs`? Actually the spec uses `FleetCommand` as an *event* but calls it `FleetCommand::ConsumeTile { fleet: Entity, target: GridPosition }`). The spec says:
     `app.add_event::<FleetCommand>();`
     `app.world_mut().resource_mut::<Events<FleetCommand>>().send(FleetCommand::ConsumeTile { fleet: eater, target: GridPosition { x: 5, y: 5 } });`
     So I will define `FleetCommand` in `src/layer2/fleet.rs` as:
     ```rust
     #[derive(Event, Debug, Clone, Copy)]
     pub enum FleetCommand {
         ConsumeTile { fleet: Entity, target: crate::layer1::map::GridPosition },
     }
     ```
     Also, we will add `WorldEater` component in `src/layer2/fleet.rs`:
     ```rust
     #[derive(Component)]
     pub struct WorldEater { pub efficiency: u32 }
     ```
4. GREEN Phase: Implement `process_world_eater_system` in `src/layer2/ecophagy.rs`.
   - Iterate over `EventReader<FleetCommand>`.
   - If `ConsumeTile { fleet, target }`:
     - Lookup `fleet` in `Query<&WorldEater>`.
     - Check `grid.get(target.x as usize, target.y as usize)`. If it's not `TerrainType::Void` (our equivalent of Bedrock), set it to `TerrainType::Void` and add `eater.efficiency` to `RawResources`.
5. Run tests (`cargo test -p scale --lib layer2::ecophagy`). Make sure they pass.
6. Verify code formatting and linting.

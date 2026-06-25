1. **Fix Missing Pop Simulation Removal**
   - Update `escape_pods_chronicle_bridge` to iterate through `distress_signal.occupants` and despawn them from the Layer 1 simulation.
   - Wait, `commands.entity(occupant_entity).despawn()` is appropriate, OR maybe despawn recursively? Just despawn is probably fine since they are just `Entity`s.

2. **Update Tests**
   - Update `tests/integration/escape_pods_chronicle.rs` to verify that the occupants are despawned from the ECS world.

3. **Verify**
   - Run tests and check clippy.

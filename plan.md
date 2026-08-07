1. Run the following bash command to append the new system to `src/layer2/integration.rs`:
```bash
cat << 'EOF' >> src/layer2/integration.rs

/// INT-317: Bridges `OrbitalDebris` from Layer 2 to `DebrisRainChance` in Layer 1.
pub fn orbital_junkyard_bridge_system(
    query: bevy_ecs::system::Query<&crate::layer2::debris::OrbitalDebris>,
    mut rain_chance: bevy_ecs::system::ResMut<crate::layer1::environment::orbital_junkyard::DebrisRainChance>,
) {
    let mut total_debris = 0.0;
    for debris in query.iter() {
        total_debris += debris.0;
    }
    rain_chance.0 = (total_debris * 0.1).clamp(0.0, 1.0);
}

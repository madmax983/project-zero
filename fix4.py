with open("src/layer2/orbital_scrapyard.rs", "r") as f:
    content = f.read()

# Add RNG logic to check_deorbit_trigger
import re

new_trigger = """// System to trigger deorbit events
pub fn check_deorbit_trigger(
    mut debris_query: Query<&mut OrbitalDebrisField>,
    mut deorbit_events: EventWriter<DeorbitEvent>,
) {
    let mut rng = rand::thread_rng();
    for mut debris in debris_query.iter_mut() {
        if debris.stability < 0.2 {
            // Small random chance to trigger event based on low stability.
            use rand::Rng;
            if rng.gen_bool(0.05) {
                // Randomize position based on TerrainGrid bounds roughly (10 to 70 for 80x50 map)
                let x = rng.gen_range(10..70);
                let y = rng.gen_range(10..40);
                deorbit_events.send(DeorbitEvent { target: GridPosition { x, y }, mass: 1000.0 });
                // Reset stability slightly to prevent frame-by-frame spam
                debris.stability = (debris.stability + 0.1).min(1.0);
            }
        }
    }
}"""

content = re.sub(r"// System to trigger deorbit events.*?(?=\n// System to handle impacts on the ground)", new_trigger + "\n", content, flags=re.DOTALL)

with open("src/layer2/orbital_scrapyard.rs", "w") as f:
    f.write(content)

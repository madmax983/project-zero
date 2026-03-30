import re

with open("src/simulation.rs", "r") as f:
    content = f.read()

systems = """        // Orbital Scrapyard Systems
        crate::layer2::orbital_scrapyard::process_salvage_missions,
        crate::layer2::orbital_scrapyard::check_deorbit_trigger,
        crate::layer2::orbital_scrapyard::process_debris_impact,
        // Debris Systems"""

content = content.replace("// Debris Systems", systems)

with open("src/simulation.rs", "w") as f:
    f.write(content)

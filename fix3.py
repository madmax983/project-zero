import re

with open("src/simulation.rs", "r") as f:
    content = f.read()

# We broke the 20-element tuple limit for add_systems.
# Let's revert and use a new schedule.add_systems call.
content = content.replace("""        // Orbital Scrapyard Systems
        crate::layer2::orbital_scrapyard::process_salvage_missions,
        crate::layer2::orbital_scrapyard::check_deorbit_trigger,
        crate::layer2::orbital_scrapyard::process_debris_impact,
        // Debris Systems""", "// Debris Systems")

content = content.replace("    schedule.add_systems((", """    schedule.add_systems((
        crate::layer2::orbital_scrapyard::process_salvage_missions,
        crate::layer2::orbital_scrapyard::check_deorbit_trigger,
        crate::layer2::orbital_scrapyard::process_debris_impact,
    ));

    schedule.add_systems((""")

with open("src/simulation.rs", "w") as f:
    f.write(content)

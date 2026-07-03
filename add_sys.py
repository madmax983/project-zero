with open('src/simulation.rs', 'r') as f:
    content = f.read()

injection = """
    schedule.add_systems((
        crate::layer1::biology::chromotaxis::chromotaxis_attraction_system,
        crate::layer1::biology::chromotaxis::chromotaxis_aggro_system,
    ));"""

if "chromotaxis_attraction_system" not in content:
    content = content.replace("    schedule.add_systems((", injection + "\n    schedule.add_systems((", 1)

with open('src/simulation.rs', 'w') as f:
    f.write(content)

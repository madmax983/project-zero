with open("src/layer1/systems/observation.rs", "r") as f:
    for i, line in enumerate(f):
        if "aggro_network_entities" in line:
            print(f"Found on line {i}: {line.strip()}")

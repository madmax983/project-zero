content = open("src/layer1/drone.rs", "r").read()

content = content.replace("""        // 3. TODO: Haul Logic
        if action.current == ActionType::Idle {
            // Placeholder: Stay Idle
        }""", "")

with open("src/layer1/drone.rs", "w") as f:
    f.write(content)

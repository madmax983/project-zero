with open("src/simulation.rs") as f:
    for line in f:
        if "update_faction_satisfaction_system" in line:
            print(line)

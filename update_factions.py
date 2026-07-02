import sys

def main():
    file_path = 'src/layer1/social/factions.rs'
    with open(file_path, 'r') as f:
        content = f.read()

    new_faction = "    /// Faction dedicated to undoing mistakes of the past.\n    Penitent,\n"
    target = "    HiveMind,\n"

    if "Penitent," not in content:
        content = content.replace(target, target + new_faction)

    with open(file_path, 'w') as f:
        f.write(content)

if __name__ == "__main__":
    main()

import sys

def main():
    file_path = 'src/layer1/social/factions.rs'
    with open(file_path, 'r') as f:
        content = f.read()

    new_init = """            self.map.insert(
                FactionId::Penitent,
                FactionData {
                    name: "The Penitent".into(),
                    ..Default::default()
                },
            );
"""
    target = """            self.map.insert(
                FactionId::SubLithic,
                FactionData {
                    name: "The Sub-Lithic Cult".into(),
                    ..Default::default()
                },
            );
"""

    if "The Penitent" not in content:
        content = content.replace(target, target + new_init)

    with open(file_path, 'w') as f:
        f.write(content)

if __name__ == "__main__":
    main()

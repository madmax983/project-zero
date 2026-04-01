import re

with open('src/layer1/systems/environment.rs', 'r') as f:
    content = f.read()

# Add registration to environment.rs
new_systems = """
            crate::layer1::nature::megafauna_terrain::process_mining_titan_system,
            crate::layer1::nature::megafauna_terrain::awaken_titan_system
                .after(crate::layer1::nature::megafauna_terrain::process_mining_titan_system),
            crate::layer1::nature::seasons::update_season_system,
"""
content = content.replace("crate::layer1::nature::seasons::update_season_system,", new_systems)

with open('src/layer1/systems/environment.rs', 'w') as f:
    f.write(content)

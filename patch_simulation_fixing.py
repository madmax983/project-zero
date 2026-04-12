import re

with open("src/simulation.rs", "r") as f:
    content = f.read()

# Make sure we didn't accidentally delete DerelictStation stuff or others
assert "crate::layer2::derelict_stations::claim_station_system" in content
assert "crate::layer2::integration::derelict_station_chronicle_bridge" in content

import os

with open("src/ui/status.rs", "r") as f:
    content = f.read()

get_status_string_sig_old = """    efficiency: f32,
    season: Option<Season>,
    solar_cycle: Option<SolarCycle>,
) -> String {"""
get_status_string_sig_new = """    efficiency: f32,
    season: Option<Season>,
    solar_cycle: Option<SolarCycle>,
    risk_pct: f32,
) -> String {"""
content = content.replace(get_status_string_sig_old, get_status_string_sig_new)


with open("src/ui/status.rs", "w") as f:
    f.write(content)

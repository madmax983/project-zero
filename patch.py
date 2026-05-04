import sys

filename = 'src/layer1/systems/observation.rs'
with open(filename, 'r') as file:
    content = file.read()

content = content.replace(
    'crate::layer1::social::cadet::death_consequence_system,',
    'crate::layer1::social::cadet::death_consequence_system,\n            crate::layer1::social::golden_age::complacency_accumulation_system,\n            crate::layer1::social::golden_age::apply_complacency_debuffs_system,\n            crate::layer1::social::golden_age::pop_alert_response_system,'
)

with open(filename, 'w') as file:
    file.write(content)

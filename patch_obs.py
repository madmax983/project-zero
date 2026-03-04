import os

with open("src/layer1/systems/observation.rs", "r") as f:
    content = f.read()

# Fix cycle: death_consequence_system is scheduled before despawn_dead_entities_system,
# but maybe it depends on something that depends on despawn?
# Wait, the error is:
# system `despawn_dead_entities_system (in set Consumption)`
#  ... which must run before system `death_consequence_system (in set Observation)`
#  ... which must run before system `despawn_dead_entities_system (in set Consumption)`

content = content.replace("            crate::layer1::social::cadet::death_consequence_system\n                .before(crate::layer1::health::despawn_dead_entities_system),",
                          "            crate::layer1::social::cadet::death_consequence_system,")

with open("src/layer1/systems/observation.rs", "w") as f:
    f.write(content)

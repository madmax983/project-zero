import re

with open("src/layer1/core/integration.rs", "r") as f:
    content = f.read()

# Replace `crate::layer1::law::Policies` with `crate::layer1::administration::edicts::ColonyPolicies`
content = content.replace("crate::layer1::law::Policies", "crate::layer1::administration::edicts::ColonyPolicies")
content = content.replace("crate::layer1::law::Policy::Aesthetic", "crate::layer1::administration::edicts::Policy::Aesthetic")

with open("src/layer1/core/integration.rs", "w") as f:
    f.write(content)

with open("tests/integration/aesthetic_edict_chronicle_bridge.rs", "r") as f:
    content = f.read()

content = content.replace("scale::layer1::law::{Policies, Policy}", "scale::layer1::administration::edicts::{ColonyPolicies, Policy}")
content = content.replace("Policies::default()", "ColonyPolicies::default()")
content = content.replace("Policies>", "ColonyPolicies>")
content = content.replace("Policies>::default", "ColonyPolicies>::default")

with open("tests/integration/aesthetic_edict_chronicle_bridge.rs", "w") as f:
    f.write(content)

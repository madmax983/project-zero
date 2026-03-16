import re

with open("src/layer1/utility_types.rs", "r") as f:
    content = f.read()

# Replace /// See [`crate::layer1::actions::something::evaluate_something`].
# with /// See `crate::layer1::actions::something::evaluate_something`.
content = re.sub(r'\[`(crate::layer1::(?:actions|utility_eval_types|husbandry|justice|predictive_policing|memetic|chemical)::[^`]+)`\]', r'`\1`', content)

with open("src/layer1/utility_types.rs", "w") as f:
    f.write(content)

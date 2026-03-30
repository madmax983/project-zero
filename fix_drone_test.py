import re

with open("src/layer1/drone_tests.rs", "r") as f:
    content = f.read()

content = content.replace(
    "fn test_drones_deactivate_without_power_or_bandwidth() {",
    "fn test_drones_deactivate_without_power_or_bandwidth() {\n        use crate::layer1::utility_ai::StartPlan;\n"
)

with open("src/layer1/drone_tests.rs", "w") as f:
    f.write(content)

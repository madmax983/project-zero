import re

with open("src/layer1/tech/hypno_learning.rs", "r") as f:
    code = f.read()

code = code.replace(
    "Needs { hunger: 100.0, ..Default::default() }",
    "Needs { hunger: 100.0, rest: 100.0, leisure: 100.0, hygiene: 100.0 }"
)

with open("src/layer1/tech/hypno_learning.rs", "w") as f:
    f.write(code)

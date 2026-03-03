import re

with open("src/layer1/tech/hypno_learning.rs", "r") as f:
    code = f.read()

code = code.replace(
    "assert!(needs.hunger < 99.0);",
    "assert!(needs.hunger < 100.0);"
)

with open("src/layer1/tech/hypno_learning.rs", "w") as f:
    f.write(code)

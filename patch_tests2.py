import re

with open("src/layer1/tech/hypno_learning.rs", "r") as f:
    code = f.read()

code = code.replace(
    "let pod = world.spawn(HypnoPod::default()).id();",
    "let pod = world.spawn((HypnoPod::default(), PowerConsumer { active: true, demand: 10.0 })).id();"
)

with open("src/layer1/tech/hypno_learning.rs", "w") as f:
    f.write(code)

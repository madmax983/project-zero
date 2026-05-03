with open("src/layer1/actions/haul.rs", "r") as f:
    content = f.read()

content = content.replace("UtilityWeights {\n            distance_weight:", "UtilityWeights {\n            work: 1.0,\n            defend: 1.0,\n            distance_weight:")

with open("src/layer1/actions/haul.rs", "w") as f:
    f.write(content)

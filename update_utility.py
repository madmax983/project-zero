with open("src/layer1/mind/utility_types.rs", "r") as f:
    content = f.read()

content = content.replace("pub struct UtilityWeights {\n    /// How much distance", "pub struct UtilityWeights {\n    pub work: f32,\n    pub defend: f32,\n    /// How much distance")

content = content.replace("fn default() -> Self {\n        Self {\n            distance_weight:", "fn default() -> Self {\n        Self {\n            work: 1.0,\n            defend: 1.0,\n            distance_weight:")

with open("src/layer1/mind/utility_types.rs", "w") as f:
    f.write(content)

with open("src/layer1/psychology/sleepwalking_tests.rs", "r") as f:
    content = f.read()

content = content.replace("crate::layer1::mind::utility_types::UtilityWeights { work: 1.0, defend: 1.0, distance_weight: 1.0, availability_weight: 1.0 }", "crate::layer1::mind::utility_types::UtilityWeights::default()")

with open("src/layer1/psychology/sleepwalking_tests.rs", "w") as f:
    f.write(content)

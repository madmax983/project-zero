import re

files = [
    "src/gpu/buffers.rs",
    "src/layer1/mind/utility_eval_types.rs",
    "benches/utility_ai_bench.rs",
    "tests/integration/sanctuary_ai.rs"
]

for file in files:
    with open(file, "r") as f:
        content = f.read()

    # Replace `UtilityWeights {` with `UtilityWeights { work: 1.0, defend: 1.0,`
    # But only if it doesn't already have work and defend.
    # We will just do a regex replace on `UtilityWeights\s*\{`
    # and then clean up any duplicates if they exist, but it's simpler to just be careful.

    # First, let's just find where distance_weight or availability_weight is being set and insert work/defend there.
    content = re.sub(r'(UtilityWeights\s*\{)(?!\s*work:)', r'\1\n                work: 1.0,\n                defend: 1.0,', content)

    with open(file, "w") as f:
        f.write(content)

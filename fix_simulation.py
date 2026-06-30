import re

with open("src/simulation.rs", "r") as f:
    content = f.read()

# I will find the EXACT end of `register_simulation_extended_systems`.
pattern = r"(fn register_simulation_extended_systems\([^)]+\)\s*\{.*?)(    \)\);\n)(\})"
new_call = '    #[cfg(feature = "nova")]\n    crate::experimental::subconscious_computing::register(schedule);\n'

new_content = re.sub(pattern, r"\1\2" + new_call + r"\3", content, flags=re.DOTALL)

with open("src/simulation.rs", "w") as f:
    f.write(new_content)

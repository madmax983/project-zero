import os
import glob
import re

MOVED_TYPES = [
    "ActionType", "PopAction", "UtilityWeights", "PlanOutcome", "UtilityConfig",
    "ColonyMemory", "StartPlan", "need_response_curve",
    "manhattan_distance", "calculate_context_score", "calculate_success_modifier",
    "evaluate_idle"
]

def main():
    files = glob.glob('src/**/*.rs', recursive=True)
    for filepath in files:
        if filepath == "src/layer1/utility_ai.rs" or filepath == "src/layer1/utility_types.rs":
            continue

        with open(filepath, 'r') as f:
            content = f.read()

        new_content = content
        for type_name in MOVED_TYPES:
            # Replace crate::layer1::utility_ai::Type with crate::layer1::utility_types::Type
            pattern = f"crate::layer1::utility_ai::{type_name}"
            replacement = f"crate::layer1::utility_types::{type_name}"
            new_content = new_content.replace(pattern, replacement)

            # Also check for use crate::layer1::utility_ai::{..., Type, ...} pattern?
            # Rust allows import splitting, but simple find/replace might miss it if it's complex.
            # But the error logs show specific full paths being used.

        if new_content != content:
            with open(filepath, 'w') as f:
                f.write(new_content)
            print(f"Updated {filepath}")

if __name__ == "__main__":
    main()

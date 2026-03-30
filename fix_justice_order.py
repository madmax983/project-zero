with open("src/layer1/justice.rs", "r") as f:
    code = f.read()

# Separate imports, production code, and test module.
# The code is currently:
# [original production]
# [test module]
# [new imports]
# [new production]

import re

# Find the test module
test_mod_match = re.search(r'#\[cfg\(test\)\]\s*mod tests \{.*?(?=\nuse crate::layer1::black_market|\Z)', code, flags=re.DOTALL)
if test_mod_match:
    test_mod = test_mod_match.group(0)

    # Everything before the test module
    before_test = code[:test_mod_match.start()]

    # Everything after the test module (the new code)
    after_test = code[test_mod_match.end():]

    # New code should go before the test module
    new_code = before_test + after_test + "\n" + test_mod + "\n"

    with open("src/layer1/justice.rs", "w") as f:
        f.write(new_code)

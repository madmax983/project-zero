with open("src/bin/headless.rs", "r") as f:
    code = f.read()

import re

# Look for print_log. The level indicators are already ERR, WRN, etc.
# We will just change them to have icons: ❌ ERR, ⚠️ WRN, ✅ OK , ℹ️ INF, 📜 LOG
# And change to_comfy_color mapping just in case.

code = code.replace('"ERR"', '"❌ ERR"')
code = code.replace('"WRN"', '"⚠️ WRN"')
code = code.replace('"OK "', '"✅ OK "')
code = code.replace('"INF"', '"ℹ️ INF"')
code = code.replace('"LOG"', '"📜 LOG"')

with open("src/bin/headless.rs", "w") as f:
    f.write(code)

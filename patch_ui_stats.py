import os

with open("tests/integration/ui_stats.rs", "r") as f:
    content = f.read()

content = content.replace(
    """            None, // Solar Cycle
        );""",
    """            None, // Solar Cycle
            0.0, // risk_pct
        );"""
)

with open("tests/integration/ui_stats.rs", "w") as f:
    f.write(content)

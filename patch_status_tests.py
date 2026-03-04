import os

with open("src/ui/status.rs", "r") as f:
    content = f.read()

content = content.replace(
    """            None,  // Solar Cycle
        );""",
    """            None,  // Solar Cycle
            0.0,   // risk_pct
        );"""
)

content = content.replace(
    """            None, // Solar Cycle
        );""",
    """            None, // Solar Cycle
            0.0,   // risk_pct
        );"""
)

content = content.replace(
    """            None,
        );""",
    """            None,
            0.0,   // risk_pct
        );"""
)

content = content.replace(
    """            Some(SolarCycle::Maximum),
        );""",
    """            Some(SolarCycle::Maximum),
            0.0,   // risk_pct
        );"""
)

with open("src/ui/status.rs", "w") as f:
    f.write(content)

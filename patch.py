import re

with open("src/layer1/systems/economy.rs", "r") as f:
    content = f.read()

# Add to the big schedule.add_systems block that contains other economy systems
target = "crate::layer1::economy::smugglers_cove::process_smuggler_decay_system,"
replacement = """crate::layer1::economy::smugglers_cove::process_smuggler_decay_system,
            crate::layer1::economy::black_market::black_market_spawn_system,
            crate::layer1::economy::black_market::smuggler_trade_system,
            crate::layer1::economy::black_market::handle_smuggler_arrival,
            crate::layer1::economy::black_market::pop_smuggling_system,
            crate::layer1::economy::black_market::shutdown_drop_node_system,"""

if target in content:
    content = content.replace(target, replacement)
    with open("src/layer1/systems/economy.rs", "w") as f:
        f.write(content)
    print("Patched successfully")
else:
    print("Target not found")

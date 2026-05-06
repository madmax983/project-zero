import re

with open("src/bin/headless.rs", "r") as f:
    code = f.read()

# Fix print_log again to make sure everything was formatted properly using sed initially
code = code.replace('"❌ ERR"', '"ERR"')
code = code.replace('"⚠️ WRN"', '"WRN"')
code = code.replace('"✅ OK "', '"OK "')
code = code.replace('"ℹ️ INF"', '"INF"')
code = code.replace('"📜 LOG"', '"LOG"')

# Better to use comfy_table modifiers
log_code_search = """        let color = to_comfy_color(msg.color);
        let level_indicator = match msg.color {
            ratatui::style::Color::Red | ratatui::style::Color::LightRed => "ERR",
            ratatui::style::Color::Yellow | ratatui::style::Color::LightYellow => "WRN",
            ratatui::style::Color::Green | ratatui::style::Color::LightGreen => "OK ",
            ratatui::style::Color::Cyan | ratatui::style::Color::LightCyan => "INF",
            _ => "LOG",
        };"""

log_code_replace = """        let color = to_comfy_color(msg.color);
        let level_indicator = match msg.color {
            ratatui::style::Color::Red | ratatui::style::Color::LightRed => "❌ ERR",
            ratatui::style::Color::Yellow | ratatui::style::Color::LightYellow => "⚠️ WRN",
            ratatui::style::Color::Green | ratatui::style::Color::LightGreen => "✅ OK ",
            ratatui::style::Color::Cyan | ratatui::style::Color::LightCyan => "ℹ️ INF",
            _ => "📜 LOG",
        };"""

code = code.replace(log_code_search, log_code_replace)

with open("src/bin/headless.rs", "w") as f:
    f.write(code)

content = open("tests/integration.rs", "r").read()
content = content.replace("#[path = \"integration/ip_piracy_diplomacy.rs\"]\n#[path = \"integration/phantom_signal.rs\"]\nmod phantom_signal;", "#[path = \"integration/ip_piracy_diplomacy.rs\"]\nmod ip_piracy_diplomacy;\n\n#[path = \"integration/phantom_signal.rs\"]\nmod phantom_signal;")
open("tests/integration.rs", "w").write(content)

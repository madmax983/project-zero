with open("src/layer2/mod.rs", "r") as f:
    content = f.read()

content = content.replace("pub mod piracy;", "pub mod piracy;\npub use piracy::*;")

with open("src/layer2/mod.rs", "w") as f:
    f.write(content)

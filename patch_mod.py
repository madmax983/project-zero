with open("src/layer1/social/mod.rs", "r") as f:
    content = f.read()

content = content.replace("pub mod debt;", "pub mod debt;\npub mod echoes;")

with open("src/layer1/social/mod.rs", "w") as f:
    f.write(content)

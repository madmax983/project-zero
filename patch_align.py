with open('src/layer2/mod.rs', 'r') as f:
    content = f.read()
if "pub mod alignment;" not in content:
    content = content.replace("pub mod syzygy;", "pub mod alignment;\npub mod syzygy;")
with open('src/layer2/mod.rs', 'w') as f:
    f.write(content)

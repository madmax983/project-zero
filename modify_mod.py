import re

with open("src/layer1/biology/mod.rs", "r") as f:
    content = f.read()

content = content.replace("pub mod medical;\n", "pub mod medical;\npub mod rust_lung;\n")
content = content.replace("pub use medical::*;\n", "pub use medical::*;\npub use rust_lung::*;\n")

with open("src/layer1/biology/mod.rs", "w") as f:
    f.write(content)

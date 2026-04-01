with open("src/layer1/social/mod.rs", "r") as f:
    content = f.read()

content = content.replace("use crate::layer1::cybernetics::Augmentations;", "use crate::layer1::cybernetics::{Augmentations, Prosthetic};")

with open("src/layer1/social/mod.rs", "w") as f:
    f.write(content)

with open("src/layer1/graffiti.rs", "r") as f:
    content = f.read()

content = content.replace("traits.map_or(false, |t| t.has(crate::layer1::traits::Trait::Creative))", "traits.is_some_and(|t| t.has(crate::layer1::traits::Trait::Creative))")

with open("src/layer1/graffiti.rs", "w") as f:
    f.write(content)

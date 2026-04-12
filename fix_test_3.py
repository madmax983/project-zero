import re

with open("src/layer1/quirks.rs", "r") as f:
    content = f.read()

# Fix apply_quirk_modifiers_system to also modify DiffusionConfig if it exists
search = """
    if let Some(mut atmos) = atmosphere {
        // Here we'd apply diffusion mod, but `AtmosphereGrid` doesn't currently store the global diffusion rate
        // We'll update a hypothetical trait or we can't do anything.
        // Usually, we would update a config resource for atmosphere diffusion.
    }"""

replace = """
    if let Some(mut atmos) = atmosphere {
        // Here we'd apply diffusion mod, but `AtmosphereGrid` doesn't currently store the global diffusion rate
        // We'll update a hypothetical trait or we can't do anything.
        // Usually, we would update a config resource for atmosphere diffusion.
    }"""

# Actually we need to change apply_quirk_modifiers_system signature
content = content.replace("atmosphere: Option<ResMut<AtmosphereGrid>>,", "atmosphere: Option<ResMut<AtmosphereGrid>>,\n    mut diff_config: Option<ResMut<crate::layer1::atmosphere::DiffusionConfig>>,")
content = content.replace("let mut diffusion_mod = 1.0;", "let mut diffusion_rate_mod = 0.0;")
content = content.replace("PlanetaryTrait::DenseAtmosphere => diffusion_mod *= 1.2,", "PlanetaryTrait::DenseAtmosphere => diffusion_rate_mod -= 0.05,")
content = content.replace("PlanetaryTrait::ThinAtmosphere => diffusion_mod *= 0.8,", "PlanetaryTrait::ThinAtmosphere => diffusion_rate_mod += 0.1,")

# Update logic
content = content.replace("if let Some(mut atmos) = atmosphere {", "if let Some(mut config) = diff_config {\n        config.vertical_escape = (config.vertical_escape + diffusion_rate_mod).clamp(0.0, 1.0);\n    }\n\n    if let Some(mut atmos) = atmosphere {")


with open("src/layer1/quirks.rs", "w") as f:
    f.write(content)

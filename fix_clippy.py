with open("src/layer1/fauna/mod.rs", "r") as f:
    content = f.read()

# Add #[allow(clippy::too_many_arguments)]
content = content.replace("#[allow(clippy::cast_precision_loss)]\npub fn fauna_behavior_system", "#[allow(clippy::cast_precision_loss, clippy::too_many_arguments)]\npub fn fauna_behavior_system")

with open("src/layer1/fauna/mod.rs", "w") as f:
    f.write(content)

with open("tests/integration/edible_architecture.rs", "r") as f:
    content = f.read()

content = content.replace("    app.add_systems(\n        Update,\n        (\n            scale::layer1::architecture::edible::consume_building_system,\n            scale::layer1::core::integration::edible_architecture_chronicle_bridge,\n        ),\n    );", "    app.insert_resource(scale::layer1::economy::resources::ColonyResources { food: 10.0, ..Default::default() });\n    app.add_systems(\n        Update,\n        (\n            scale::layer1::architecture::edible::consume_building_system,\n            scale::layer1::core::integration::edible_architecture_chronicle_bridge,\n        ),\n    );")

with open("tests/integration/edible_architecture.rs", "w") as f:
    f.write(content)

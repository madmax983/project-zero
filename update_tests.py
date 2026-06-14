import re

with open('tests/integration/edible_architecture.rs', 'r') as f:
    content = f.read()

content = content.replace("let _building = app", "let building = app")
content = re.sub(
    r"let building = app\n        \.world_mut\(\)\n        \.spawn\(\(\n            scale::layer1::architecture::building::Building \{\n                building_type: scale::layer1::architecture::building::BuildingType::Housing,\n            \},\n            scale::layer1::map::GridPosition \{ x: 5, y: 5 \},\n            scale::layer1::architecture::edible::EdibleMaterial \{ food_yield: 50.0 \},\n            scale::layer1::architecture::edible::Consumed,\n        \)\)\n        \.id\(\);",
    "let _building = app\n        .world_mut()\n        .spawn((\n            scale::layer1::architecture::building::Building {\n                building_type: scale::layer1::architecture::building::BuildingType::Housing,\n            },\n            scale::layer1::map::GridPosition { x: 5, y: 5 },\n            scale::layer1::architecture::edible::EdibleMaterial { food_yield: 50.0 },\n            scale::layer1::architecture::edible::Consumed,\n        ))\n        .id();",
    content,
    flags=re.MULTILINE
)

with open('tests/integration/edible_architecture.rs', 'w') as f:
    f.write(content)

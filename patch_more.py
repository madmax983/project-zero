import re
import sys

def replace_all(file_path):
    with open(file_path, 'r') as f:
        content = f.read()

    # trade.rs
    content = content.replace('ResourceType::HyperValuable => false,', 'ResourceType::HyperValuable => false,\n            ResourceType::BiologicalWaste | ResourceType::NutrientPaste => false,')
    content = content.replace('ResourceType::HyperValuable => {}', 'ResourceType::HyperValuable => {}\n            ResourceType::BiologicalWaste | ResourceType::NutrientPaste => {}')

    # ephemeral_market.rs
    content = content.replace('ResourceType::VoidAle | ResourceType::HyperValuable => false,', 'ResourceType::VoidAle | ResourceType::HyperValuable | ResourceType::BiologicalWaste | ResourceType::NutrientPaste => false,')
    content = content.replace('ResourceType::VoidAle | ResourceType::HyperValuable => {}', 'ResourceType::VoidAle | ResourceType::HyperValuable | ResourceType::BiologicalWaste | ResourceType::NutrientPaste => {}')

    # map.rs
    content = content.replace('ResourceType::HyperValuable => "$",', 'ResourceType::HyperValuable => "$",\n        ResourceType::BiologicalWaste => "B",\n        ResourceType::NutrientPaste => "P",')
    content = content.replace('ResourceType::HyperValuable => Color::Rgb(255, 215, 0),', 'ResourceType::HyperValuable => Color::Rgb(255, 215, 0),\n        ResourceType::BiologicalWaste => Color::Rgb(100, 100, 0),\n        ResourceType::NutrientPaste => Color::Rgb(200, 200, 200),')

    with open(file_path, 'w') as f:
        f.write(content)

replace_all('src/layer1/economy/trade.rs')
replace_all('src/layer3/market/ephemeral_market.rs')
replace_all('src/ui/map.rs')

# Additional fixes that might be needed in Cargo checking

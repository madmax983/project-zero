import re

def replace_all(file_path):
    with open(file_path, 'r') as f:
        content = f.read()

    # Remaining max_ fields in new blocks
    content = content.replace('max_waste: 0.0,\n            max_rations', 'max_waste: 0.0,\n            max_biological_waste: 0.0,\n            max_nutrient_paste: 0.0,\n            max_rations')
    content = content.replace('waste: 0.0,\n            rations: 0.0', 'waste: 0.0,\n            biological_waste: 0.0,\n            nutrient_paste: 0.0,\n            rations: 0.0')

    # Fix add_amount (lines 755/884)
    content = content.replace('ResourceType::Waste => self.waste = (self.waste + amount).min(self.max_waste),\n            }', 'ResourceType::Waste => self.waste = (self.waste + amount).min(self.max_waste),\n            ResourceType::BiologicalWaste => self.biological_waste = (self.biological_waste + amount).min(self.max_biological_waste),\n            ResourceType::NutrientPaste => self.nutrient_paste = (self.nutrient_paste + amount).min(self.max_nutrient_paste),\n            }')

    content = content.replace('ResourceType::Waste => self.waste += amount,\n            }', 'ResourceType::Waste => self.waste += amount,\n            ResourceType::BiologicalWaste => self.biological_waste += amount,\n            ResourceType::NutrientPaste => self.nutrient_paste += amount,\n            }')

    with open(file_path, 'w') as f:
        f.write(content)

replace_all('src/layer1/economy/resources.rs')

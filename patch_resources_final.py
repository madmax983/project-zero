import re

def replace_all(file_path):
    with open(file_path, 'r') as f:
        content = f.read()

    # default
    content = content.replace('max_waste: 1000.0,\n            max_rations', 'max_waste: 1000.0,\n            max_biological_waste: 1000.0,\n            max_nutrient_paste: 1000.0,\n            max_rations')

    # mul
    content = content.replace('waste: (self.waste * rhs).ceil(),\n            rations:', 'waste: (self.waste * rhs).ceil(),\n            biological_waste: (self.biological_waste * rhs).ceil(),\n            nutrient_paste: (self.nutrient_paste * rhs).ceil(),\n            rations:')
    content = content.replace('max_waste: self.max_waste,\n            max_rations:', 'max_waste: self.max_waste,\n            max_biological_waste: self.max_biological_waste,\n            max_nutrient_paste: self.max_nutrient_paste,\n            max_rations:')

    # zeroed
    content = content.replace('max_waste: 0.0,\n            max_rations', 'max_waste: 0.0,\n            max_biological_waste: 0.0,\n            max_nutrient_paste: 0.0,\n            max_rations')

    # consume_resources
    content = content.replace('ResourceType::Waste => self.waste = (self.waste - amount).max(0.0),\n            }', 'ResourceType::Waste => self.waste = (self.waste - amount).max(0.0),\n            ResourceType::BiologicalWaste => self.biological_waste = (self.biological_waste - amount).max(0.0),\n            ResourceType::NutrientPaste => self.nutrient_paste = (self.nutrient_paste - amount).max(0.0),\n            }')

    # add_resource_amount
    content = content.replace('ResourceType::Waste => self.waste += amount,\n            }', 'ResourceType::Waste => self.waste += amount,\n            ResourceType::BiologicalWaste => self.biological_waste += amount,\n            ResourceType::NutrientPaste => self.nutrient_paste += amount,\n            }')

    # remaining max_amount
    content = content.replace('ResourceType::Waste => self.max_waste,\n            }', 'ResourceType::Waste => self.max_waste,\n            ResourceType::BiologicalWaste => self.max_biological_waste,\n            ResourceType::NutrientPaste => self.max_nutrient_paste,\n            }')

    # check remaining matches
    content = content.replace('ResourceType::Waste => self.waste,\n            }', 'ResourceType::Waste => self.waste,\n            ResourceType::BiologicalWaste => self.biological_waste,\n            ResourceType::NutrientPaste => self.nutrient_paste,\n            }')

    with open(file_path, 'w') as f:
        f.write(content)

replace_all('src/layer1/economy/resources.rs')

with open('src/layer1/economy/resources.rs', 'r') as f:
    content = f.read()

# Fix default initializer for max_biological_waste and max_nutrient_paste
content = content.replace('max_waste: 0.0, // Defaults to 0, requires Landfill\n            max_rations', 'max_waste: 0.0, // Defaults to 0, requires Landfill\n            max_biological_waste: 0.0,\n            max_nutrient_paste: 0.0,\n            max_rations')

content = content.replace('ResourceType::Waste => self.max_waste,\n            ResourceType::Rations', 'ResourceType::Waste => self.max_waste,\n            ResourceType::BiologicalWaste => self.max_biological_waste,\n            ResourceType::NutrientPaste => self.max_nutrient_paste,\n            ResourceType::Rations')

content = content.replace('ResourceType::Waste => self.waste,\n            ResourceType::Rations', 'ResourceType::Waste => self.waste,\n            ResourceType::BiologicalWaste => self.biological_waste,\n            ResourceType::NutrientPaste => self.nutrient_paste,\n            ResourceType::Rations')

with open('src/layer1/economy/resources.rs', 'w') as f:
    f.write(content)

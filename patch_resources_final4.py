with open('src/layer1/economy/resources.rs', 'r') as f:
    content = f.read()

content = content.replace('ResourceType::Waste => self.add_waste(amount),\n            ResourceType::Rations', 'ResourceType::Waste => self.add_waste(amount),\n            ResourceType::BiologicalWaste => self.biological_waste = (self.biological_waste + amount).min(self.max_biological_waste),\n            ResourceType::NutrientPaste => self.nutrient_paste = (self.nutrient_paste + amount).min(self.max_nutrient_paste),\n            ResourceType::Rations')

with open('src/layer1/economy/resources.rs', 'w') as f:
    f.write(content)

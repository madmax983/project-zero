with open('src/layer1/economy/resources.rs', 'r') as f:
    content = f.read()

content = content.replace('ResourceType::Waste => self.waste = (self.waste - amount).max(0.0),\n            ResourceType::BuildingPermit', 'ResourceType::Waste => self.waste = (self.waste - amount).max(0.0),\n            ResourceType::BiologicalWaste => self.biological_waste = (self.biological_waste - amount).max(0.0),\n            ResourceType::NutrientPaste => self.nutrient_paste = (self.nutrient_paste - amount).max(0.0),\n            ResourceType::BuildingPermit')

content = content.replace('ResourceType::Waste => self.waste += amount,\n            ResourceType::BuildingPermit', 'ResourceType::Waste => self.waste += amount,\n            ResourceType::BiologicalWaste => self.biological_waste += amount,\n            ResourceType::NutrientPaste => self.nutrient_paste += amount,\n            ResourceType::BuildingPermit')

content = content.replace('ResourceType::Waste => self.waste = (self.waste + amount).min(self.max_waste),\n            ResourceType::BuildingPermit', 'ResourceType::Waste => self.waste = (self.waste + amount).min(self.max_waste),\n            ResourceType::BiologicalWaste => self.biological_waste = (self.biological_waste + amount).min(self.max_biological_waste),\n            ResourceType::NutrientPaste => self.nutrient_paste = (self.nutrient_paste + amount).min(self.max_nutrient_paste),\n            ResourceType::BuildingPermit')

with open('src/layer1/economy/resources.rs', 'w') as f:
    f.write(content)

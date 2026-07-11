import re

with open('src/layer1/economy/resources.rs', 'r') as f:
    content = f.read()

# Add to ResourceType enum
if 'BiologicalWaste,' not in content:
    content = content.replace('Waste,', 'Waste,\n    BiologicalWaste,\n    NutrientPaste,')

# Add max fields to ColonyResources
if 'max_biological_waste: f32,' not in content:
    content = content.replace('pub max_waste: f32,', 'pub max_waste: f32,\n    pub max_biological_waste: f32,\n    pub max_nutrient_paste: f32,')

# Add actual fields to ColonyResources
if 'biological_waste: f32,' not in content:
    content = content.replace('pub waste: f32,', 'pub waste: f32,\n    pub biological_waste: f32,\n    pub nutrient_paste: f32,')

# Add to Default impl for ColonyResources
if 'biological_waste: 0.0,' not in content:
    content = content.replace('waste: 0.0,', 'waste: 0.0,\n            biological_waste: 0.0,\n            nutrient_paste: 0.0,')
if 'max_biological_waste: 1000.0,' not in content:
    content = content.replace('max_waste: 1000.0,', 'max_waste: 1000.0,\n            max_biological_waste: 1000.0,\n            max_nutrient_paste: 1000.0,')

# Add to match in get_amount
if 'ResourceType::BiologicalWaste => self.biological_waste,' not in content:
    content = content.replace('ResourceType::Waste => self.waste,', 'ResourceType::Waste => self.waste,\n            ResourceType::BiologicalWaste => self.biological_waste,\n            ResourceType::NutrientPaste => self.nutrient_paste,')

# Add to match in max_amount
if 'ResourceType::BiologicalWaste => self.max_biological_waste,' not in content:
    content = content.replace('ResourceType::Waste => self.max_waste,', 'ResourceType::Waste => self.max_waste,\n            ResourceType::BiologicalWaste => self.max_biological_waste,\n            ResourceType::NutrientPaste => self.max_nutrient_paste,')

# Add to match in consume
if 'ResourceType::BiologicalWaste => self.biological_waste = (self.biological_waste - amount).max(0.0),' not in content:
    content = content.replace('ResourceType::Waste => self.waste = (self.waste - amount).max(0.0),', 'ResourceType::Waste => self.waste = (self.waste - amount).max(0.0),\n            ResourceType::BiologicalWaste => self.biological_waste = (self.biological_waste - amount).max(0.0),\n            ResourceType::NutrientPaste => self.nutrient_paste = (self.nutrient_paste - amount).max(0.0),')

# Add to match in has_room_for
if 'ResourceType::BiologicalWaste => self.biological_waste < self.max_biological_waste,' not in content:
    content = content.replace('ResourceType::Waste => self.waste < self.max_waste,', 'ResourceType::Waste => self.waste < self.max_waste,\n            ResourceType::BiologicalWaste => self.biological_waste < self.max_biological_waste,\n            ResourceType::NutrientPaste => self.nutrient_paste < self.max_nutrient_paste,')

# Add to match in add_resource
if 'ResourceType::BiologicalWaste => self.biological_waste = (self.biological_waste + amount).min(self.max_biological_waste),' not in content:
    content = content.replace('ResourceType::Waste => self.waste = (self.waste + amount).min(self.max_waste),', 'ResourceType::Waste => self.waste = (self.waste + amount).min(self.max_waste),\n            ResourceType::BiologicalWaste => self.biological_waste = (self.biological_waste + amount).min(self.max_biological_waste),\n            ResourceType::NutrientPaste => self.nutrient_paste = (self.nutrient_paste + amount).min(self.max_nutrient_paste),')

with open('src/layer1/economy/resources.rs', 'w') as f:
    f.write(content)

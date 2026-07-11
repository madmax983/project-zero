import re

def replace_all(file_path):
    with open(file_path, 'r') as f:
        content = f.read()

    # ResourceType enum
    content = content.replace('Waste,\n    ///', 'Waste,\n    BiologicalWaste,\n    NutrientPaste,\n    ///')

    # ColonyResources struct
    content = content.replace('pub waste: f32,\n    ///', 'pub waste: f32,\n    pub biological_waste: f32,\n    pub nutrient_paste: f32,\n    pub max_biological_waste: f32,\n    pub max_nutrient_paste: f32,\n    ///')

    # ColonyResources default
    content = content.replace('waste: 0.0,\n            rations:', 'waste: 0.0,\n            biological_waste: 0.0,\n            nutrient_paste: 0.0,\n            rations:')
    content = content.replace('max_waste: 1000.0,\n            max_rations:', 'max_waste: 1000.0,\n            max_biological_waste: 1000.0,\n            max_nutrient_paste: 1000.0,\n            max_rations:')

    # get_amount
    content = content.replace('ResourceType::Waste => self.waste,\n            ResourceType::Rations', 'ResourceType::Waste => self.waste,\n            ResourceType::BiologicalWaste => self.biological_waste,\n            ResourceType::NutrientPaste => self.nutrient_paste,\n            ResourceType::Rations')

    # max_amount
    content = content.replace('ResourceType::Waste => self.max_waste,\n            ResourceType::Rations', 'ResourceType::Waste => self.max_waste,\n            ResourceType::BiologicalWaste => self.max_biological_waste,\n            ResourceType::NutrientPaste => self.max_nutrient_paste,\n            ResourceType::Rations')

    # consume
    content = content.replace('ResourceType::Waste => self.waste = (self.waste - amount).max(0.0),\n            ResourceType::Rations', 'ResourceType::Waste => self.waste = (self.waste - amount).max(0.0),\n            ResourceType::BiologicalWaste => self.biological_waste = (self.biological_waste - amount).max(0.0),\n            ResourceType::NutrientPaste => self.nutrient_paste = (self.nutrient_paste - amount).max(0.0),\n            ResourceType::Rations')

    # has_room_for
    content = content.replace('ResourceType::Waste => self.waste < self.max_waste,\n            ResourceType::Rations', 'ResourceType::Waste => self.waste < self.max_waste,\n            ResourceType::BiologicalWaste => self.biological_waste < self.max_biological_waste,\n            ResourceType::NutrientPaste => self.nutrient_paste < self.max_nutrient_paste,\n            ResourceType::Rations')

    # add_resource
    content = content.replace('ResourceType::Waste => self.waste = (self.waste + amount).min(self.max_waste),\n            ResourceType::Rations', 'ResourceType::Waste => self.waste = (self.waste + amount).min(self.max_waste),\n            ResourceType::BiologicalWaste => self.biological_waste = (self.biological_waste + amount).min(self.max_biological_waste),\n            ResourceType::NutrientPaste => self.nutrient_paste = (self.nutrient_paste + amount).min(self.max_nutrient_paste),\n            ResourceType::Rations')

    # add_amount (if exists) - not required if we use add_resource

    # ResourceRates struct
    if 'pub waste: f32,' in content and 'max_waste' not in content[content.find('pub waste: f32,')-30:content.find('pub waste: f32,')]:
        content = content.replace('pub waste: f32,\n    ///', 'pub waste: f32,\n    pub biological_waste: f32,\n    pub nutrient_paste: f32,\n    ///')

    # ResourceRates default
    content = content.replace('waste: 0.0,\n            rations:', 'waste: 0.0,\n            biological_waste: 0.0,\n            nutrient_paste: 0.0,\n            rations:')

    with open(file_path, 'w') as f:
        f.write(content)

replace_all('src/layer1/economy/resources.rs')

def replace_all_other(file_path, replacements):
    with open(file_path, 'r') as f:
        content = f.read()
    for o, n in replacements:
        content = content.replace(o, n)
    with open(file_path, 'w') as f:
        f.write(content)

replace_all_other('src/layer1/economy/trade.rs', [
    ('ResourceType::HyperValuable => false,', 'ResourceType::HyperValuable => false,\n            ResourceType::BiologicalWaste | ResourceType::NutrientPaste => false,'),
    ('ResourceType::HyperValuable => {}', 'ResourceType::HyperValuable => {},\n            ResourceType::BiologicalWaste | ResourceType::NutrientPaste => {}'),
])

replace_all_other('src/layer3/market/ephemeral_market.rs', [
    ('ResourceType::VoidAle | ResourceType::HyperValuable => false,', 'ResourceType::VoidAle | ResourceType::HyperValuable | ResourceType::BiologicalWaste | ResourceType::NutrientPaste => false,'),
    ('ResourceType::VoidAle | ResourceType::HyperValuable => {}', 'ResourceType::VoidAle | ResourceType::HyperValuable | ResourceType::BiologicalWaste | ResourceType::NutrientPaste => {}'),
])

replace_all_other('src/ui/map.rs', [
    ('ResourceType::HyperValuable => "$",', 'ResourceType::HyperValuable => "$",\n        ResourceType::BiologicalWaste => "B",\n        ResourceType::NutrientPaste => "P",'),
    ('ResourceType::HyperValuable => Color::Rgb(255, 215, 0),', 'ResourceType::HyperValuable => Color::Rgb(255, 215, 0),\n        ResourceType::BiologicalWaste => Color::Rgb(100, 100, 0),\n        ResourceType::NutrientPaste => Color::Rgb(200, 200, 200),')
])

replace_all_other('src/gpu/buffers.rs', [
    ('ResourceType::HyperValuable => true,', 'ResourceType::HyperValuable => true,\n            ResourceType::BiologicalWaste | ResourceType::NutrientPaste => false,')
])

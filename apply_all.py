import sys

# Update src/layer1/economy/items.rs
with open('src/layer1/economy/items.rs', 'r') as f:
    content = f.read()

content = content.replace("    AgonyExtract,", "    AgonyExtract,\n    /// Fermented luxury good from orbital stations.\n    VoidAle,")
content = content.replace("            | Self::MysteryMeal => Some(ResourceType::Food),", "            | Self::MysteryMeal\n            | Self::VoidAle => Some(ResourceType::Food),")

with open('src/layer1/economy/items.rs', 'w') as f:
    f.write(content)

print("Updated items.rs")

# Update src/layer1/economy/resources.rs
with open('src/layer1/economy/resources.rs', 'r') as f:
    content = f.read()

content = content.replace("    MemoryCore,", "    MemoryCore,\n    /// Void-Ale.\n    VoidAle,")
content = content.replace("    pub max_memory_cores: f32,", "    pub max_memory_cores: f32,\n    /// Total Void-Ale available.\n    pub void_ale: f32,\n    /// Max Void-Ale.\n    pub max_void_ale: f32,")
content = content.replace("            max_memory_cores: 50.0,", "            max_memory_cores: 50.0,\n            void_ale: 0.0,\n            max_void_ale: 50.0,")
content = content.replace("            max_memory_cores: 0.0,", "            max_memory_cores: 0.0,\n            void_ale: 0.0,\n            max_void_ale: 0.0,")
content = content.replace("            max_memory_cores: self.max_memory_cores,", "            max_memory_cores: self.max_memory_cores,\n            void_ale: (self.void_ale * rhs).ceil(),\n            max_void_ale: self.max_void_ale,")

add_func = """
    /// Adds void ale, clamping to the maximum capacity.
    pub fn add_void_ale(&mut self, amount: f32) {
        if amount.is_finite() {
            self.void_ale = (self.void_ale + amount).clamp(0.0, self.max_void_ale);
        }
    }
"""
content = content.replace("    /// Returns the total food available (food aggregate + rations).", add_func + "    /// Returns the total food available (food aggregate + rations).")

content = content.replace("            || self.memory_cores < 0.0", "            || self.memory_cores < 0.0\n            || self.void_ale < 0.0")
content = content.replace("            && self.memory_cores.is_finite()", "            && self.memory_cores.is_finite()\n            && self.void_ale.is_finite()")
content = content.replace("            && self.memory_cores >= cost.memory_cores", "            && self.memory_cores >= cost.memory_cores\n            && self.void_ale >= cost.void_ale")
content = content.replace("        self.memory_cores -= cost.memory_cores;", "        self.memory_cores -= cost.memory_cores;\n        self.void_ale -= cost.void_ale;")
content = content.replace("            ResourceType::MemoryCore => {", "            ResourceType::VoidAle => self.void_ale = (self.void_ale - amount).max(0.0),\n            ResourceType::MemoryCore => {")
content = content.replace("            ResourceType::MemoryCore => self.memory_cores < self.max_memory_cores,", "            ResourceType::MemoryCore => self.memory_cores < self.max_memory_cores,\n            ResourceType::VoidAle => self.void_ale < self.max_void_ale,")
content = content.replace("            ResourceType::MemoryCore => self.add_memory_cores(amount),", "            ResourceType::MemoryCore => self.add_memory_cores(amount),\n            ResourceType::VoidAle => self.add_void_ale(amount),")

with open('src/layer1/economy/resources.rs', 'w') as f:
    f.write(content)

print("Updated resources.rs")

# Update src/layer2/system.rs
with open('src/layer2/system.rs', 'r') as f:
    content = f.read()

gravity_level = """
/// Gravity level of an orbital body or station.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GravityLevel {
    /// Normal planetary gravity
    #[default]
    Normal,
    /// Microgravity (e.g. moons, asteroids)
    MicroGravity,
    /// Zero-G environment (e.g. deep space stations)
    ZeroG,
}
"""

if "pub enum GravityLevel {" not in content:
    content = content.replace("pub struct Orbit {", gravity_level + "\n#[derive(Component, Debug, Clone)]\npub struct Orbit {")
    with open('src/layer2/system.rs', 'w') as f:
        f.write(content)

print("Updated system.rs")

# Update src/layer2/station.rs
with open('src/layer2/station.rs', 'r') as f:
    content = f.read()

if "Brewery," not in content:
    content = content.replace("    Shipyard,", "    Shipyard,\n    /// A brewery specializing in zero-g fermentation.\n    Brewery,")

station_comps = """
/// Component for a Zero-G Brewery station.
#[derive(Component, Debug, Clone)]
pub struct ZeroGBrewery {
    /// Timer for producing Void-Ale.
    pub production_time: bevy_time::Timer,
}

/// System to handle Zero-G fermentation.
pub fn zero_g_fermentation_system(
    time: Res<bevy_time::Time>,
    mut query: Query<(
        &mut ZeroGBrewery,
        &crate::layer2::system::GravityLevel,
        &mut crate::layer1::economy::inventory::Inventory,
    )>,
) {
    for (mut brewery, gravity, mut inventory) in query.iter_mut() {
        if *gravity == crate::layer2::system::GravityLevel::ZeroG {
            brewery.production_time.tick(time.delta());
            if brewery.production_time.just_finished() {
                inventory.try_add(crate::layer1::economy::inventory::InventoryItem {
                    item_type: crate::layer1::items::ItemType::VoidAle,
                    entity: None,
                });
            }
        }
    }
}
"""

if "ZeroGBrewery" not in content:
    content = content.replace("pub struct Station {", station_comps + "\n#[derive(Component, Debug, Clone)]\npub struct Station {")
    content = content.replace("Self::Shipyard => vec![(ResourceType::Metal, 200.0), (ResourceType::Fuel, 50.0)],", "Self::Shipyard => vec![(ResourceType::Metal, 200.0), (ResourceType::Fuel, 50.0)],\n            Self::Brewery => vec![(ResourceType::Metal, 150.0)],")
    content = content.replace("Self::Shipyard => \"Shipyard\",", "Self::Shipyard => \"Shipyard\",\n            Self::Brewery => \"Zero-G Brewery\",")
    content = content.replace("Self::Shipyard => '⚓',", "Self::Shipyard => '⚓',\n            Self::Brewery => 'B',")
    with open('src/layer2/station.rs', 'w') as f:
        f.write(content)

print("Updated station.rs")

# Update src/layer1/systems/consumption.rs
with open('src/layer1/systems/consumption.rs', 'r') as f:
    content = f.read()

system_str = """
/// System for consuming Void-Ale to boost morale and leisure.
pub fn consume_void_ale_system(
    mut query: Query<(
        &mut crate::layer1::needs::Needs,
        &mut crate::layer1::economy::inventory::Inventory,
    )>,
) {
    for (mut needs, mut inventory) in query.iter_mut() {
        if needs.leisure < 0.5 || needs.morale() < 0.5 {
            let ale_index = inventory.items.iter().position(|item| item.item_type == crate::layer1::items::ItemType::VoidAle);

            if let Some(index) = ale_index {
                // Consume the ale
                inventory.items.remove(index);

                // Huge boost to leisure and rest
                needs.leisure = (needs.leisure + 0.6).min(1.0);
                needs.rest = (needs.rest + 0.4).min(1.0);
            }
        }
    }
}
"""

if "consume_void_ale_system" not in content:
    content += "\n" + system_str
    content = content.replace("            consume_food_system\n                .after(produce_food_system)\n                .after(update_resource_caps_system),", "            consume_food_system\n                .after(produce_food_system)\n                .after(update_resource_caps_system),\n            consume_void_ale_system.after(consume_food_system),")
    with open('src/layer1/systems/consumption.rs', 'w') as f:
        f.write(content)

print("Updated consumption.rs")

# Update src/simulation.rs
with open('src/simulation.rs', 'r') as f:
    content = f.read()

if "zero_g_fermentation_system" not in content:
    content = content.replace("        crate::layer2::station::build_station_system,", "        crate::layer2::station::build_station_system,\n        crate::layer2::station::zero_g_fermentation_system\n            .after(crate::layer2::fleet::fleet_order_system),")
    with open('src/simulation.rs', 'w') as f:
        f.write(content)

print("Updated simulation.rs")

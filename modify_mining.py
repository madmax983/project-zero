import re

with open("src/layer1/execution/mining.rs", "r") as f:
    content = f.read()

imports = """use crate::layer1::biology::rust_lung::RustLung;
use crate::layer1::economy::inventory::Inventory;
use crate::layer1::items::ItemType;
"""

content = content.replace("use crate::shared::log::MessageLog;\n", "use crate::shared::log::MessageLog;\n" + imports)

rust_lung_logic = """
    // Apply RustLung if mining low purity ore (we approximate low purity based on PurityMap or assume it's checked here)
    if let Some(p) = pos {
        let purity = world
            .get_resource::<crate::layer1::purity::PurityMap>()
            .map_or(0.2, |map| map.get(p.x, p.y));

        // If purity is low (e.g. < 0.5) it counts as low purity.
        if purity < 0.5 {
            let has_rebreather = world.get::<Inventory>(worker_entity)
                .map_or(false, |inv| inv.has_item(ItemType::Rebreather));

            if !has_rebreather {
                world.entity_mut(worker_entity).insert(RustLung);
            }
        }
    }
"""

# Insert right before process_mother_lode
content = content.replace("    let is_mother_lode = world.get::<MotherLode>(entity).is_some();\n", rust_lung_logic + "    let is_mother_lode = world.get::<MotherLode>(entity).is_some();\n")

with open("src/layer1/execution/mining.rs", "w") as f:
    f.write(content)

with open("src/layer1/execution/mining.rs", "r") as f:
    content = f.read()

content = content.replace(".map_or(false, |inv| inv.has_item(ItemType::Rebreather));", ".is_some_and(|inv| inv.has_item(ItemType::Rebreather));")

with open("src/layer1/execution/mining.rs", "w") as f:
    f.write(content)

with open("src/experimental/dream_economy.rs", "r") as f:
    code = f.read()

code = code.replace("let mut query = world.query::<(&Item, &GridPosition), With<DreamMoteItem>>();", "let mut query = world.query_filtered::<(&Item, &GridPosition), With<DreamMoteItem>>();")
code = code.replace("Item { item_type: ItemType::None, ..Default::default() }", "Item { item_type: ItemType::None }")
code = code.replace("if time.tick % 10 != 0 {", "#[allow(clippy::manual_is_multiple_of)]\n    if time.tick % 10 != 0 {")

with open("src/experimental/dream_economy.rs", "w") as f:
    f.write(code)

with open("src/experimental/dream_economy.rs", "r") as f:
    code = f.read()

# Revert is_multiple_of
code = code.replace("!time.tick.is_multiple_of(10)", "time.tick % 10 != 0")
code = code.replace("if time.tick % 10 != 0 { // allow manual multiple\n#[allow(clippy::manual_is_multiple_of)]\n", "if time.tick % 10 != 0 {")

# Introduce components instead of ItemType
components = """
/// Marker component for a positive dream item.
#[derive(Component, Debug, Clone)]
pub struct DreamMoteItem;

/// Marker component for a negative dream item.
#[derive(Component, Debug, Clone)]
pub struct NightmareFragmentItem;
"""
code = code.replace("pub struct DreamCatcher;", "pub struct DreamCatcher;\n" + components)

# Update harvest_dreams_system
old_harvest = """                    let item_type = if dream.is_nightmare {
                        ItemType::NightmareFragment
                    } else {
                        ItemType::DreamMote
                    };

                    commands.spawn((
                        Item { item_type },
                        *catcher_pos,
                    ));"""

new_harvest = """                    let mut entity = commands.spawn((
                        Item { item_type: ItemType::None, ..Default::default() },
                        *catcher_pos,
                    ));
                    if dream.is_nightmare {
                        entity.insert(NightmareFragmentItem);
                    } else {
                        entity.insert(DreamMoteItem);
                    }"""
code = code.replace(old_harvest, new_harvest)

# Update nightmare_paranoia_system
old_paranoia = """    for (item_pos, item) in &item_query {
        if item.item_type == ItemType::NightmareFragment {"""
new_paranoia = """    for (item_pos, _item) in &item_query {
        // Only run for items with NightmareFragmentItem
        if true {"""

# Wait, we need to change the query
code = code.replace("item_query: Query<(&GridPosition, &Item)>,", "item_query: Query<(&GridPosition, &Item), With<NightmareFragmentItem>>,")
code = code.replace(old_paranoia, new_paranoia)

# Update tests
code = code.replace("if item.item_type == ItemType::DreamMote && pos.x == 5 && pos.y == 5 {", "if pos.x == 5 && pos.y == 5 {")
code = code.replace("let mut query = world.query::<(&Item, &GridPosition)>();", "let mut query = world.query::<(&Item, &GridPosition), With<DreamMoteItem>>();")

code = code.replace("Item { item_type: ItemType::NightmareFragment }", "Item { item_type: ItemType::None, ..Default::default() }")
code = code.replace("GridPosition { x: 5, y: 5 },\n        ));", "GridPosition { x: 5, y: 5 },\n            NightmareFragmentItem,\n        ));")

with open("src/experimental/dream_economy.rs", "w") as f:
    f.write(code)

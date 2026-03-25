with open("src/layer1/items.rs", "r") as f:
    items_rs = f.read()

items_rs = items_rs.replace("""Scrap,
    #[cfg(feature = "nova")]
    DreamMote,
    #[cfg(feature = "nova")]
    NightmareFragment,""", "Scrap,")

with open("src/layer1/items.rs", "w") as f:
    f.write(items_rs)

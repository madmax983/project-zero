with open("src/experimental/architectural_palimpsest.rs", "r") as f:
    content = f.read()

content = content.replace("building.building_type.beauty_radius() as f32", "building.building_type.beauty_radius()")

with open("src/experimental/architectural_palimpsest.rs", "w") as f:
    f.write(content)

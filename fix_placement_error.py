import re

with open('src/layer1/architecture/building.rs', 'r', encoding='utf-8') as f:
    content = f.read()

# Instead of returning Result<..., PlacementError>, return Result<(), &'static str>
# and avoid the enum altogether.

# Let's replace the enum definition
enum_regex = r'#\[derive\(Debug, Clone, Copy, PartialEq, Eq\)\]\s*enum PlacementError \{\s*OutOfBounds,\s*Occupied,\s*InvalidTerrain\(TerrainType\),\s*\}'
content = re.sub(enum_regex, '', content)

# Let's replace validate_building_placement
old_validate = r'''fn validate_building_placement\(world: &World, x: i32, y: i32\) -> Result<\(\), PlacementError> \{
    let terrain = world\.resource::<TerrainGrid>\(\);
    let occupied = world\.resource::<OccupiedTiles>\(\);

    // Check bounds
    if x < 0 \|\| y < 0 \{
        return Err\(PlacementError::OutOfBounds\);
    \}

    // Check terrain
    #\[allow\(clippy::cast_sign_loss\)\]
    let tile = terrain
        \.get\(x as usize, y as usize\)
        \.ok_or\(PlacementError::OutOfBounds\)\?;

    match tile \{
        TerrainType::Water \| TerrainType::Rock => Err\(PlacementError::InvalidTerrain\(\*tile\)\),
        _ => \{
            // Check occupation
            if occupied\.0\.contains\(&\(x, y\)\) \{
                Err\(PlacementError::Occupied\)
            \} else \{
                Ok\(\(\)\)
            \}
        \}
    \}
\}'''

new_validate = r'''fn validate_building_placement(world: &World, x: i32, y: i32) -> Result<(), &'static str> {
    let terrain = world.resource::<TerrainGrid>();
    let occupied = world.resource::<OccupiedTiles>();

    if x < 0 || y < 0 { return Err("Out of bounds"); }

    #[allow(clippy::cast_sign_loss)]
    let tile = terrain.get(x as usize, y as usize).ok_or("Out of bounds")?;

    if *tile == TerrainType::Water { return Err("Cannot build on Water"); }
    if *tile == TerrainType::Rock { return Err("Cannot build on Rock"); }

    if occupied.0.contains(&(x, y)) {
        return Err("Location occupied");
    }

    Ok(())
}'''

content = re.sub(r'fn validate_building_placement[\s\S]*?Ok\(\(\)\)\n            \}\n        \}\n    \}\n\}', new_validate, content)

# handle_placement_error
old_handle = r'''fn handle_placement_error\(world: &mut World, error: PlacementError\) \{
    let reason = match error \{
        PlacementError::OutOfBounds => "Out of bounds",
        PlacementError::Occupied => "Location occupied",
        PlacementError::InvalidTerrain\(TerrainType::Water\) => "Cannot build on Water",
        PlacementError::InvalidTerrain\(TerrainType::Rock\) => "Cannot build on Rock",
        PlacementError::InvalidTerrain\(_\) => "Cannot build here",
    \};

    if let Some\(mut log\) = world\.get_resource_mut::<MessageLog>\(\) \{
        log\.add\(format!\("Failed: \{reason\}"\)\);
    \}
\}'''

new_handle = r'''fn handle_placement_error(world: &mut World, reason: &str) {
    if let Some(mut log) = world.get_resource_mut::<MessageLog>() {
        log.add(format!("Failed: {reason}"));
    }
}'''

content = re.sub(r'fn handle_placement_error[\s\S]*?\}\n\}', new_handle, content)

# usage in try_place_building
old_usage = r'''    if let Err\(e\) = validate_building_placement\(world, x, y\) \{
        let allow_override = e == PlacementError::Occupied && grave_entity\.is_some\(\);
        if !allow_override \{
            handle_placement_error\(world, e\);
            return false;
        \}
    \}'''

new_usage = r'''    if let Err(e) = validate_building_placement(world, x, y) {
        let allow_override = e == "Location occupied" && grave_entity.is_some();
        if !allow_override {
            handle_placement_error(world, e);
            return false;
        }
    }'''

content = content.replace('    if let Err(e) = validate_building_placement(world, x, y) {\n        let allow_override = e == PlacementError::Occupied && grave_entity.is_some();\n        if !allow_override {\n            handle_placement_error(world, e);\n            return false;\n        }\n    }', new_usage)

with open('src/layer1/architecture/building.rs', 'w', encoding='utf-8') as f:
    f.write(content)

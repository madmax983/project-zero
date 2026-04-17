import sys

def modify_building():
    filepath = 'src/layer1/architecture/building.rs'
    with open(filepath, 'r') as f:
        content = f.read()

    # Fix Office
    content = content.replace("entity.insert(crate::layer1::admin::Office);", "entity.insert(crate::layer1::admin::Office { capacity: 5, workers: Vec::new() });")

    # 7. Match arm in insert_specific_building_components
    # Wait, the previous script inserted it into the WRONG match arm?
    # Let's remove the previous bad insert
    content = content.replace("""BuildingType::Spaceport => {
            entity.insert(crate::layer1::admin::Office { capacity: 5, workers: Vec::new() });
            entity.insert(crate::layer1::trade::TradeDepot);
        }
        _ => {}""", "_ => {}")

    match_start = content.find("fn insert_specific_building_components(entity: &mut EntityWorldMut<'_>, building_type: BuildingType) {")
    if match_start == -1:
        match_start = content.find("fn insert_specific_building_components(")

    match_stmt_start = content.find("match building_type {", match_start)
    catchall_pos = content.find("_ => {}", match_stmt_start)
    if catchall_pos != -1:
        insert_pos = catchall_pos
        new_content = content[:insert_pos] + """Self::Spaceport => {
            entity.insert(crate::layer1::admin::Office { capacity: 5, workers: Vec::new() });
            entity.insert(crate::layer1::trade::TradeDepot);
        }
        """ + content[insert_pos:]
        content = new_content

    with open(filepath, 'w') as f:
        f.write(content)

modify_building()

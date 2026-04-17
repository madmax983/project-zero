import sys

def modify_building():
    filepath = 'src/layer1/architecture/building.rs'
    with open(filepath, 'r') as f:
        content = f.read()

    # We need to insert into `insert_specific_building_components`
    search_str = "Self::MediaStation => {"
    media_station_pos = content.find(search_str)
    if media_station_pos != -1:
        insert_pos = content.find("}", media_station_pos) + 1
        content = content[:insert_pos] + "\n        Self::Spaceport => {\n            entity.insert(crate::layer1::admin::Office { capacity: 5, workers: Vec::new() });\n            entity.insert(crate::layer1::trade::TradeDepot);\n        }" + content[insert_pos:]
    else:
        print("Could not find MediaStation in insert_specific_building_components")

    with open(filepath, 'w') as f:
        f.write(content)

modify_building()

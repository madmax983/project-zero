import sys
import re

def modify_building():
    filepath = 'src/layer1/architecture/building.rs'
    with open(filepath, 'r') as f:
        content = f.read()

    # Find the end of this match block
    school_pos = content.find("BuildingType::School | BuildingType::MediaStation => {")
    if school_pos != -1:
        insert_pos = content.find("}", school_pos) + 1
        content = content[:insert_pos] + "\n        BuildingType::Spaceport => configure_civic(&mut entity, building_type)," + content[insert_pos:]
    else:
        print("Could not find School | MediaStation")

    with open(filepath, 'w') as f:
        f.write(content)

modify_building()

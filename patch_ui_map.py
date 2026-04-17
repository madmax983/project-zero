import sys

def modify_ui_map():
    filepath = 'src/ui/map.rs'
    with open(filepath, 'r') as f:
        content = f.read()

    # 1. line 733 match building
    search_str1 = "BuildingType::MediaStation => \"M\","
    media_station_pos1 = content.find(search_str1)
    if media_station_pos1 != -1:
        insert_pos1 = media_station_pos1 + len(search_str1)
        content = content[:insert_pos1] + "\n        BuildingType::Spaceport => \"P\"," + content[insert_pos1:]
    else:
        print("Could not find MediaStation in map.rs first match")

    # 2. line 813 match building
    search_str2 = "BuildingType::MediaStation => Color::Cyan,"
    media_station_pos2 = content.find(search_str2)
    if media_station_pos2 != -1:
        insert_pos2 = media_station_pos2 + len(search_str2)
        content = content[:insert_pos2] + "\n            BuildingType::Spaceport => Color::LightBlue," + content[insert_pos2:]
    else:
        print("Could not find MediaStation in map.rs second match")

    with open(filepath, 'w') as f:
        f.write(content)

modify_ui_map()

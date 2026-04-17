import sys
import re

def modify_building():
    filepath = 'src/layer1/architecture/building.rs'
    with open(filepath, 'r') as f:
        content = f.read()

    # The compiler output says: src/layer1/architecture/building.rs:1311:11
    #     match building_type {

    # We will search for: match building_type {
    # near line 1311.

    # Find the function fn insert_specific_building_components
    func_start = content.find("fn insert_specific_building_components")
    match_start = content.find("match building_type {", func_start)

    # Find the end of this match block
    catchall_pos = content.find("_ => {}", match_start)
    if catchall_pos != -1:
        insert_pos = catchall_pos
        content = content[:insert_pos] + "Self::Spaceport => {}\n        " + content[insert_pos:]
    else:
        # maybe there is no catchall?
        # let's just insert before the last arm?
        # actually E0004 means there is NO catchall.
        print("No catchall _ => {}")

        # let's find Self::MediaStation =>
        media_station_pos = content.find("Self::MediaStation =>", match_start)
        if media_station_pos != -1:
            end_of_media_station = content.find("}", media_station_pos) + 1
            insert_pos = end_of_media_station
            content = content[:insert_pos] + "\n        Self::Spaceport => {}" + content[insert_pos:]
        else:
            print("Could not find Self::MediaStation =>")

    with open(filepath, 'w') as f:
        f.write(content)

modify_building()

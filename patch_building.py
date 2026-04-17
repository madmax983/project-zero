import sys

def modify_building():
    filepath = 'src/layer1/architecture/building.rs'
    with open(filepath, 'r') as f:
        content = f.read()

    # 1. Add to BuildingType enum
    enum_start = content.find("pub enum BuildingType {")
    enum_content = content[enum_start:content.find("}", enum_start)]
    media_station_pos = enum_content.find("MediaStation,")
    insert_pos = enum_start + media_station_pos + len("MediaStation,")
    content = content[:insert_pos] + "\n    /// Port for arriving and departing spacecraft.\n    Spaceport," + content[insert_pos:]

    # 2. Add to blocks_wind
    blocks_wind_start = content.find("pub const fn blocks_wind(&self) -> bool {")
    shower_pos = content.find("Self::Shower => true,", blocks_wind_start)
    insert_pos = shower_pos + len("Self::Shower")
    content = content[:insert_pos] + "\n            | Self::Spaceport" + content[insert_pos:]

    # 4. Add to label
    label_start = content.find("pub const fn label(&self) -> &'static str {")
    media_station_label_pos = content.find("Self::MediaStation => \"Media Station\",", label_start)
    insert_pos = media_station_label_pos + len("Self::MediaStation => \"Media Station\",")
    content = content[:insert_pos] + "\n            Self::Spaceport => \"Spaceport\"," + content[insert_pos:]

    # 5. Add to char
    char_start = content.find("pub const fn char(&self) -> char {")
    school_char_pos = content.find("Self::School => 'S',", char_start)
    insert_pos = school_char_pos + len("Self::School => 'S',")
    content = content[:insert_pos] + "\n            Self::Spaceport => 'P'," + content[insert_pos:]

    # 6. Add to cost
    cost_start = content.find("pub const fn cost(&self, material: MaterialType) -> ColonyResources {")
    school_cost_pos = content.find("Self::School => ColonyResources {", cost_start)
    content = content[:school_cost_pos] + """Self::Spaceport => ColonyResources {
                metal: 500.0,
                stone: 200.0,
                tools: 50.0,
                ..ColonyResources::zeroed()
            },
            """ + content[school_cost_pos:]

    # 7. Match arm in insert_specific_building_components
    match_start = content.find("match building_type {")
    catchall_pos = content.find("_ => {}", match_start)
    content = content[:catchall_pos] + """BuildingType::Spaceport => {
            entity.insert(crate::layer1::admin::Office);
            entity.insert(crate::layer1::trade::TradeDepot);
        }
        """ + content[catchall_pos:]

    # 8. test_building_type_next
    search_str = "assert_eq!(BuildingType::School.next(), BuildingType::MediaStation);"
    pos = content.find(search_str)
    insert_pos = pos + len(search_str)
    content = content[:insert_pos] + "\n        assert_eq!(BuildingType::MediaStation.next(), BuildingType::Spaceport);" + content[insert_pos:]
    content = content.replace("assert_eq!(BuildingType::MediaStation.next(), BuildingType::Housing);", "assert_eq!(BuildingType::Spaceport.next(), BuildingType::Housing);")

    # 9. test_build_mode_type_cycling
    replacement = """
        mode.selected = mode.selected.next();
        assert_eq!(mode.selected, BuildingType::School);

        mode.selected = mode.selected.next();
        assert_eq!(mode.selected, BuildingType::MediaStation);

        mode.selected = mode.selected.next();
        assert_eq!(mode.selected, BuildingType::Spaceport);

        mode.selected = mode.selected.next();
        assert_eq!(mode.selected, BuildingType::Housing);
    }
"""
    school_pos = content.find("mode.selected = mode.selected.next();\n        assert_eq!(mode.selected, BuildingType::School);")
    end_pos = content.find("}", school_pos) + 1
    content = content[:school_pos] + replacement.strip() + "\n" + content[end_pos:]

    with open(filepath, 'w') as f:
        f.write(content)

modify_building()

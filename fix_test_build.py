import re

with open('src/layer1/building.rs', 'r') as f:
    content = f.read()

content = content.replace(
'''        mode.selected = mode.selected.next();
        assert_eq!(mode.selected, BuildingType::Hospital);''',
'''        mode.selected = mode.selected.next();
        assert_eq!(mode.selected, BuildingType::AntiGravGenerator);

        mode.selected = mode.selected.next();
        assert_eq!(mode.selected, BuildingType::Hospital);'''
)

with open('src/layer1/building.rs', 'w') as f:
    f.write(content)

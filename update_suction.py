with open('src/layer1/suction.rs', 'r') as f:
    content = f.read()

content = content.replace('use std::collections::HashSet;', 'use bevy_utils::HashSet;')

with open('src/layer1/suction.rs', 'w') as f:
    f.write(content)

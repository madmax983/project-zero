with open('src/layer1/flora.rs', 'r') as f:
    content = f.read()

content = content.replace('let occupied: std::collections::HashSet<(i32, i32)> =', 'let occupied: bevy_utils::HashSet<(i32, i32)> =')

with open('src/layer1/flora.rs', 'w') as f:
    f.write(content)

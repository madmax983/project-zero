with open('src/layer1/core/events.rs', 'r') as f:
    content = f.read()

content = content.replace('\\n/// Event fired when a pop is consumed by a living building.', '\n/// Event fired when a pop is consumed by a living building.')

with open('src/layer1/core/events.rs', 'w') as f:
    f.write(content)

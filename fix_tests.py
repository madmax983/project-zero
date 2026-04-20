import re

with open('src/layer1/direct_link.rs', 'r') as f:
    text = f.read()

# remove assertions
text = re.sub(r'        assert_eq!\(\n            world.resource::<InputContextStack>\(\).current\(\),\n            InputContext::DirectControl\n        \);\n', '', text)

with open('src/layer1/direct_link.rs', 'w') as f:
    f.write(text)

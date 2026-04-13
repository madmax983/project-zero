import re

with open('src/layer1/culture/animism.rs', 'r') as f:
    content = f.read()

# Fix the doctest
content = content.replace('use scale::layer1::animism::{Spirit, evolve_spirits_system};', 'use scale::layer1::culture::animism::{Spirit, evolve_spirits_system};')

with open('src/layer1/culture/animism.rs', 'w') as f:
    f.write(content)

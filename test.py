import re

content = open('src/layer1/economy/existential_audit.rs').read()
print("Contains 'meaning':", 'meaning' in content.lower())

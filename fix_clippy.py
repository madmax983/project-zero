with open("src/layer1/social/xenoflora_pet.rs", "r") as f:
    text = f.read()

import re
new_text = re.sub(
    r'if resources\.try_deduct\(&cost\) \{\s*pet\.is_starving = false;\s*\} else \{\s*pet\.is_starving = true;\s*\}',
    'pet.is_starving = !resources.try_deduct(&cost);',
    text
)

with open("src/layer1/social/xenoflora_pet.rs", "w") as f:
    f.write(new_text)

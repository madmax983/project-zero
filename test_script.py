import re
with open("src/layer1/mod.rs", "r") as f:
    text = f.read()

# see if there is any mention of resonance
resonance = re.findall(r'reso\w+', text)
print(resonance)

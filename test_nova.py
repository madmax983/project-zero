import re

with open(".jules/nova.md", "r") as f:
    nova = f.read()

print("Meme Plague" in nova)
print("The Gossip Plaque" in nova)

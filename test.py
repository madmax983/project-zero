import re

print("Starting")
with open('specs/184-orbital-debris.md', 'r') as f:
    text = f.read()

# check if question has architect answer
for line in text.split('\n'):
    if 'Builder:' in line:
        print(line)

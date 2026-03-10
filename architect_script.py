import os

with open('design/BACKLOG.md', 'r') as f:
    backlog = f.read()

count = sum(1 for line in backlog.split('\n') if '- [ ]' in line)

if count >= 10:
    print(f"Backlog has {count} items. Waiting for builders to catch up. I have no more tasks to do.")
else:
    print(f"Backlog has {count} items. I should add more specs.")

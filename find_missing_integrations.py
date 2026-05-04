import re

with open('design/COMPLETED.md', 'r') as f:
    lines = f.readlines()

completed_specs = set()
integrations = set()

for line in lines:
    spec_match = re.search(r'`(\d{3,4})` (.*?) — `specs/', line)
    if spec_match:
        completed_specs.add((spec_match.group(1), spec_match.group(2).strip()))

    int_match = re.search(r'`INT-(\d{3,4})`', line)
    if int_match:
        integrations.add(int_match.group(1))

print("Completed specs without integrations:")
for spec_id, name in sorted(list(completed_specs)):
    if spec_id not in integrations:
        print(f"{spec_id}: {name}")

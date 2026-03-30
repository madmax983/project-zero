import re

with open("specs/562-the-justice-system.md", "r") as f:
    text = f.read()

# find RED tests code
red_tests = re.search(r"## 3. RED Phase: Tests First\n\n```rust\n(.*?)```", text, re.DOTALL).group(1)
print("Found RED tests")

green_code = re.search(r"## 4. GREEN Phase: Minimal Implementation\n\n```rust\n(.*?)```", text, re.DOTALL).group(1)
print("Found GREEN code")

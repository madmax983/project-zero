import re
import os

ideas_file = "design/IDEAS.md"
backlog_file = "design/BACKLOG.md"

with open(ideas_file, "r") as f:
    ideas_content = f.read()

# Find the first 4 ideas that are NOT specced
# The regex needs to handle the fact that there might be empty lines or not
pattern = r'## (?!.*\[SPECCED\])(.*?)\n+(.*?)\*\*Layer:\*\*\s*(.*?)\n+(.*?)\*\*Fantasy:\*\*\s*(.*?)\n+(.*?)\*\*Mechanic:\*\*\s*(.*?)\n+(.*?)\*\*Emergence:\*\*\s*(.*?)\n+(.*?)\*\*Tension:\*\*\s*(.*?)\n+'

matches = re.finditer(pattern, ideas_content)
ideas = []
for match in matches:
    ideas.append({
        'title': match.group(1).strip(),
        'layer': match.group(3).strip(),
        'fantasy': match.group(5).strip(),
        'mechanic': match.group(7).strip(),
        'emergence': match.group(9).strip(),
        'tension': match.group(11).strip(),
    })
    if len(ideas) == 4:
        break

if not ideas:
    print("No unspecced ideas found.")
    exit()

next_spec_num = 728

for idea in ideas:
    title = idea['title']
    layer = idea['layer']
    fantasy = idea['fantasy']
    mechanic = idea['mechanic']
    emergence = idea['emergence']
    tension = idea['tension']

    # Generate filename
    clean_title = title.lower().replace(" ", "-").replace("'", "").replace('"', "").replace(":", "").replace(",", "")
    filename = f"specs/{next_spec_num:03d}-{clean_title}.md"

    spec_content = f"""# Spec {next_spec_num}: {title}

## 1. Overview
**Layer:** {layer}
**Fantasy:** {fantasy}
**Mechanic:** {mechanic}
**Emergence:** {emergence}
**Tension:** {tension}

## 2. Dependencies
- Base simulation framework
- Map and Grid systems

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {{
    use super::*;

    #[test]
    fn test_{clean_title.replace("-", "_")}_basic_behavior() {{
        // Arrange
        // Act
        // Assert
        panic!("Test not implemented");
    }}

    #[test]
    fn test_{clean_title.replace("-", "_")}_edge_cases() {{
        // Arrange
        // Act
        // Assert
        panic!("Test not implemented");
    }}
}}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// Minimal implementation to make the tests pass
pub fn process_{clean_title.replace("-", "_")}() {{
    unimplemented!("Implementation not written");
}}
```

## 5. REFACTOR Phase: Quality & Design
- Extract magic numbers into constants or configuration.
- Ensure proper use of Bevy's ECS systems and queries.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >= 85% for new code

## 7. Technical Guidance
- Integrate into the appropriate Layer simulation schedule.
- Use events for cross-system communication if necessary.

## 8. Questions
*Builder: add questions here if spec is unclear.*
"""
    with open(filename, "w") as f:
        f.write(spec_content)

    print(f"Created {filename}")

    # Update IDEAS.md
    ideas_content = ideas_content.replace(f"## {title}", f"## {title} [SPECCED]")

    # Update BACKLOG.md entry
    backlog_entry = f"- [ ] `{next_spec_num:03d}` {title} — `{filename}`\n"
    with open(backlog_file, "a") as f:
        f.write(backlog_entry)

    next_spec_num += 1

with open(ideas_file, "w") as f:
    f.write(ideas_content)

print("Updated IDEAS.md and BACKLOG.md")

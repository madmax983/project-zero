import os

ideas_file = "design/IDEAS.md"
with open(ideas_file, "r") as f:
    ideas_content = f.read()

import re
ideas = re.split(r'\n## ', '\n' + ideas_content)

spec_number = 1276

def format_idea_to_spec(title, body, spec_num):
    # Extract details
    layer_match = re.search(r'\*\*Layer:\*\*\s*(.*?)\n', body)
    fantasy_match = re.search(r'\*\*Fantasy:\*\*\s*(.*?)\n', body)
    mechanic_match = re.search(r'\*\*Mechanic:\*\*\s*(.*?)\n', body)

    layer = layer_match.group(1).strip() if layer_match else "1"
    fantasy = fantasy_match.group(1).strip() if fantasy_match else ""
    mechanic = mechanic_match.group(1).strip() if mechanic_match else ""

    clean_title = title.replace('[SPECCED]', '').strip()
    formatted_title = clean_title.replace(' ', '-').lower()

    spec_content = f"""# {spec_num}: {clean_title}

## 1. Overview
**Layer:** {layer}

**Fantasy:** {fantasy}

**Mechanic:** {mechanic}

## 2. Dependencies
- Base simulation framework (`App`, `World`)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {{
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_{formatted_title.replace('-', '_')}_basic_behavior() {{
        // Arrange
        let mut app = App::new();
        // Act
        // Assert
    }}
}}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

// Components and systems here
```

## 5. REFACTOR Phase: Quality & Design
- TBD

## 6. Acceptance Criteria (Testable!)
- [ ] All RED phase tests pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >=85% for new code

## 7. Technical Guidance
- Implement MVP

## 8. Questions
*Builder: add questions here if spec is unclear.*
"""
    return spec_content, formatted_title, clean_title

# Find unspecced ideas
new_specs = []
for idea in ideas[1:]:
    lines = idea.split('\n')
    title = lines[0]
    if '[SPECCED]' not in title:
        body = '\n'.join(lines[1:])
        spec_content, filename_slug, clean_title = format_idea_to_spec(title, body, spec_number)
        filename = f"specs/{spec_number}-{filename_slug}.md"
        new_specs.append((filename, spec_content, spec_number, clean_title, filename_slug))
        spec_number += 1
        if len(new_specs) == 5: # Just do 5 for now
            break

for filename, content, num, title, slug in new_specs:
    with open(filename, "w") as f:
        f.write(content)
    print(f"Created {filename}")

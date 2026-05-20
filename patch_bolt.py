import re

with open("src/ui/inspector.rs", "r") as f:
    content = f.read()

# Replace 1
search1 = """    // Intersperse with commas
    let mut spans = Vec::new();
    spans.push(Span::raw("Traits: "));
    for (i, t) in traits.into_iter().enumerate() {
        if i > 0 {
            spans.push(Span::raw(", "));
        }
        spans.push(t);
    }

    let p = Paragraph::new(Line::from(spans)).block(Block::default().borders(Borders::NONE)); // No block to save space"""

replace1 = """    // Intersperse with commas
    // ⚡ Bolt Optimization: Use `Vec::with_capacity` to prevent multiple allocations
    // while building the list of trait spans.
    let mut spans = Vec::with_capacity(traits.len() * 2 + 1);
    spans.push(Span::raw("Traits: "));
    for (i, t) in traits.into_iter().enumerate() {
        if i > 0 {
            spans.push(Span::raw(", "));
        }
        spans.push(t);
    }

    let p = Paragraph::new(Line::from(spans)).block(Block::default().borders(Borders::NONE)); // No block to save space"""

content = content.replace(search1, replace1)

with open("src/ui/inspector.rs", "w") as f:
    f.write(content)

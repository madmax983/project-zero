import re

with open("src/ui/inspector.rs", "r") as f:
    content = f.read()

# Replace 2
search2 = """    let mut lines = Vec::new();
    if stockpile.food_bonus > 0.0 {
        lines.push(Line::from(vec![
            Span::raw("Food: +"),"""

replace2 = """    // ⚡ Bolt Optimization: Pre-allocate capacity for `lines`
    let mut lines = Vec::with_capacity(4);
    if stockpile.food_bonus > 0.0 {
        lines.push(Line::from(vec![
            Span::raw("Food: +"),"""

content = content.replace(search2, replace2)

with open("src/ui/inspector.rs", "w") as f:
    f.write(content)

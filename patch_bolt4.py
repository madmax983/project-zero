import re

with open("src/ui/inspector.rs", "r") as f:
    content = f.read()

# Replace 4
search4 = """    let events: Vec<ListItem> = bio
        .events
        .iter()
        .rev()
        .take(5)
        .map(|e| {"""

replace4 = """    // ⚡ Bolt Optimization: Use an iterator instead of collecting into an intermediate `Vec<ListItem>`
    // to prevent allocations when rendering the biography list.
    let events = bio
        .events
        .iter()
        .rev()
        .take(5)
        .map(|e| {"""

content = content.replace(search4, replace4)

# Replace 5
search5 = """                Span::styled("│ ", Style::default().fg(Color::DarkGray)),
                Span::raw(&e.text),
            ]))
        })
        .collect();

    let list = List::new(events).block(bio_block);"""

replace5 = """                Span::styled("│ ", Style::default().fg(Color::DarkGray)),
                Span::raw(&e.text),
            ]))
        });

    let list = List::new(events).block(bio_block);"""

content = content.replace(search5, replace5)

with open("src/ui/inspector.rs", "w") as f:
    f.write(content)

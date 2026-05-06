with open("src/bin/headless.rs", "r") as f:
    code = f.read()

# Make the minor events dim and legendaries pop even more
search = """        let importance_color = match event.importance {
            EventImportance::Legendary => Color::Yellow,
            EventImportance::Major => Color::Magenta,
            EventImportance::Standard => Color::White,
            EventImportance::Minor => Color::DarkGrey,
        };

        // Legendary events get bold text
        let mut event_cell = Cell::new(&event.text).fg(importance_color);
        if event.importance == EventImportance::Legendary {
            event_cell = event_cell.add_attribute(Attribute::Bold);
        }

        table.add_row(vec![
            Cell::new(event.year.to_string()),
            Cell::new(event.tick.to_string()),
            event_cell,
        ]);"""

replace = """        let importance_color = match event.importance {
            EventImportance::Legendary => Color::Yellow,
            EventImportance::Major => Color::Magenta,
            EventImportance::Standard => Color::White,
            EventImportance::Minor => Color::DarkGrey,
        };

        let mut event_cell = Cell::new(&event.text).fg(importance_color);
        let mut year_cell = Cell::new(event.year.to_string());
        let mut tick_cell = Cell::new(event.tick.to_string());

        if event.importance == EventImportance::Legendary {
            event_cell = event_cell.add_attribute(Attribute::Bold);
            year_cell = year_cell.fg(Color::Yellow).add_attribute(Attribute::Bold);
            tick_cell = tick_cell.fg(Color::Yellow).add_attribute(Attribute::Bold);
        } else if event.importance == EventImportance::Minor {
            year_cell = year_cell.fg(Color::DarkGrey);
            tick_cell = tick_cell.fg(Color::DarkGrey);
        }

        table.add_row(vec![year_cell, tick_cell, event_cell]);"""

code = code.replace(search, replace)
with open("src/bin/headless.rs", "w") as f:
    f.write(code)

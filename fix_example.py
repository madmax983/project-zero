with open("examples/minimal_nova_demo.rs", "r") as f:
    code = f.read()

search = """    for story in &tradition.stories {
        let genre_str = format!("{:?}", story.genre);
        let genre_color = match genre_str.as_str() {
            "Heroic" => TableColor::Yellow,
            "Tragedy" => TableColor::Red,
            "Cautionary" => TableColor::Magenta,
            _ => TableColor::DarkGrey,
        };

        table.add_row(vec![
            Cell::new(&genre_str).fg(genre_color),
            Cell::new(story.historical_date.to_string()).fg(TableColor::Cyan),
            Cell::new(story.mutations.to_string()).fg(TableColor::Cyan),
            Cell::new(&story.text).fg(TableColor::White),
        ]);
    }"""

replace = """    for story in &tradition.stories {
        let genre_str = format!("{:?}", story.genre);
        let genre_color = match genre_str.as_str() {
            "Heroic" => TableColor::Yellow,
            "Tragedy" => TableColor::Red,
            "Cautionary" => TableColor::Magenta,
            _ => TableColor::DarkGrey,
        };

        let genre_text = match genre_str.as_str() {
            "Heroic" => "🌟 Heroic",
            "Tragedy" => "🎭 Tragedy",
            "Cautionary" => "⚠️ Cautionary",
            _ => "📝 Trivial",
        };

        table.add_row(vec![
            Cell::new(genre_text).fg(genre_color),
            Cell::new(story.historical_date.to_string()).fg(TableColor::Cyan),
            Cell::new(story.mutations.to_string()).fg(TableColor::Cyan),
            Cell::new(&story.text).fg(TableColor::White),
        ]);
    }"""

code = code.replace(search, replace)
with open("examples/minimal_nova_demo.rs", "w") as f:
    f.write(code)

import re

with open("src/ui/status.rs", "r") as f:
    content = f.read()

# Refactor get_status_line

old_func = """pub fn get_status_line<'a>(
    tick: u64,
    speed: SimSpeed,
    paused: bool,
    build_mode: &BuildMode,
    designation_mode: &DesignationMode,
    location_name: Option<&'a str>,
    pop_count: usize,
    food_yield: f32,
    rations: f32,
    tools: f32,
    morale: f32,
    efficiency: f32,
    season: Option<Season>,
    solar_cycle: Option<SolarCycle>,
    risk_pct: f32,
) -> Line<'a> {
    let mut spans = Vec::new();

    // 1. Play/Pause
    if paused {
        spans.push(Span::styled(
            " ⏸ ",
            Style::default()
                .fg(Color::White)
                .bg(Color::Red)
                .add_modifier(Modifier::BOLD),
        ));
    } else {
        spans.push(Span::styled(
            " ▶ ",
            Style::default()
                .fg(Color::Black)
                .bg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ));
    }

    // 2. Day & Time Group
    spans.push(Span::styled(
        format!(" Day {} ", tick),
        Style::default().add_modifier(Modifier::BOLD),
    ));

    // 2b. Season & Solar
    if let Some(s) = season {
        let color = match s {
            Season::Spring => Color::Green,
            Season::Summer => Color::Yellow,
            Season::Autumn => Color::Rgb(200, 100, 0),
            Season::Winter => Color::Cyan,
        };
        spans.push(Span::styled(
            format!(" {} ", s.name()),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ));
    }

    if let Some(cycle) = solar_cycle {
        spans.push(Span::styled(
            format!(" {} ", cycle.label()),
            Style::default().fg(Color::Yellow),
        ));
    }

    // Separator block
    spans.push(Span::styled("  ║  ", Style::default().fg(Color::DarkGray)));

    // 3. Colony Stats Group (Souls, Morale, Admin)
    spans.push(Span::styled("👨 ", Style::default().fg(Color::Cyan)));
    spans.push(Span::styled(
        format!("{} ", pop_count),
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    ));

    let morale_percent = (morale * 100.0).round() as u8;
    let morale_color = if morale < 0.3 {
        Color::Red
    } else if morale < 0.7 {
        Color::Yellow
    } else {
        Color::Green
    };
    spans.push(Span::styled(" 😊 ", Style::default().fg(morale_color)));
    spans.push(Span::styled(
        format!("{}% ", morale_percent),
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    ));

    let eff_percent = (efficiency * 100.0).round() as u8;
    let eff_color = if efficiency < 0.5 {
        Color::Red
    } else if efficiency < 0.8 {
        Color::Yellow
    } else {
        Color::Green
    };
    spans.push(Span::styled(" ⚙ ", Style::default().fg(eff_color)));
    spans.push(Span::styled(
        format!("{}% ", eff_percent),
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    ));

    // Separator block
    spans.push(Span::styled("  ║  ", Style::default().fg(Color::DarkGray)));

    // 4. Resources Group (Food, Tools, Risk)
    let total_food = food_yield + rations;
    let food_color = if total_food < 10.0 {
        Color::Red
    } else {
        Color::Green
    };
    spans.push(Span::styled("🌾 ", Style::default().fg(food_color)));
    spans.push(Span::styled(
        format!("{:.0}+{:.0} ", food_yield, rations),
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    ));

    spans.push(Span::styled(" 🔨 ", Style::default().fg(Color::Yellow)));
    spans.push(Span::styled(
        format!("{:.0} ", tools),
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    ));

    let risk_color = if risk_pct > 80.0 {
        Color::White
    } else if risk_pct > 50.0 {
        Color::Yellow
    } else {
        Color::DarkGray
    };
    let risk_bg = if risk_pct > 80.0 {
        Color::Red
    } else {
        Color::Reset
    };
    spans.push(Span::styled(
        " 👁 ",
        Style::default().fg(risk_color).bg(risk_bg),
    ));
    spans.push(Span::styled(
        format!("{:.0}% ", risk_pct),
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    ));

    // 5. Build/Designation Mode Indicator
    if build_mode.active {
        spans.push(Span::styled(
            "  [BUILD MODE] ",
            Style::default().fg(Color::Yellow).bg(Color::DarkGray),
        ));
        spans.push(Span::styled(
            format!(" Type: {} (Tab to cycle) ", build_mode.selected.label()),
            Style::default().fg(Color::White).bg(Color::DarkGray),
        ));
    } else if designation_mode.active {
        let tool_name = designation_mode.tool.label();
        spans.push(Span::styled(
            format!("  [{tool_name} MODE] "),
            Style::default().fg(Color::Magenta).bg(Color::DarkGray),
        ));
    }

    if let Some(name) = location_name {
        spans.push(Span::styled("  ║  ", Style::default().fg(Color::DarkGray)));
        spans.push(Span::styled(
            format!(" 📍 {name} "),
            Style::default().fg(Color::Cyan),
        ));
    }

    Line::from(spans)
}"""

new_func = """fn build_play_pause_span<'a>(paused: bool) -> Span<'a> {
    if paused {
        Span::styled(
            " ⏸ ",
            Style::default()
                .fg(Color::White)
                .bg(Color::Red)
                .add_modifier(Modifier::BOLD),
        )
    } else {
        Span::styled(
            " ▶ ",
            Style::default()
                .fg(Color::Black)
                .bg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )
    }
}

fn build_time_spans<'a>(tick: u64, season: Option<Season>, solar_cycle: Option<SolarCycle>) -> Vec<Span<'a>> {
    let mut spans = Vec::new();
    spans.push(Span::styled(
        format!(" Day {} ", tick),
        Style::default().add_modifier(Modifier::BOLD),
    ));

    if let Some(s) = season {
        let color = match s {
            Season::Spring => Color::Green,
            Season::Summer => Color::Yellow,
            Season::Autumn => Color::Rgb(200, 100, 0),
            Season::Winter => Color::Cyan,
        };
        spans.push(Span::styled(
            format!(" {} ", s.name()),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ));
    }

    if let Some(cycle) = solar_cycle {
        spans.push(Span::styled(
            format!(" {} ", cycle.label()),
            Style::default().fg(Color::Yellow),
        ));
    }
    spans
}

fn build_colony_stats_spans<'a>(pop_count: usize, morale: f32, efficiency: f32) -> Vec<Span<'a>> {
    let mut spans = Vec::new();
    spans.push(Span::styled("👨 ", Style::default().fg(Color::Cyan)));
    spans.push(Span::styled(
        format!("{} ", pop_count),
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    ));

    let morale_percent = (morale * 100.0).round() as u8;
    let morale_color = if morale < 0.3 {
        Color::Red
    } else if morale < 0.7 {
        Color::Yellow
    } else {
        Color::Green
    };
    spans.push(Span::styled(" 😊 ", Style::default().fg(morale_color)));
    spans.push(Span::styled(
        format!("{}% ", morale_percent),
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    ));

    let eff_percent = (efficiency * 100.0).round() as u8;
    let eff_color = if efficiency < 0.5 {
        Color::Red
    } else if efficiency < 0.8 {
        Color::Yellow
    } else {
        Color::Green
    };
    spans.push(Span::styled(" ⚙ ", Style::default().fg(eff_color)));
    spans.push(Span::styled(
        format!("{}% ", eff_percent),
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    ));
    spans
}

fn build_resources_spans<'a>(food_yield: f32, rations: f32, tools: f32, risk_pct: f32) -> Vec<Span<'a>> {
    let mut spans = Vec::new();
    let total_food = food_yield + rations;
    let food_color = if total_food < 10.0 {
        Color::Red
    } else {
        Color::Green
    };
    spans.push(Span::styled("🌾 ", Style::default().fg(food_color)));
    spans.push(Span::styled(
        format!("{:.0}+{:.0} ", food_yield, rations),
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    ));

    spans.push(Span::styled(" 🔨 ", Style::default().fg(Color::Yellow)));
    spans.push(Span::styled(
        format!("{:.0} ", tools),
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    ));

    let risk_color = if risk_pct > 80.0 {
        Color::White
    } else if risk_pct > 50.0 {
        Color::Yellow
    } else {
        Color::DarkGray
    };
    let risk_bg = if risk_pct > 80.0 {
        Color::Red
    } else {
        Color::Reset
    };
    spans.push(Span::styled(
        " 👁 ",
        Style::default().fg(risk_color).bg(risk_bg),
    ));
    spans.push(Span::styled(
        format!("{:.0}% ", risk_pct),
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    ));
    spans
}

fn build_mode_spans<'a>(build_mode: &BuildMode, designation_mode: &DesignationMode) -> Vec<Span<'a>> {
    let mut spans = Vec::new();
    if build_mode.active {
        spans.push(Span::styled(
            "  [BUILD MODE] ",
            Style::default().fg(Color::Yellow).bg(Color::DarkGray),
        ));
        spans.push(Span::styled(
            format!(" Type: {} (Tab to cycle) ", build_mode.selected.label()),
            Style::default().fg(Color::White).bg(Color::DarkGray),
        ));
    } else if designation_mode.active {
        let tool_name = designation_mode.tool.label();
        spans.push(Span::styled(
            format!("  [{tool_name} MODE] "),
            Style::default().fg(Color::Magenta).bg(Color::DarkGray),
        ));
    }
    spans
}

pub fn get_status_line<'a>(
    tick: u64,
    _speed: SimSpeed,
    paused: bool,
    build_mode: &BuildMode,
    designation_mode: &DesignationMode,
    location_name: Option<&'a str>,
    pop_count: usize,
    food_yield: f32,
    rations: f32,
    tools: f32,
    morale: f32,
    efficiency: f32,
    season: Option<Season>,
    solar_cycle: Option<SolarCycle>,
    risk_pct: f32,
) -> Line<'a> {
    let mut spans = Vec::new();

    spans.push(build_play_pause_span(paused));
    spans.extend(build_time_spans(tick, season, solar_cycle));
    spans.push(Span::styled("  ║  ", Style::default().fg(Color::DarkGray)));
    spans.extend(build_colony_stats_spans(pop_count, morale, efficiency));
    spans.push(Span::styled("  ║  ", Style::default().fg(Color::DarkGray)));
    spans.extend(build_resources_spans(food_yield, rations, tools, risk_pct));
    spans.extend(build_mode_spans(build_mode, designation_mode));

    if let Some(name) = location_name {
        spans.push(Span::styled("  ║  ", Style::default().fg(Color::DarkGray)));
        spans.push(Span::styled(
            format!(" 📍 {name} "),
            Style::default().fg(Color::Cyan),
        ));
    }

    Line::from(spans)
}"""

content = content.replace(old_func, new_func)

with open("src/ui/status.rs", "w") as f:
    f.write(content)

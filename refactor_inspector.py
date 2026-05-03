import re

with open("src/ui/inspector.rs", "r") as f:
    content = f.read()

# Refactor render_entity_inspector

old_func = """fn render_entity_inspector(frame: &mut Frame, area: Rect, world: &World, entity: Entity) {
    if !world.entities().contains(entity) {
        frame.render_widget(
            Paragraph::new("Entity Despawned").style(Style::default().fg(Color::Red)),
            area,
        );
        return;
    }

    let (name, color) = get_entity_header(world, entity);
    let action_line = get_action_line(world, entity);
    let generation_line = get_generation_line(world, entity);
    let info = get_inspector_layout_info(world, entity);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),                                    // Name
            Constraint::Length(1),                                    // Pos
            Constraint::Length(u16::from(generation_line.is_some())), // Generation
            Constraint::Length(u16::from(action_line.is_some())),     // Action
            Constraint::Length(1),                                    // Spacer
            Constraint::Length(info.details_height),                  // Needs or Details
            Constraint::Length(u16::from(info.has_structure)),        // Structure HP
            Constraint::Length(info.diag_height),                     // Diagnostics
            Constraint::Length(info.extra_height),                    // Extra Info
            Constraint::Length(info.personality_height),              // Personality
            Constraint::Length(info.dream_height),                    // Dream
            Constraint::Length(info.diet_height),                     // Diet
            Constraint::Min(1),                                       // Biography
        ])
        .split(area);

    frame.render_widget(
        Paragraph::new(Span::styled(
            name,
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        )),
        layout[0],
    );

    if let Some(pos) = world.get::<GridPosition>(entity) {
        frame.render_widget(
            Paragraph::new(format!("Position: ({}, {})", pos.x, pos.y))
                .style(Style::default().fg(Color::DarkGray)),
            layout[1],
        );
    }

    if let Some(line) = generation_line {
        frame.render_widget(Paragraph::new(line), layout[2]);
    }

    if let Some(line) = action_line {
        frame.render_widget(Paragraph::new(line), layout[3]);
    }

    let details_area = layout[5];
    if let Some(needs) = world.get::<Needs>(entity) {
        let bio_opt = world.get::<Biocompatibility>(entity);
        let health_opt = world.get::<Health>(entity);
        render_bio_monitor(frame, details_area, needs, bio_opt, health_opt);
    } else if let Some(housing) = world.get::<Housing>(entity) {
        render_housing_details(frame, details_area, housing);
    } else if let Some(farm) = world.get::<Farm>(entity) {
        render_farm_details(frame, details_area, farm);
    } else if let Some(stockpile) = world.get::<Stockpile>(entity) {
        render_stockpile_details(frame, details_area, stockpile);
    } else if let Some(progress) = world.get::<RefiningProgress>(entity) {
        render_refining_details(frame, details_area, progress);
    } else if let Some(obs) = world.get::<Observatory>(entity) {
        render_observatory_details(frame, details_area, obs, world);
    } else if let Some(fauna) = world.get::<NocturnalFauna>(entity) {
        render_nocturnal_fauna_details(frame, details_area, fauna, world);
    }

    if let Some(structure) = world.get::<Structure>(entity) {
        let pct = if structure.max_hp > 0.0 {
            (structure.current_hp / structure.max_hp * 100.0) as u16
        } else {
            0
        };
        let color = if pct > 66 {
            Color::Green
        } else if pct > 33 {
            Color::Yellow
        } else {
            Color::Red
        };
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::raw("HP: "),
                Span::styled(
                    format!("{:.0}/{:.0}", structure.current_hp, structure.max_hp),
                    Style::default().fg(color),
                ),
            ])),
            layout[6],
        );
    }

    if info.show_diagnostics {
        let diag_area = layout[7];
        let spirit_opt = world.get::<MachineSpirit>(entity);
        let quirk_opt = world.get::<Quirk>(entity);
        render_diagnostics(frame, diag_area, spirit_opt, quirk_opt);
    }

    if info.extra_height > 0 {
        let extra_area = layout[8];
        render_extra_info(frame, extra_area, info.extra_height, world, entity);
    }

    if let Some(weights) = world.get::<UtilityWeights>(entity) {
        render_personality(frame, layout[9], *weights);
    }

    if let Some(journal) = world.get::<DreamJournal>(entity) {
        render_dream_journal(frame, layout[10], journal);
    }

    if let Some(history) = world.get::<DietaryHistory>(entity) {
        render_dietary_history(frame, layout[11], history);
    }

    let bottom_area = layout[12];
    if let Some(bio) = world.get::<Biography>(entity) {
        render_biography(frame, bottom_area, bio, world);
    }
}"""

new_func = """fn render_specific_details(frame: &mut Frame, details_area: Rect, world: &World, entity: Entity) {
    if let Some(needs) = world.get::<Needs>(entity) {
        let bio_opt = world.get::<Biocompatibility>(entity);
        let health_opt = world.get::<Health>(entity);
        render_bio_monitor(frame, details_area, needs, bio_opt, health_opt);
        return;
    }
    if let Some(housing) = world.get::<Housing>(entity) {
        render_housing_details(frame, details_area, housing);
        return;
    }
    if let Some(farm) = world.get::<Farm>(entity) {
        render_farm_details(frame, details_area, farm);
        return;
    }
    if let Some(stockpile) = world.get::<Stockpile>(entity) {
        render_stockpile_details(frame, details_area, stockpile);
        return;
    }
    if let Some(progress) = world.get::<RefiningProgress>(entity) {
        render_refining_details(frame, details_area, progress);
        return;
    }
    if let Some(obs) = world.get::<Observatory>(entity) {
        render_observatory_details(frame, details_area, obs, world);
        return;
    }
    if let Some(fauna) = world.get::<NocturnalFauna>(entity) {
        render_nocturnal_fauna_details(frame, details_area, fauna, world);
    }
}

fn render_entity_inspector(frame: &mut Frame, area: Rect, world: &World, entity: Entity) {
    if !world.entities().contains(entity) {
        frame.render_widget(
            Paragraph::new("Entity Despawned").style(Style::default().fg(Color::Red)),
            area,
        );
        return;
    }

    let (name, color) = get_entity_header(world, entity);
    let action_line = get_action_line(world, entity);
    let generation_line = get_generation_line(world, entity);
    let info = get_inspector_layout_info(world, entity);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),                                    // Name
            Constraint::Length(1),                                    // Pos
            Constraint::Length(u16::from(generation_line.is_some())), // Generation
            Constraint::Length(u16::from(action_line.is_some())),     // Action
            Constraint::Length(1),                                    // Spacer
            Constraint::Length(info.details_height),                  // Needs or Details
            Constraint::Length(u16::from(info.has_structure)),        // Structure HP
            Constraint::Length(info.diag_height),                     // Diagnostics
            Constraint::Length(info.extra_height),                    // Extra Info
            Constraint::Length(info.personality_height),              // Personality
            Constraint::Length(info.dream_height),                    // Dream
            Constraint::Length(info.diet_height),                     // Diet
            Constraint::Min(1),                                       // Biography
        ])
        .split(area);

    frame.render_widget(
        Paragraph::new(Span::styled(
            name,
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        )),
        layout[0],
    );

    if let Some(pos) = world.get::<GridPosition>(entity) {
        frame.render_widget(
            Paragraph::new(format!("Position: ({}, {})", pos.x, pos.y))
                .style(Style::default().fg(Color::DarkGray)),
            layout[1],
        );
    }

    if let Some(line) = generation_line {
        frame.render_widget(Paragraph::new(line), layout[2]);
    }

    if let Some(line) = action_line {
        frame.render_widget(Paragraph::new(line), layout[3]);
    }

    render_specific_details(frame, layout[5], world, entity);

    if let Some(structure) = world.get::<Structure>(entity) {
        let pct = if structure.max_hp > 0.0 {
            (structure.current_hp / structure.max_hp * 100.0) as u16
        } else {
            0
        };
        let color = if pct > 66 {
            Color::Green
        } else if pct > 33 {
            Color::Yellow
        } else {
            Color::Red
        };
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::raw("HP: "),
                Span::styled(
                    format!("{:.0}/{:.0}", structure.current_hp, structure.max_hp),
                    Style::default().fg(color),
                ),
            ])),
            layout[6],
        );
    }

    if info.show_diagnostics {
        let diag_area = layout[7];
        let spirit_opt = world.get::<MachineSpirit>(entity);
        let quirk_opt = world.get::<Quirk>(entity);
        render_diagnostics(frame, diag_area, spirit_opt, quirk_opt);
    }

    if info.extra_height > 0 {
        let extra_area = layout[8];
        render_extra_info(frame, extra_area, info.extra_height, world, entity);
    }

    if let Some(weights) = world.get::<UtilityWeights>(entity) {
        render_personality(frame, layout[9], *weights);
    }

    if let Some(journal) = world.get::<DreamJournal>(entity) {
        render_dream_journal(frame, layout[10], journal);
    }

    if let Some(history) = world.get::<DietaryHistory>(entity) {
        render_dietary_history(frame, layout[11], history);
    }

    let bottom_area = layout[12];
    if let Some(bio) = world.get::<Biography>(entity) {
        render_biography(frame, bottom_area, bio, world);
    }
}"""

content = content.replace(old_func, new_func)

with open("src/ui/inspector.rs", "w") as f:
    f.write(content)

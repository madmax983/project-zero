#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]

use bevy_ecs::prelude::*;
use ratatui::{
    prelude::*,
    widgets::{
        Block, BorderType, Borders, Cell, Gauge, List, ListItem, Paragraph, Row, Table, Wrap,
    },
};

use crate::experimental::biography::Biography;
use crate::layer1::{
    ColonyResources, Farm, GridPosition, Housing, TerrainGrid, building::Building, needs::Needs,
    pop::Pop, thoughts::Thought,
};
use crate::shared::selection::{Selection, SelectionTarget};
use crate::ui::map::{get_building_color, get_terrain_char, get_terrain_color};

/// Renders the inspector panel content based on current selection.
pub fn render_inspector(frame: &mut Frame, area: Rect, world: &World) {
    let selection = world.resource::<Selection>();

    match selection.target() {
        SelectionTarget::None => render_colony_stats(frame, area, world),
        SelectionTarget::Tile(x, y) => render_tile_inspector(frame, area, world, x, y),
        SelectionTarget::Entity(entity) => render_entity_inspector(frame, area, world, entity),
    }
}

fn render_colony_stats(frame: &mut Frame, area: Rect, world: &World) {
    let resources = world.resource::<ColonyResources>();

    // Calculate population stats
    let pop_count = world
        .iter_entities()
        .filter(bevy_ecs::world::EntityRef::contains::<Pop>)
        .count();

    let (housing_count, housing_capacity, housing_used) = world
        .iter_entities()
        .filter_map(|e| e.get::<Housing>())
        .fold((0, 0, 0), |(count, cap, used), h| {
            (count + 1, cap + h.capacity, used + h.residents.len())
        });

    let (farm_count, farm_capacity, farm_used) = world
        .iter_entities()
        .filter_map(|e| e.get::<Farm>())
        .fold((0, 0, 0), |(count, cap, used), f| {
            (count + 1, cap + f.capacity, used + f.workers.len())
        });

    let rows = vec![
        Row::new(vec![
            Cell::from("Population").style(Style::default().fg(Color::Cyan)),
            Cell::from(pop_count.to_string()),
        ]),
        Row::new(vec![
            Cell::from("Housing").style(Style::default().fg(Color::Cyan)),
            Cell::from(format!(
                "{housing_used}/{housing_capacity} ({housing_count} buildings)"
            )),
        ]),
        Row::new(vec![
            Cell::from("Workers").style(Style::default().fg(Color::Cyan)),
            Cell::from(format!("{farm_used}/{farm_capacity} ({farm_count} farms)")),
        ]),
        Row::new(vec![Cell::from("")]), // Spacer
        Row::new(vec![
            Cell::from("Food").style(Style::default().fg(Color::Yellow)),
            Cell::from(format!("{:.1}/{:.0}", resources.food, resources.max_food)),
        ]),
        Row::new(vec![
            Cell::from("Wood").style(Style::default().fg(Color::Green)),
            Cell::from(format!("{:.1}/{:.0}", resources.wood, resources.max_wood)),
        ]),
        Row::new(vec![
            Cell::from("Stone").style(Style::default().fg(Color::Gray)),
            Cell::from(format!("{:.1}/{:.0}", resources.stone, resources.max_stone)),
        ]),
    ];

    let table = Table::new(
        rows,
        [Constraint::Percentage(40), Constraint::Percentage(60)],
    )
    .block(Block::default().borders(Borders::NONE)); // Parent has borders

    frame.render_widget(table, area);
}

fn render_tile_inspector(frame: &mut Frame, area: Rect, world: &World, x: i32, y: i32) {
    let terrain = world.resource::<TerrainGrid>();

    if x < 0 || y < 0 {
        let text = Paragraph::new("Outside Map").style(Style::default().fg(Color::Red));
        frame.render_widget(text, area);
        return;
    }

    let (name, char, color) = terrain
        .get(x as usize, y as usize)
        .map_or(("Unknown", "?", Color::Red), |t| {
            (t.name(), get_terrain_char(t), get_terrain_color(t))
        });

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Header
            Constraint::Length(1), // Coords
            Constraint::Min(1),    // Visual
        ])
        .split(area);

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::raw("Terrain: "),
            Span::styled(
                name,
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            ),
        ])),
        layout[0],
    );

    frame.render_widget(
        Paragraph::new(format!("Position: ({x}, {y})")).style(Style::default().fg(Color::DarkGray)),
        layout[1],
    );

    // Big visual representation
    let visual_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(" Visual ");
    let visual_inner = visual_block.inner(layout[2]);
    frame.render_widget(visual_block, layout[2]);

    let visual = Paragraph::new(char)
        .style(Style::default().fg(color))
        .alignment(Alignment::Center);
    // .block(Block::default().borders(Borders::NONE)); // Centered inside block

    // Center the char vertically
    let v_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Length(1),
            Constraint::Percentage(50),
        ])
        .split(visual_inner);

    frame.render_widget(visual, v_layout[1]);
}

fn render_entity_inspector(frame: &mut Frame, area: Rect, world: &World, entity: Entity) {
    if !world.entities().contains(entity) {
        frame.render_widget(
            Paragraph::new("Entity Despawned").style(Style::default().fg(Color::Red)),
            area,
        );
        return;
    }

    // Determine Entity Type and Name
    let (name, color) = if world.get::<Pop>(entity).is_some() {
        ("Colonist", Color::Yellow)
    } else if let Some(b) = world.get::<Building>(entity) {
        (b.building_type.label(), get_building_color(b.building_type))
    } else {
        ("Entity", Color::White)
    };

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Name
            Constraint::Length(1), // Pos
            Constraint::Length(1), // Spacer
            Constraint::Length(3), // Needs (if any)
            Constraint::Length(1), // Spacer
            Constraint::Min(1),    // Thoughts/Extra
        ])
        .split(area);

    // 1. Name
    frame.render_widget(
        Paragraph::new(Span::styled(
            name,
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        )),
        layout[0],
    );

    // 2. Position
    if let Some(pos) = world.get::<GridPosition>(entity) {
        frame.render_widget(
            Paragraph::new(format!("Position: ({}, {})", pos.x, pos.y))
                .style(Style::default().fg(Color::DarkGray)),
            layout[1],
        );
    }

    // 3. Needs (Pops only)
    if let Some(needs) = world.get::<Needs>(entity) {
        let needs_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50),
                Constraint::Length(1),
                Constraint::Percentage(50),
            ])
            .split(layout[3]);

        let hunger_percent = (needs.hunger * 100.0) as u16;
        let rest_percent = (needs.rest * 100.0) as u16;

        let hunger_color = if needs.hunger < 0.3 {
            Color::Red
        } else {
            Color::Green
        };
        let rest_color = if needs.rest < 0.3 {
            Color::Red
        } else {
            Color::Cyan
        };

        let hunger_gauge = Gauge::default()
            .block(Block::default().title("Hunger").borders(Borders::NONE))
            .gauge_style(Style::default().fg(hunger_color))
            .percent(hunger_percent);

        let rest_gauge = Gauge::default()
            .block(Block::default().title("Rest").borders(Borders::NONE))
            .gauge_style(Style::default().fg(rest_color))
            .percent(rest_percent);

        frame.render_widget(hunger_gauge, needs_layout[0]);
        frame.render_widget(rest_gauge, needs_layout[2]);
    }

    // 4. Thoughts & Biography
    let bottom_area = layout[5];
    let thought_opt = world.get::<Thought>(entity);
    let bio_opt = world.get::<Biography>(entity);

    if let Some(thought) = thought_opt {
        if let Some(bio) = bio_opt {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Min(1)])
                .split(bottom_area);

            render_thought(frame, chunks[0], thought);
            render_biography(frame, chunks[1], bio);
        } else {
            render_thought(frame, bottom_area, thought);
        }
    } else if let Some(bio) = bio_opt {
        render_biography(frame, bottom_area, bio);
    }
}

fn render_thought(frame: &mut Frame, area: Rect, thought: &Thought) {
    let thought_block = Block::default()
        .borders(Borders::TOP)
        .title(" Thoughts ")
        .title_style(Style::default().fg(Color::Magenta));

    let thought_text = Paragraph::new(format!("\"{}\"", thought.text))
        .wrap(Wrap { trim: true })
        .style(
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::ITALIC),
        )
        .block(thought_block);

    frame.render_widget(thought_text, area);
}

fn render_biography(frame: &mut Frame, area: Rect, bio: &Biography) {
    let bio_block = Block::default()
        .borders(Borders::TOP)
        .title(" Biography ")
        .title_style(Style::default().fg(Color::Blue));

    // Show last 5 events reversed
    let events: Vec<ListItem> = bio
        .events
        .iter()
        .rev()
        .take(5)
        .map(|e| {
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!("[{}] ", e.tick),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::raw(&e.text),
            ]))
        })
        .collect();

    let list = List::new(events).block(bio_block);

    frame.render_widget(list, area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;

    #[test]
    fn test_inspector_render_pop() {
        let mut world = World::new();
        // Setup world
        world.insert_resource(Selection::default());
        let entity = world
            .spawn((
                Pop,
                GridPosition { x: 1, y: 1 },
                Needs {
                    hunger: 0.5,
                    rest: 0.8,
                },
                Thought {
                    text: "Thinking...".to_string(),
                    tick: 0,
                },
            ))
            .id();

        world.resource_mut::<Selection>().select_entity(entity);

        // Setup Terminal
        let backend = TestBackend::new(40, 20);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|f| {
                render_inspector(f, f.area(), &world);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();

        // Assert content
        // Convert buffer to string for checking
        let _content = format!("{buffer:?}"); // Debug representation

        // Check for specific strings
        // We can't easily check full content but we can check cells exist with text
        let cells: Vec<String> = buffer
            .content
            .iter()
            .map(|c| c.symbol().to_string())
            .collect();
        let full_text = cells.join("");

        // Check "Colonist" title
        assert!(full_text.contains("Colonist"));
        // Check Needs Gauge Titles (might be part of block titles or implicit?)
        // The Gauge widget doesn't inherently render its title unless it's in a Block.
        // We put title "Hunger" in the block.
        assert!(full_text.contains("Hunger"));
        assert!(full_text.contains("Rest"));

        // Check Thought
        assert!(full_text.contains("Thinking..."));
    }
}

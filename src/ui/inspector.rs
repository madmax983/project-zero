#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::option_if_let_else)]

//! Inspector panel rendering.
//!
//! The inspector provides context-sensitive details about the currently selected entity or tile.
//! It updates dynamically based on the [`crate::shared::selection::Selection`] resource.

use bevy_ecs::prelude::*;
use ratatui::{
    prelude::*,
    widgets::{Block, BorderType, Borders, Cell, Gauge, List, ListItem, Paragraph, Row, Table},
};

use crate::experimental::biography::Biography;
use crate::layer1::{
    ActionType, ColonyResources, Farm, GridPosition, Housing, PopAction, TerrainGrid,
    building::Building,
    needs::Needs,
    pop::{Pop, PopName},
    resources::RefiningProgress,
    stockpile::Stockpile,
    structure::Structure,
};
use crate::shared::selection::{Selection, SelectionTarget};
use crate::ui::map::{get_building_color, get_terrain_char, get_terrain_color};

/// Renders the inspector panel content based on current selection.
///
/// Dispatches rendering to specific helpers based on the [`SelectionTarget`]:
/// - [`SelectionTarget::None`] -> Colony stats (global overview).
/// - [`SelectionTarget::Tile`] -> Tile inspector (terrain info).
/// - [`SelectionTarget::Entity`] -> Entity inspector (pop/building details).
pub fn render_inspector(frame: &mut Frame, area: Rect, world: &World) {
    let selection = world.resource::<Selection>();

    match selection.target() {
        SelectionTarget::None => render_colony_stats(frame, area, world),
        SelectionTarget::Tile(x, y) => render_tile_inspector(frame, area, world, x, y),
        SelectionTarget::Entity(entity) => render_entity_inspector(frame, area, world, entity),
    }
}

#[allow(clippy::too_many_lines)]
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

    // Split layout into two sections
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6), // Demographics
            Constraint::Min(9),    // Resources
        ])
        .split(area);

    // --- Demographics Table ---
    let demo_rows = vec![
        Row::new(vec![
            Cell::from("👥 Population").style(Style::default().fg(Color::Cyan)),
            Cell::from(pop_count.to_string()),
        ]),
        Row::new(vec![
            Cell::from("🏠 Housing").style(Style::default().fg(Color::Cyan)),
            Cell::from(format!(
                "{housing_used}/{housing_capacity} ({housing_count})"
            )),
        ]),
        Row::new(vec![
            Cell::from("⚒  Workers").style(Style::default().fg(Color::Cyan)),
            Cell::from(format!("{farm_used}/{farm_capacity} ({farm_count})")),
        ]),
    ];

    let demo_table = Table::new(
        demo_rows,
        [Constraint::Percentage(50), Constraint::Percentage(50)],
    )
    .block(
        Block::default()
            .title(" Demographics ")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Cyan)),
    );

    frame.render_widget(demo_table, chunks[0]);

    // --- Resources Table ---
    let resource_rows = vec![
        Row::new(vec![
            Cell::from("🍖 Food").style(Style::default().fg(Color::Yellow)),
            Cell::from(format!("{:.1}/{:.0}", resources.food, resources.max_food)),
        ]),
        Row::new(vec![
            Cell::from("🌲 Wood").style(Style::default().fg(Color::Green)),
            Cell::from(format!("{:.1}/{:.0}", resources.wood, resources.max_wood)),
        ]),
        Row::new(vec![
            Cell::from("🪨 Stone").style(Style::default().fg(Color::Gray)),
            Cell::from(format!("{:.1}/{:.0}", resources.stone, resources.max_stone)),
        ]),
        Row::new(vec![
            Cell::from("🔧 Tools").style(Style::default().fg(Color::Cyan)),
            Cell::from(format!("{:.1}/{:.0}", resources.tools, resources.max_tools)),
        ]),
        Row::new(vec![
            Cell::from("Fb Fiber").style(Style::default().fg(Color::Green)),
            Cell::from(format!("{:.1}/{:.0}", resources.fiber, resources.max_fiber)),
        ]),
        Row::new(vec![
            Cell::from("Cl Cloth").style(Style::default().fg(Color::Magenta)),
            Cell::from(format!("{:.1}/{:.0}", resources.cloth, resources.max_cloth)),
        ]),
        Row::new(vec![
            Cell::from("Cg Clothing").style(Style::default().fg(Color::LightMagenta)),
            Cell::from(format!(
                "{:.1}/{:.0}",
                resources.clothing, resources.max_clothing
            )),
        ]),
    ];

    let resource_table = Table::new(
        resource_rows,
        [Constraint::Percentage(50), Constraint::Percentage(50)],
    )
    .block(
        Block::default()
            .title(" Resources ")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Yellow)),
    );

    frame.render_widget(resource_table, chunks[1]);
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

#[allow(clippy::too_many_lines)]
fn render_entity_inspector(frame: &mut Frame, area: Rect, world: &World, entity: Entity) {
    if !world.entities().contains(entity) {
        frame.render_widget(
            Paragraph::new("Entity Despawned").style(Style::default().fg(Color::Red)),
            area,
        );
        return;
    }

    // Determine Entity Type and Name
    let (name, color): (String, Color) = if let Some(pop_name) = world.get::<PopName>(entity) {
        (pop_name.0.clone(), Color::Yellow)
    } else if world.get::<Pop>(entity).is_some() {
        ("Colonist".to_string(), Color::Yellow)
    } else if let Some(b) = world.get::<Building>(entity) {
        (
            b.building_type.label().to_string(),
            get_building_color(b.building_type),
        )
    } else {
        ("Entity".to_string(), Color::White)
    };

    // Determine Action if Pop
    let action_line = if let Some(action) = world.get::<PopAction>(entity) {
        let (icon, label) = match action.current {
            ActionType::SatisfyHunger => ("🍖", "Eating"),
            ActionType::SatisfyRest => ("💤", "Sleeping"),
            ActionType::Socialize => ("💬", "Socializing"),
            ActionType::Explore => ("🔭", "Exploring"),
            ActionType::Work => ("⚒", "Working"),
            ActionType::Repair => ("🔧", "Repairing"),
            ActionType::Research => ("📚", "Researching"),
            ActionType::Haul => ("📦", "Hauling"),
            ActionType::SeekMedicalCare => ("🏥", "Healing"),
            ActionType::BuryCorpse => ("⚰️", "Burying"),
            ActionType::FetchTool => ("🔧", "Fetching Tool"),
            ActionType::Idle => ("⏳", "Idle"),
            ActionType::Vandalize => ("🔨", "Vandalizing"),
            ActionType::Binge => ("🍖", "Bingeing"),
            ActionType::Daze => ("😵", "Dazed"),
        };
        Some(Line::from(vec![
            Span::raw("Action: "),
            Span::styled(format!("{icon} {label}"), Style::default().fg(Color::White)),
        ]))
    } else {
        None
    };

    // Dynamic height for details section
    let details_height = if world.get::<Needs>(entity).is_some() {
        5
    } else {
        6
    };

    let has_structure = world.get::<Structure>(entity).is_some();

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),                                // Name
            Constraint::Length(1),                                // Pos
            Constraint::Length(u16::from(action_line.is_some())), // Action
            Constraint::Length(1),                                // Spacer
            Constraint::Length(details_height),                   // Needs or Details
            Constraint::Length(u16::from(has_structure)),         // Structure HP
            Constraint::Length(1),                                // Spacer
            Constraint::Min(1),                                   // Thoughts/Extra
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

    // 3. Action
    if let Some(line) = action_line {
        frame.render_widget(Paragraph::new(line), layout[2]);
    }

    // 4. Needs or Building Details
    let details_area = layout[4];
    if let Some(needs) = world.get::<Needs>(entity) {
        // Split into two rows: gauges on top, morale below
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Length(1)])
            .split(details_area);

        let needs_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50),
                Constraint::Length(1),
                Constraint::Percentage(50),
            ])
            .split(rows[0]);

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
            .label(format!("🍖 {hunger_percent}%"))
            .percent(hunger_percent);

        let rest_gauge = Gauge::default()
            .block(Block::default().title("Rest").borders(Borders::NONE))
            .gauge_style(Style::default().fg(rest_color))
            .label(format!("💤 {rest_percent}%"))
            .percent(rest_percent);

        frame.render_widget(hunger_gauge, needs_layout[0]);
        frame.render_widget(rest_gauge, needs_layout[2]);

        // Morale row
        let morale = needs.morale();
        let morale_percent = (morale * 100.0) as u16;
        let morale_color = if morale < 0.3 {
            Color::Red
        } else if morale < 0.7 {
            Color::Yellow
        } else {
            Color::Green
        };
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::raw("Morale: "),
                Span::styled(
                    format!("{morale_percent}%"),
                    Style::default().fg(morale_color),
                ),
            ])),
            rows[1],
        );
    } else if let Some(housing) = world.get::<Housing>(entity) {
        render_housing_details(frame, details_area, housing);
    } else if let Some(farm) = world.get::<Farm>(entity) {
        render_farm_details(frame, details_area, farm);
    } else if let Some(stockpile) = world.get::<Stockpile>(entity) {
        render_stockpile_details(frame, details_area, stockpile);
    } else if let Some(progress) = world.get::<RefiningProgress>(entity) {
        render_refining_details(frame, details_area, progress);
    }

    // 5. Structure HP
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
            layout[5],
        );
    }

    // 6. Biography
    let bottom_area = layout[7];
    let bio_opt = world.get::<Biography>(entity);

    if let Some(bio) = bio_opt {
        render_biography(frame, bottom_area, bio);
    }
}

fn render_housing_details(frame: &mut Frame, area: Rect, housing: &Housing) {
    let residents_count = housing.residents.len();
    let capacity = housing.capacity;
    let percent = (residents_count as f32 / capacity as f32).clamp(0.0, 1.0);

    let gauge = Gauge::default()
        .block(
            Block::default()
                .title(" Housing ")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .gauge_style(Style::default().fg(if residents_count >= capacity {
            Color::Red
        } else {
            Color::Green
        }))
        .label(format!("{residents_count}/{capacity}"))
        .percent((percent * 100.0) as u16);

    frame.render_widget(gauge, area);
}

fn render_farm_details(frame: &mut Frame, area: Rect, farm: &Farm) {
    let workers_count = farm.workers.len();
    let capacity = farm.capacity;
    let percent = (workers_count as f32 / capacity as f32).clamp(0.0, 1.0);

    let gauge = Gauge::default()
        .block(
            Block::default()
                .title(" Farm ")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Yellow)),
        )
        .gauge_style(Style::default().fg(if workers_count >= capacity {
            Color::Green
        } else {
            Color::Yellow
        }))
        .label(format!("{workers_count}/{capacity}"))
        .percent((percent * 100.0) as u16);

    frame.render_widget(gauge, area);
}

fn render_stockpile_details(frame: &mut Frame, area: Rect, stockpile: &Stockpile) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Storage Bonus ")
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Gray));

    let mut lines = Vec::new();
    if stockpile.food_bonus > 0.0 {
        lines.push(Line::from(vec![
            Span::raw("Food: +"),
            Span::styled(
                format!("{:.0}", stockpile.food_bonus),
                Style::default().fg(Color::Yellow),
            ),
        ]));
    }
    if stockpile.wood_bonus > 0.0 {
        lines.push(Line::from(vec![
            Span::raw("Wood: +"),
            Span::styled(
                format!("{:.0}", stockpile.wood_bonus),
                Style::default().fg(Color::Green),
            ),
        ]));
    }
    if stockpile.stone_bonus > 0.0 {
        lines.push(Line::from(vec![
            Span::raw("Stone: +"),
            Span::styled(
                format!("{:.0}", stockpile.stone_bonus),
                Style::default().fg(Color::DarkGray),
            ),
        ]));
    }
    if stockpile.waste_bonus > 0.0 {
        lines.push(Line::from(vec![
            Span::raw("Waste: +"),
            Span::styled(
                format!("{:.0}", stockpile.waste_bonus),
                Style::default().fg(Color::Rgb(85, 107, 47)),
            ),
        ]));
    }

    let p = Paragraph::new(lines)
        .block(block)
        .alignment(Alignment::Left);

    frame.render_widget(p, area);
}

fn render_refining_details(frame: &mut Frame, area: Rect, progress: &RefiningProgress) {
    let gauge = Gauge::default()
        .block(
            Block::default()
                .title(" Production ")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .gauge_style(Style::default().fg(Color::LightGreen))
        .percent(((progress.current / progress.max) * 100.0) as u16);

    frame.render_widget(gauge, area);
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
        use crate::layer1::pop::PopName;

        let mut world = World::new();
        // Setup world
        world.insert_resource(Selection::default());
        let entity = world
            .spawn((
                Pop,
                PopName("Ada".to_string()),
                GridPosition { x: 1, y: 1 },
                Needs {
                    hunger: 0.5,
                    rest: 0.8,
                    ..Default::default()
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

        let cells: Vec<String> = buffer
            .content
            .iter()
            .map(|c| c.symbol().to_string())
            .collect();
        let full_text = cells.join("");

        // Check pop name appears instead of generic "Colonist"
        assert!(full_text.contains("Ada"));
        assert!(full_text.contains("Hunger"));
        assert!(full_text.contains("Rest"));
    }

    #[test]
    fn test_inspector_render_pop_morale() {
        use crate::layer1::pop::PopName;

        let mut world = World::new();
        world.insert_resource(Selection::default());
        let entity = world
            .spawn((
                Pop,
                PopName("Bryn".to_string()),
                GridPosition { x: 2, y: 3 },
                Needs {
                    hunger: 0.8,
                    rest: 0.9,
                    leisure: 0.6,
                },
            ))
            .id();

        world.resource_mut::<Selection>().select_entity(entity);

        let backend = TestBackend::new(40, 20);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|f| {
                render_inspector(f, f.area(), &world);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let cells: Vec<String> = buffer
            .content
            .iter()
            .map(|c| c.symbol().to_string())
            .collect();
        let full_text = cells.join("");

        assert!(full_text.contains("Bryn"));
        assert!(full_text.contains("Morale"));
    }

    #[test]
    fn test_inspector_render_stockpile() {
        use crate::layer1::building::BuildingType;

        let mut world = World::new();
        world.insert_resource(Selection::default());
        let entity = world
            .spawn((
                Building {
                    building_type: BuildingType::Stockpile,
                },
                Stockpile {
                    food_bonus: 0.0,
                    wood_bonus: 100.0,
                    stone_bonus: 50.0,
                    waste_bonus: 0.0,
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        world.resource_mut::<Selection>().select_entity(entity);

        let backend = TestBackend::new(40, 20);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|f| {
                render_inspector(f, f.area(), &world);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let cells: Vec<String> = buffer
            .content
            .iter()
            .map(|c| c.symbol().to_string())
            .collect();
        let full_text = cells.join("");

        assert!(full_text.contains("Stockpile"));
        assert!(full_text.contains("Storage Bonus"));
        assert!(full_text.contains("Wood: +100"));
        assert!(full_text.contains("Stone: +50"));
        // Food is 0, so it should NOT be there
        assert!(!full_text.contains("Food: +0"));
    }

    #[test]
    fn test_inspector_render_refining() {
        use crate::layer1::building::BuildingType;

        let mut world = World::new();
        world.insert_resource(Selection::default());
        let entity = world
            .spawn((
                Building {
                    building_type: BuildingType::LumberMill,
                },
                RefiningProgress {
                    current: 50.0,
                    max: 100.0,
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        world.resource_mut::<Selection>().select_entity(entity);

        let backend = TestBackend::new(40, 20);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|f| {
                render_inspector(f, f.area(), &world);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let cells: Vec<String> = buffer
            .content
            .iter()
            .map(|c| c.symbol().to_string())
            .collect();
        let full_text = cells.join("");

        assert!(full_text.contains("Lumber Mill"));
        // "Production" is the title of the gauge block
        assert!(full_text.contains("Production"));
    }
}

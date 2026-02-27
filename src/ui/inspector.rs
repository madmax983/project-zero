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

use crate::layer1::biography::Biography;
use crate::layer1::day_night::DayNightCycle;
use crate::layer1::dreams::DreamJournal;
use crate::layer1::purity::PurityMap;
use crate::layer1::rituals::{MachineSpirit, Quirk, QuirkType};
use crate::layer1::social::old_guard::{Arrival, Generation};
use crate::layer1::utility_types::UtilityWeights;
use crate::layer1::{
    ActionType, Biocompatibility, ColonyResources, Farm, GridPosition, Housing, PopAction,
    TerrainGrid,
    building::Building,
    building::Material,
    building::MaterialType,
    needs::Needs,
    palette_fatigue::DietaryHistory,
    pop::{Pop, PopName},
    resources::RefiningProgress,
    stockpile::Stockpile,
    structure::Structure,
};
use crate::shared::selection::{Selection, SelectionTarget};
use crate::ui::map::{get_building_color, get_terrain_char, get_terrain_color};

/// Helper to format `ActionType` into an icon and label.
const fn format_action_type(action: ActionType) -> (&'static str, &'static str, Color) {
    match action {
        ActionType::SatisfyHunger => ("🍖", "Eating", Color::Green),
        ActionType::SatisfyRest => ("💤", "Sleeping", Color::Blue),
        ActionType::Socialize => ("💬", "Socializing", Color::Yellow),
        ActionType::Explore => ("🔭", "Exploring", Color::Cyan),
        ActionType::Work => ("⚒", "Working", Color::White),
        ActionType::Repair => ("🔧", "Repairing", Color::White),
        ActionType::Research => ("📚", "Researching", Color::Magenta),
        ActionType::Haul => ("📦", "Hauling", Color::Gray),
        ActionType::SeekMedicalCare => ("🏥", "Healing", Color::Red),
        ActionType::BuryCorpse => ("⚰️", "Burying", Color::DarkGray),
        ActionType::FetchTool => ("🔧", "Fetching Tool", Color::Gray),
        ActionType::Idle => ("⏳", "Idle", Color::DarkGray),
        ActionType::Vandalize => ("🔨", "Vandalizing", Color::Red),
        ActionType::Binge => ("🍖", "Bingeing", Color::Red),
        ActionType::Daze => ("😵", "Dazed", Color::Magenta),
        ActionType::Fight => ("⚔️", "Fighting", Color::Red),
        ActionType::Refine => ("⚙️", "Refining", Color::White),
        ActionType::Farm => ("🌾", "Farming", Color::Green),
        ActionType::Warden => ("👮", "Arresting", Color::Blue),
        ActionType::Sleepwalking => ("💤", "Sleepwalking", Color::Magenta),
        ActionType::Tame => ("♥", "Taming", Color::LightGreen),
        ActionType::FireStarting => ("🔥", "Starting Fire", Color::Red),
        ActionType::HideInRoom => ("🚪", "Hiding", Color::DarkGray),
        ActionType::SadWander => ("😢", "Wandering Sadly", Color::Blue),
        ActionType::FetchClothing => ("👕", "Fetching Clothes", Color::Cyan),
        ActionType::Surgery => ("🏥", "Undergoing Surgery", Color::Red),
        ActionType::Charge => ("⚡", "Charging", Color::Cyan),
        ActionType::Hobby => ("🎨", "Hobby", Color::Magenta),
        ActionType::Admin => ("📝", "Administering", Color::Blue),
        ActionType::ScrawlMemeticSigil => ("👁", "Scrawling Sigil", Color::Red),
        ActionType::PreCrimeArrest => ("🛡", "Pre-Crime Arrest", Color::Blue),
        ActionType::ConsumeChemical => ("💊", "Consuming", Color::Magenta),
        ActionType::CollectSample => ("🧬", "Collecting", Color::Cyan),
        ActionType::UseShower => ("🚿", "Showering", Color::Cyan),
        ActionType::ListenToTheHum => ("🌀", "Listening", Color::Magenta),
        ActionType::Clean => ("🧹", "Cleaning", Color::Yellow),
    }
}

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

fn get_resource_color(current: f32, max: f32, inverse: bool) -> Color {
    if max <= f32::EPSILON {
        return Color::Gray;
    }
    let pct = current / max;
    if inverse {
        if pct < 0.2 {
            Color::Green
        } else if pct < 0.8 {
            Color::Yellow
        } else {
            Color::Red
        }
    } else if pct < 0.2 {
        Color::Red
    } else if pct < 0.5 {
        Color::Yellow
    } else {
        Color::Green
    }
}

#[allow(clippy::too_many_lines)]
fn render_colony_stats(frame: &mut Frame, area: Rect, world: &World) {
    let resources = world.resource::<ColonyResources>();

    // Calculate population stats
    let (pop_count, total_morale) = world
        .iter_entities()
        .filter_map(|e| e.get::<Needs>())
        .fold((0, 0.0), |(count, sum), needs| {
            (count + 1, sum + needs.morale())
        });

    let avg_morale = if pop_count > 0 {
        total_morale / pop_count as f32
    } else {
        0.0
    };

    let (_housing_count, housing_capacity, housing_used) = world
        .iter_entities()
        .filter_map(|e| e.get::<Housing>())
        .fold((0, 0, 0), |(count, cap, used), h| {
            (count + 1, cap + h.capacity, used + h.residents.len())
        });

    // Dashboard Layout
    // 1. Status (Top)
    // 2. Survival
    // 3. Industry
    // 4. Economy
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5), // Status (Pop + Morale)
            Constraint::Length(6), // Survival
            Constraint::Length(6), // Industry
            Constraint::Min(6),    // Economy
        ])
        .split(area);

    // --- 1. Status Module ---
    let morale_color = if avg_morale > 0.7 {
        Color::Green
    } else if avg_morale > 0.4 {
        Color::Yellow
    } else {
        Color::Red
    };

    let status_rows = vec![
        Row::new(vec![
            Cell::from("👥 Pop").style(Style::default().fg(Color::Cyan)),
            Cell::from(format!("{pop_count}")),
        ]),
        Row::new(vec![
            Cell::from("😃 Morale").style(Style::default().fg(Color::Cyan)),
            Cell::from(format!("{:.0}%", avg_morale * 100.0))
                .style(Style::default().fg(morale_color)),
        ]),
        Row::new(vec![
            Cell::from("🏠 Housing").style(Style::default().fg(Color::Cyan)),
            Cell::from(format!("{housing_used}/{housing_capacity}")),
        ]),
    ];

    let status_table = Table::new(
        status_rows,
        [Constraint::Percentage(50), Constraint::Percentage(50)],
    )
    .block(
        Block::default()
            .title(" Status ")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Cyan)),
    );
    frame.render_widget(status_table, chunks[0]);

    // --- 2. Survival Module ---
    let survival_rows = vec![
        Row::new(vec![
            Cell::from("🍖 Food").style(Style::default().fg(Color::Green)),
            Cell::from(format!(
                "{:.0}/{:.0}",
                resources.total_food(),
                resources.max_food
            ))
            .style(Style::default().fg(get_resource_color(
                resources.total_food(),
                resources.max_food,
                false,
            ))),
        ]),
        Row::new(vec![
            Cell::from("💧 Water").style(Style::default().fg(Color::Blue)),
            Cell::from(format!("{:.0}/{:.0}", resources.water, resources.max_water)).style(
                Style::default().fg(get_resource_color(
                    resources.water,
                    resources.max_water,
                    false,
                )),
            ),
        ]),
        Row::new(vec![
            Cell::from("⛽ Fuel").style(Style::default().fg(Color::Yellow)),
            Cell::from(format!("{:.0}/{:.0}", resources.fuel, resources.max_fuel)).style(
                Style::default().fg(get_resource_color(
                    resources.fuel,
                    resources.max_fuel,
                    false,
                )),
            ),
        ]),
        Row::new(vec![
            Cell::from("🗑 Waste").style(Style::default().fg(Color::DarkGray)),
            Cell::from(format!("{:.0}/{:.0}", resources.waste, resources.max_waste)).style(
                Style::default().fg(get_resource_color(
                    resources.waste,
                    resources.max_waste,
                    true,
                )),
            ),
        ]),
    ];

    let survival_table = Table::new(
        survival_rows,
        [Constraint::Percentage(50), Constraint::Percentage(50)],
    )
    .block(
        Block::default()
            .title(" Survival ")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Green)),
    );
    frame.render_widget(survival_table, chunks[1]);

    // --- 3. Construction & Industry Module ---
    let industry_rows = vec![
        Row::new(vec![
            Cell::from("🌲 Wood").style(Style::default().fg(Color::White)),
            Cell::from(format!("{:.0}/{:.0}", resources.wood, resources.max_wood)).style(
                Style::default().fg(get_resource_color(
                    resources.wood,
                    resources.max_wood,
                    false,
                )),
            ),
        ]),
        Row::new(vec![
            Cell::from("🪨 Stone").style(Style::default().fg(Color::Gray)),
            Cell::from(format!("{:.0}/{:.0}", resources.stone, resources.max_stone)).style(
                Style::default().fg(get_resource_color(
                    resources.stone,
                    resources.max_stone,
                    false,
                )),
            ),
        ]),
        Row::new(vec![
            Cell::from("⚙ Metal").style(Style::default().fg(Color::LightBlue)),
            Cell::from(format!("{:.0}/{:.0}", resources.metal, resources.max_metal)).style(
                Style::default().fg(get_resource_color(
                    resources.metal,
                    resources.max_metal,
                    false,
                )),
            ),
        ]),
        Row::new(vec![
            Cell::from("🔧 Tools").style(Style::default().fg(Color::Cyan)),
            Cell::from(format!("{:.0}/{:.0}", resources.tools, resources.max_tools)).style(
                Style::default().fg(get_resource_color(
                    resources.tools,
                    resources.max_tools,
                    false,
                )),
            ),
        ]),
    ];

    let industry_table = Table::new(
        industry_rows,
        [Constraint::Percentage(50), Constraint::Percentage(50)],
    )
    .block(
        Block::default()
            .title(" Industry ")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Gray)),
    );
    frame.render_widget(industry_table, chunks[2]);

    // --- 4. Economy & Science Module ---
    let economy_rows = vec![
        Row::new(vec![
            Cell::from("🔬 Tech").style(Style::default().fg(Color::Magenta)),
            Cell::from(format!(
                "{:.0}/{:.0}",
                resources.knowledge, resources.max_knowledge
            ))
            .style(Style::default().fg(get_resource_color(
                resources.knowledge,
                resources.max_knowledge,
                false,
            ))),
        ]),
        Row::new(vec![
            Cell::from("👕 Clothes").style(Style::default().fg(Color::LightMagenta)),
            Cell::from(format!(
                "{:.0}/{:.0}",
                resources.clothing, resources.max_clothing
            ))
            .style(Style::default().fg(get_resource_color(
                resources.clothing,
                resources.max_clothing,
                false,
            ))),
        ]),
        Row::new(vec![
            Cell::from("🍺 Alcohol").style(Style::default().fg(Color::Yellow)),
            Cell::from(format!(
                "{:.0}/{:.0}",
                resources.alcohol, resources.max_alcohol
            ))
            .style(Style::default().fg(get_resource_color(
                resources.alcohol,
                resources.max_alcohol,
                false,
            ))),
        ]),
        Row::new(vec![
            Cell::from("📜 Permits").style(Style::default().fg(Color::White)),
            Cell::from(format!("{:.0}", resources.building_permits)),
        ]),
    ];

    let economy_table = Table::new(
        economy_rows,
        [Constraint::Percentage(50), Constraint::Percentage(50)],
    )
    .block(
        Block::default()
            .title(" Economy ")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Magenta)),
    );
    frame.render_widget(economy_table, chunks[3]);
}

#[allow(clippy::too_many_lines)]
fn render_tile_inspector(frame: &mut Frame, area: Rect, world: &World, x: i32, y: i32) {
    let terrain = world.resource::<TerrainGrid>();

    if x < 0 || y < 0 {
        let text = Paragraph::new("Outside Map").style(Style::default().fg(Color::Red));
        frame.render_widget(text, area);
        return;
    }

    let (name, char, color, terrain_type) = terrain
        .get(x as usize, y as usize)
        .map_or(("Unknown", "?", Color::Red, None), |t| {
            (t.name(), get_terrain_char(t), get_terrain_color(t), Some(t))
        });

    // Purity Logic for Rocks
    let purity_line = if terrain_type == Some(crate::layer1::terrain::TerrainType::Rock) {
        if let Some(map) = world.get_resource::<PurityMap>() {
            let purity = map.get(x, y);
            let pct = (purity * 100.0) as u32;
            let color = if purity > 0.8 {
                Color::Green
            } else if purity > 0.4 {
                Color::Yellow
            } else {
                Color::Red
            };
            Some(Line::from(vec![
                Span::raw("Purity: "),
                Span::styled(format!("{pct}%"), Style::default().fg(color)),
            ]))
        } else {
            None
        }
    } else {
        None
    };

    let echo_line: Option<Line> = None;

    let mut constraints = vec![
        Constraint::Length(1), // Header
        Constraint::Length(1), // Coords
    ];

    if purity_line.is_some() {
        constraints.push(Constraint::Length(1));
    }

    if echo_line.is_some() {
        constraints.push(Constraint::Length(1));
    }

    constraints.push(Constraint::Min(1)); // Visual

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
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

    let mut current_idx = 2;
    if let Some(line) = purity_line {
        frame.render_widget(Paragraph::new(line), layout[current_idx]);
        current_idx += 1;
    }

    if let Some(line) = echo_line {
        frame.render_widget(Paragraph::new(line), layout[current_idx]);
        current_idx += 1;
    }

    let visual_idx = current_idx;

    // Big visual representation
    let visual_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(" Visual ");
    let visual_inner = visual_block.inner(layout[visual_idx]);
    frame.render_widget(visual_block, layout[visual_idx]);

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
        let material = world
            .get::<Material>(entity)
            .map_or_else(MaterialType::default, |m| m.0);
        (
            b.building_type.label().to_string(),
            get_building_color(b.building_type, material),
        )
    } else {
        ("Entity".to_string(), Color::White)
    };

    // Determine Action if Pop
    let action_line = if let Some(action) = world.get::<PopAction>(entity) {
        let (icon, label, color) = format_action_type(action.current);
        Some(Line::from(vec![
            Span::raw("Action: "),
            Span::styled(format!("{icon} {label}"), Style::default().fg(color)),
        ]))
    } else {
        None
    };

    // Determine Generation
    let generation_line = if let Some(generation) = world.get::<Generation>(entity) {
        let label = match generation {
            Generation::Founder => "Founder",
            Generation::Immigrant => "Immigrant",
        };
        let year_str = if let Some(arrival) = world.get::<Arrival>(entity) {
            let year = 1 + arrival.tick / crate::layer1::balance::TICKS_PER_YEAR;
            format!(" (Year {year})")
        } else {
            String::new()
        };

        let color = match generation {
            Generation::Founder => Color::LightYellow,
            Generation::Immigrant => Color::Gray,
        };

        Some(Line::from(vec![
            Span::raw("Status: "),
            Span::styled(format!("{label}{year_str}"), Style::default().fg(color)),
        ]))
    } else {
        None
    };

    // Dynamic height for details section
    let details_height = 6;

    let has_structure = world.get::<Structure>(entity).is_some();
    let has_personality = world.get::<UtilityWeights>(entity).is_some();
    let personality_height = if has_personality { 2 } else { 0 };
    let has_dream = world.get::<DreamJournal>(entity).is_some();
    let dream_height = u16::from(has_dream);
    let has_diet = world.get::<DietaryHistory>(entity).is_some();
    let diet_height = u16::from(has_diet);
    let has_spirit = world.get::<MachineSpirit>(entity).is_some();
    let has_quirk = world.get::<Quirk>(entity).is_some();
    let show_diagnostics = has_spirit || has_quirk;
    let diag_content_height = u16::from(has_spirit) + u16::from(has_quirk);
    let diag_height = if show_diagnostics {
        2 + diag_content_height
    } else {
        0
    };

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),                                    // Name
            Constraint::Length(1),                                    // Pos
            Constraint::Length(u16::from(generation_line.is_some())), // Generation
            Constraint::Length(u16::from(action_line.is_some())),     // Action
            Constraint::Length(1),                                    // Spacer
            Constraint::Length(details_height),                       // Needs or Details
            Constraint::Length(u16::from(has_structure)),             // Structure HP
            Constraint::Length(diag_height), // Diagnostics (Spirit + Quirk)
            Constraint::Length(personality_height), // Personality + Spacer
            Constraint::Length(dream_height), // Last Dream
            Constraint::Length(diet_height), // Dietary History
            Constraint::Min(1),              // Biography
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

    // 3. Generation
    if let Some(line) = generation_line {
        frame.render_widget(Paragraph::new(line), layout[2]);
    }

    // 4. Action
    if let Some(line) = action_line {
        frame.render_widget(Paragraph::new(line), layout[3]);
    }

    // 5. Needs or Building Details
    let details_area = layout[5];
    if let Some(needs) = world.get::<Needs>(entity) {
        // Bio-Monitor Block
        let bio_block = Block::default()
            .title(" Bio-Monitor ")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Green));

        let bio_inner = bio_block.inner(details_area);
        frame.render_widget(bio_block, details_area);

        // Split inner area
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Hunger/Rest
                Constraint::Length(1), // Morale
                Constraint::Length(1), // Bio-Comp
            ])
            .split(bio_inner);

        // Row 1: Hunger & Rest
        let needs_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50),
                Constraint::Length(1), // Gap
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

        // Compact Gauges
        let hunger_gauge = Gauge::default()
            .gauge_style(Style::default().fg(hunger_color))
            .label(format!("🍖 {hunger_percent}%"))
            .percent(hunger_percent);

        let rest_gauge = Gauge::default()
            .gauge_style(Style::default().fg(rest_color))
            .label(format!("💤 {rest_percent}%"))
            .percent(rest_percent);

        frame.render_widget(hunger_gauge, needs_layout[0]);
        frame.render_widget(rest_gauge, needs_layout[2]);

        // Row 2: Morale
        let morale = needs.morale();
        let morale_percent = (morale * 100.0) as u16;
        let morale_color = if morale < 0.3 {
            Color::Red
        } else if morale < 0.7 {
            Color::Yellow
        } else {
            Color::Green
        };

        let morale_gauge = Gauge::default()
            .gauge_style(Style::default().fg(morale_color))
            .label(format!("😃 Morale: {morale_percent}%"))
            .percent(morale_percent);

        frame.render_widget(morale_gauge, rows[1]);

        // Row 3: Bio-Comp
        if let Some(bio) = world.get::<Biocompatibility>(entity) {
            let bio_percent = (bio.value * 100.0) as u16;
            let bio_color = if bio.value < 0.4 {
                Color::Red
            } else if bio.value < 0.7 {
                Color::Yellow
            } else {
                Color::Green
            };

            let bio_gauge = Gauge::default()
                .gauge_style(Style::default().fg(bio_color))
                .label(format!("🧬 Bio-Comp: {bio_percent}%"))
                .percent(bio_percent);

            frame.render_widget(bio_gauge, rows[2]);
        }
    } else if let Some(housing) = world.get::<Housing>(entity) {
        render_housing_details(frame, details_area, housing);
    } else if let Some(farm) = world.get::<Farm>(entity) {
        render_farm_details(frame, details_area, farm);
    } else if let Some(stockpile) = world.get::<Stockpile>(entity) {
        render_stockpile_details(frame, details_area, stockpile);
    } else if let Some(progress) = world.get::<RefiningProgress>(entity) {
        render_refining_details(frame, details_area, progress);
    }

    // 6. Structure HP
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

    // 7. Diagnostics (Spirit + Quirk)
    if show_diagnostics {
        let diag_area = layout[7];
        let block = Block::default()
            .title(" Diagnostics ")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Yellow));

        let inner = block.inner(diag_area);
        frame.render_widget(block, diag_area);

        let constraints = if has_spirit && has_quirk {
            vec![Constraint::Length(1), Constraint::Length(1)]
        } else {
            vec![Constraint::Length(1)]
        };

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(inner);

        let mut current_chunk = 0;
        if let Some(spirit) = world.get::<MachineSpirit>(entity) {
            render_machine_spirit(frame, chunks[current_chunk], spirit);
            current_chunk += 1;
        }
        if let Some(quirk) = world.get::<Quirk>(entity) {
            render_quirk(frame, chunks[current_chunk], quirk);
        }
    }

    // 8. Personality
    if let Some(weights) = world.get::<UtilityWeights>(entity) {
        render_personality(frame, layout[8], *weights);
    }

    // 9. Last Dream
    if let Some(journal) = world.get::<DreamJournal>(entity) {
        render_dream_journal(frame, layout[9], journal);
    }

    // 10. Dietary History
    if let Some(history) = world.get::<DietaryHistory>(entity) {
        render_dietary_history(frame, layout[10], history);
    }

    // 11. Biography
    let bottom_area = layout[11];
    let bio_opt = world.get::<Biography>(entity);

    if let Some(bio) = bio_opt {
        render_biography(frame, bottom_area, bio, world);
    }
}

fn render_machine_spirit(frame: &mut Frame, area: Rect, spirit: &MachineSpirit) {
    let pct = spirit.anger.clamp(0.0, 100.0) as u16;
    let color = if spirit.anger > 70.0 {
        Color::Red
    } else if spirit.anger > 30.0 {
        Color::Yellow
    } else {
        Color::Green
    };

    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::NONE))
        .gauge_style(Style::default().fg(color))
        .label(format!("Spirit Anger: {:.0}%", spirit.anger))
        .percent(pct);

    frame.render_widget(gauge, area);
}

fn render_quirk(frame: &mut Frame, area: Rect, quirk: &Quirk) {
    let text = match quirk.quirk_type {
        QuirkType::Glitchy => "⚠ Glitchy (Stops Production)",
        QuirkType::Overheating => "⚠ Overheating (Fire Risk)",
        QuirkType::Demanding => "⚠ Demanding (Anger++)",
    };

    let p =
        Paragraph::new(text).style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD));
    frame.render_widget(p, area);
}

fn render_dream_journal(frame: &mut Frame, area: Rect, journal: &DreamJournal) {
    if let Some(last_dream) = &journal.last_dream {
        let color = if last_dream.is_nightmare {
            Color::Red
        } else {
            Color::LightBlue
        };
        let p = Paragraph::new(Line::from(vec![
            Span::raw("Dream: "),
            Span::styled(&last_dream.content, Style::default().fg(color)),
        ]));
        frame.render_widget(p, area);
    }
}

fn render_dietary_history(frame: &mut Frame, area: Rect, history: &DietaryHistory) {
    if history.recent_meals.is_empty() {
        return;
    }
    let meals: Vec<String> = history
        .recent_meals
        .iter()
        .map(|i| format!("{i:?}"))
        .collect();
    let text = format!("Recent Meals: {}", meals.join(", "));
    frame.render_widget(
        Paragraph::new(text).style(Style::default().fg(Color::Gray)),
        area,
    );
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

fn render_biography(frame: &mut Frame, area: Rect, bio: &Biography, world: &World) {
    let bio_block = Block::default()
        .borders(Borders::TOP)
        .title(" Biography ")
        .title_style(Style::default().fg(Color::Blue));

    let cycle = world.resource::<DayNightCycle>();
    let ticks_per_day = cycle.ticks_per_day.max(1); // Avoid div by zero

    // Show last 5 events reversed
    let events: Vec<ListItem> = bio
        .events
        .iter()
        .rev()
        .take(5)
        .map(|e| {
            let day = e.tick / ticks_per_day;
            let day_tick = e.tick % ticks_per_day;
            #[allow(clippy::cast_precision_loss)]
            let pct = day_tick as f32 / ticks_per_day as f32;

            // Approximate phase for past events (since we don't store phase history)
            // Using same thresholds as day_night.rs
            let phase = if pct < 0.1 {
                "Dawn"
            } else if pct < 0.75 {
                "Day"
            } else if pct < 0.85 {
                "Dusk"
            } else {
                "Night"
            };

            let phase_color = match phase {
                "Dawn" => Color::LightYellow,
                "Day" => Color::Yellow,
                "Dusk" => Color::Rgb(255, 165, 0), // Orange-ish
                "Night" => Color::Blue,
                _ => Color::White,
            };

            ListItem::new(Line::from(vec![
                Span::styled(format!("Day {day} "), Style::default().fg(Color::White)),
                Span::styled(format!("{phase:5} "), Style::default().fg(phase_color)),
                Span::styled("│ ", Style::default().fg(Color::DarkGray)),
                Span::raw(&e.text),
            ]))
        })
        .collect();

    let list = List::new(events).block(bio_block);

    frame.render_widget(list, area);
}

fn render_personality(frame: &mut Frame, area: Rect, weights: UtilityWeights) {
    if area.height < 2 {
        return;
    }

    let mut traits = Vec::new();

    // Distance
    if weights.distance_weight > 1.2 {
        traits.push(Span::styled(
            "Homebody",
            Style::default().fg(Color::LightBlue),
        ));
    } else if weights.distance_weight < 0.8 {
        traits.push(Span::styled(
            "Nomad",
            Style::default().fg(Color::LightGreen),
        ));
    }

    // Availability
    if weights.availability_weight > 1.2 {
        traits.push(Span::styled(
            "Introvert",
            Style::default().fg(Color::LightMagenta),
        ));
    } else if weights.availability_weight < 0.8 {
        traits.push(Span::styled(
            "Socialite",
            Style::default().fg(Color::Yellow),
        ));
    }

    // Default if boring
    if traits.is_empty() {
        traits.push(Span::styled(
            "Average Joe",
            Style::default().fg(Color::Gray),
        ));
    }

    // Intersperse with commas
    let mut spans = Vec::new();
    spans.push(Span::raw("Traits: "));
    for (i, t) in traits.into_iter().enumerate() {
        if i > 0 {
            spans.push(Span::raw(", "));
        }
        spans.push(t);
    }

    let p = Paragraph::new(Line::from(spans)).block(Block::default().borders(Borders::NONE)); // No block to save space

    frame.render_widget(p, area);
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
            .map(|c: &ratatui::buffer::Cell| c.symbol().to_string())
            .collect();
        let full_text = cells.join("");

        // Check pop name appears instead of generic "Colonist"
        assert!(full_text.contains("Ada"));
        assert!(full_text.contains("Bio-Monitor"));
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
                    hygiene: 0.8,
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

    #[test]
    fn test_inspector_render_dream() {
        use crate::layer1::dreams::{Dream, DreamJournal};
        use crate::layer1::pop::PopName;

        let mut world = World::new();
        world.insert_resource(Selection::default());
        let entity = world
            .spawn((
                Pop,
                PopName("Dreamer".to_string()),
                GridPosition { x: 1, y: 1 },
                DreamJournal {
                    last_dream: Some(Dream {
                        content: "dreamed of flying pigs".to_string(),
                        tick: 100,
                        impact: 0.1,
                        is_nightmare: false,
                    }),
                    history: vec![],
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

        assert!(full_text.contains("Dreamer"));
        assert!(full_text.contains("Dream:"));
        assert!(full_text.contains("flying pigs"));
    }

    #[test]
    fn test_inspector_render_rock_purity() {
        use crate::layer1::purity::PurityMap;
        use crate::layer1::terrain::{TerrainGrid, TerrainType};

        let mut world = World::new();
        // Setup Rock tile
        let mut tiles = vec![TerrainType::Grass; 100];
        tiles[55] = TerrainType::Rock; // (5, 5)
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });

        // Setup PurityMap
        let mut purity_map = PurityMap::default();
        purity_map.set_override(5, 5, 0.85);
        world.insert_resource(purity_map);

        // Select Tile
        world.insert_resource(Selection::default());
        world.resource_mut::<Selection>().select_tile(5, 5);

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

        assert!(full_text.contains("Rock"));
        assert!(full_text.contains("Purity: 85%"));
    }

    #[test]
    fn test_inspector_render_pop_biocompatibility() {
        use crate::layer1::biocompatibility::Biocompatibility;
        use crate::layer1::pop::PopName;

        let mut world = World::new();
        world.insert_resource(Selection::default());
        let entity = world
            .spawn((
                Pop,
                PopName("Cade".to_string()),
                GridPosition { x: 2, y: 3 },
                Needs::default(),
                Biocompatibility { value: 0.85 },
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

        assert!(full_text.contains("Cade"));
        assert!(full_text.contains("Bio-Comp: 85%"));
    }

    #[test]
    fn test_inspector_render_machine_spirit() {
        use crate::layer1::building::BuildingType;
        use crate::layer1::rituals::{MachineSpirit, Quirk, QuirkType};

        let mut world = World::new();
        world.insert_resource(Selection::default());
        let entity = world
            .spawn((
                Building {
                    building_type: BuildingType::AncientReactor,
                },
                MachineSpirit { anger: 80.0 },
                Quirk {
                    quirk_type: QuirkType::Glitchy,
                },
                GridPosition { x: 0, y: 0 },
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

        assert!(full_text.contains("Spirit Anger: 80%"));
        assert!(full_text.contains("Glitchy"));
    }

    #[test]
    fn test_inspector_render_dietary_history() {
        use crate::layer1::items::ItemType;
        use crate::layer1::palette_fatigue::DietaryHistory;
        use crate::layer1::pop::PopName;

        let mut world = World::new();
        world.insert_resource(Selection::default());

        let mut history = DietaryHistory::default();
        history.recent_meals.push_back(ItemType::Potato);
        history.recent_meals.push_back(ItemType::Meat);

        let entity = world
            .spawn((
                Pop,
                PopName("Gourmand".to_string()),
                GridPosition { x: 1, y: 1 },
                history,
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

        assert!(full_text.contains("Gourmand"));
        assert!(full_text.contains("Recent Meals:"));
        assert!(full_text.contains("Potato"));
        assert!(full_text.contains("Meat"));
    }
}

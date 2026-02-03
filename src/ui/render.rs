//! Main rendering logic.
//!
//! This module coordinates the layout and rendering of the entire application.

use super::map::{MapRenderContext, render_map_layer};
use crate::layer1::{
    BuildMode, Building, BuildingType, Chronicle, ChronicleUiState, ColonyResources, Designation,
    DesignationMode, DesignationType, Farm, GridPosition, Housing, Needs, Pop, TerrainGrid,
    Viewport, can_designate, can_place_building, format_event_prefix,
};
use crate::shared::log::MessageLog;
use crate::shared::selection::{Selection, SelectionTarget, inspect_entity, inspect_tile};
use crate::shared::state::GameState;
use crate::shared::time::{SimSpeed, SimulationTime};
use crate::ui::colors::get_log_color;
use bevy_ecs::prelude::*;
use ratatui::{
    prelude::*,
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph},
};
use std::collections::HashMap;

/// Renders the entire application.
pub fn render_app(terminal: &mut Terminal<impl Backend>, world: &World) -> anyhow::Result<()> {
    terminal.draw(|frame| render(world, frame))?;
    Ok(())
}

fn render(world: &World, frame: &mut Frame) {
    // Main vertical split: content + status bar
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(10),   // Content area
            Constraint::Length(1), // Status bar
        ])
        .split(frame.area());

    // Horizontal split: map + info panel
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(20),    // Map area
            Constraint::Length(20), // Info panel
        ])
        .split(main_chunks[0]);

    let map_area = content_chunks[0];
    let info_area = content_chunks[1];
    let status_area = main_chunks[1];

    // Render map
    render_map(frame, map_area, world);

    // Render info panel
    render_info_panel(frame, info_area, world);

    // Render status bar
    render_status_bar(frame, status_area, world);

    // Render chronicle
    render_chronicle(frame, frame.area(), world);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn render_chronicle(frame: &mut Frame, area: Rect, world: &World) {
    let ui_state = world.resource::<ChronicleUiState>();
    if !ui_state.is_open {
        return;
    }

    let chronicle = world.resource::<Chronicle>();

    let block = Block::default()
        .title(" Chronicle (Press L/H to close) ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .style(Style::default().bg(Color::Black));

    let popup_area = centered_rect(60, 60, area);
    frame.render_widget(Clear, popup_area); // Clear background
    frame.render_widget(block.clone(), popup_area);

    let inner = block.inner(popup_area);

    let items: Vec<ListItem> = chronicle
        .events
        .iter()
        .rev() // Newest first
        .map(|evt| {
            let prefix = format_event_prefix(evt.importance);
            ListItem::new(format!(
                "[{}] Y{}: {} {}",
                evt.tick, evt.year, prefix, evt.text
            ))
        })
        .collect();

    let list = List::new(items);
    frame.render_widget(list, inner);
}

fn render_map(frame: &mut Frame, area: Rect, world: &World) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(" Colony ");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Render terrain inside
    let terrain = world.resource::<TerrainGrid>();
    let viewport = world.resource::<Viewport>();
    let build_mode = world.resource::<BuildMode>();
    let designation_mode = world.resource::<DesignationMode>();

    let pops_data = get_pops_render_data(world);
    let buildings_data = get_buildings_render_data(world);
    let designations_data = get_designations_render_data(world);

    // Build mode cursor info
    let build_mode_cursor = if build_mode.active {
        let can_place = can_place_building(world, build_mode.cursor.x, build_mode.cursor.y);
        Some((build_mode.cursor, build_mode.selected, can_place))
    } else {
        None
    };

    // Designation mode cursor info
    let designation_mode_cursor = if designation_mode.active {
        let can = can_designate(
            world,
            designation_mode.cursor.x,
            designation_mode.cursor.y,
            designation_mode.tool,
        );
        Some((designation_mode.cursor, designation_mode.tool, can))
    } else {
        None
    };

    let ctx = MapRenderContext {
        area: inner,
        terrain,
        viewport,
        pops_data: &pops_data,
        buildings_data: &buildings_data,
        designations_data: &designations_data,
        build_mode: build_mode_cursor,
        designation_mode: designation_mode_cursor,
    };

    render_map_layer(frame, ctx);
}

fn get_pops_render_data(world: &World) -> HashMap<GridPosition, Needs> {
    world
        .iter_entities()
        .filter(|e| e.contains::<GridPosition>() && e.contains::<Needs>())
        .map(|e| {
            let pos = *e.get::<GridPosition>().unwrap();
            let needs = *e.get::<Needs>().unwrap();
            (pos, needs)
        })
        .collect()
}

fn get_buildings_render_data(world: &World) -> HashMap<GridPosition, BuildingType> {
    world
        .iter_entities()
        .filter(|e| e.contains::<GridPosition>() && e.contains::<Building>())
        .map(|e| {
            let pos = *e.get::<GridPosition>().unwrap();
            let building = e.get::<Building>().unwrap();
            (pos, building.building_type)
        })
        .collect()
}

fn get_designations_render_data(world: &World) -> HashMap<GridPosition, DesignationType> {
    world
        .iter_entities()
        .filter(|e| e.contains::<GridPosition>() && e.contains::<Designation>())
        .map(|e| {
            let pos = *e.get::<GridPosition>().unwrap();
            let designation = e.get::<Designation>().unwrap();
            (pos, designation.designation_type)
        })
        .collect()
}

fn render_info_panel(frame: &mut Frame, area: Rect, world: &World) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(10),    // Stats
            Constraint::Length(10), // Log
        ])
        .split(area);

    let stats_area = chunks[0];
    let log_area = chunks[1];

    // Stats Panel
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(" Info ");
    let inner = block.inner(stats_area);
    frame.render_widget(block, stats_area);

    let selection = world.resource::<Selection>();
    let text = match selection.target() {
        SelectionTarget::None => {
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

            let resources = world.resource::<ColonyResources>();

            format!(
                "Population: {pop_count}\n\n\
                 Food: {:.1}\n\n\
                 Housing: {housing_count}\n\
                 Beds: {housing_used}/{housing_capacity}\n\n\
                 Farms: {farm_count}\n\
                 Workers: {farm_used}/{farm_capacity}\n",
                resources.food
            )
        }
        SelectionTarget::Tile(x, y) => inspect_tile(world, x, y),
        SelectionTarget::Entity(e) => inspect_entity(world, e),
    };

    let paragraph = Paragraph::new(text);
    frame.render_widget(paragraph, inner);

    // Message Log Panel
    let log_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(" Log ");
    let log_inner = log_block.inner(log_area);
    frame.render_widget(log_block, log_area);

    if let Some(log) = world.get_resource::<MessageLog>() {
        let height = log_inner.height as usize;
        let start = log.messages.len().saturating_sub(height);
        let items: Vec<ListItem> = log
            .messages
            .iter()
            .skip(start)
            .map(|m| ListItem::new(Line::styled(
                m.text.clone(),
                Style::default().fg(get_log_color(m.color)),
            )))
            .collect();

        let list = List::new(items);
        frame.render_widget(list, log_inner);
    }
}

fn render_status_bar(frame: &mut Frame, area: Rect, world: &World) {
    let sim_time = world.resource::<SimulationTime>();
    let game_state = world.resource::<GameState>();
    let build_mode = world.resource::<BuildMode>();
    let designation_mode = world.resource::<DesignationMode>();

    // NOTE: Dual pause state check. GameState::Paused is controlled by spacebar,
    // SimSpeed::Paused exists but is currently not used (no key binds to it).
    // This allows for future distinction between "paused but still simulating at 0x"
    // vs "completely frozen". Current behavior: only GameState::Paused matters (main.rs:86).
    let paused = *game_state == GameState::Paused || sim_time.speed == SimSpeed::Paused;

    let status = get_status_string(
        sim_time.tick,
        sim_time.speed,
        paused,
        build_mode,
        designation_mode,
    );

    let bar = Paragraph::new(status).style(Style::default().bg(Color::DarkGray).fg(Color::White));
    frame.render_widget(bar, area);
}

fn get_status_string(
    tick: u64,
    speed: SimSpeed,
    paused: bool,
    build_mode: &BuildMode,
    designation_mode: &DesignationMode,
) -> String {
    let mode_str = if build_mode.active {
        format!(
            "BUILD: {} (Tab:switch Enter:place Esc:exit)",
            build_mode.selected.label()
        )
    } else if designation_mode.active {
        format!(
            "DESIGNATE: {} (Enter:apply Esc:exit)",
            designation_mode.tool.label()
        )
    } else {
        "B:Build  M:Mine  X:Demolish  L:Chronicle  1-3:Speed  q:Quit".to_string()
    };

    format!(
        " {} │ Tick: {} │ {} │ {} ",
        if paused { "⏸" } else { "▶" },
        tick,
        speed.label(),
        mode_str
    )
}

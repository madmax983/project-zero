use bevy_ecs::prelude::*;
use ratatui::{
    prelude::*,
    widgets::{Block, BorderType, Borders, List, ListItem, Paragraph},
};

use crate::layer1::{ColonyResources, Farm, Housing, Pop};
use crate::shared::log::MessageLog;
use crate::shared::selection::{Selection, SelectionTarget, inspect_entity, inspect_tile};

pub fn render_info_panel(frame: &mut Frame, area: Rect, world: &World) {
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
                 Food: {:.1}/{:.0}\n\
                 Wood: {:.1}/{:.0}\n\
                 Stone: {:.1}/{:.0}\n\n\
                 Housing: {housing_count}\n\
                 Beds: {housing_used}/{housing_capacity}\n\n\
                 Farms: {farm_count}\n\
                 Workers: {farm_used}/{farm_capacity}\n",
                resources.food,
                resources.max_food,
                resources.wood,
                resources.max_wood,
                resources.stone,
                resources.max_stone
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
            .map(|m| ListItem::new(Line::styled(m.text.clone(), Style::default().fg(m.color))))
            .collect();

        let list = List::new(items);
        frame.render_widget(list, log_inner);
    }
}

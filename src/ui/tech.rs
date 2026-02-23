use bevy_ecs::prelude::*;
use ratatui::prelude::*;
use crate::layer1::tech::Tech;
use strum::IntoEnumIterator;
use crate::layer1::building::BuildingType;

/// State for the Tech Tree UI.
#[derive(Resource, Default, Debug)]
pub struct TechUiState {
    /// Whether the Tech Tree UI is currently open.
    pub is_open: bool,
    /// The index of the currently selected technology in the list.
    pub selected_index: usize,
}

impl TechUiState {
    /// Select the next technology in the list.
    pub const fn next(&mut self, count: usize) {
        if count == 0 {
            return;
        }
        self.selected_index = (self.selected_index + 1) % count;
    }

    /// Select the previous technology in the list.
    pub const fn prev(&mut self, count: usize) {
        if count == 0 {
            return;
        }
        self.selected_index = if self.selected_index == 0 {
            count - 1
        } else {
            self.selected_index - 1
        };
    }
}

/// Returns the list of all available technologies to display in the UI.
#[must_use]
pub fn get_tech_list() -> Vec<Tech> {
    vec![
        Tech::Masonry,
        Tech::MetalWorking,
        Tech::SocialStructures,
        Tech::Astronomy,
        Tech::Hydroponics,
        Tech::Militia,
        Tech::Medical,
        Tech::Electromagnetism,
        Tech::VoidWhispers,
    ]
}

use ratatui::widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap};

/// Renders the Tech Tree UI.
pub fn render_tech_tree(frame: &mut Frame, area: Rect, world: &World) {
    let ui_state = world.resource::<TechUiState>();
    if !ui_state.is_open {
        return;
    }

    let tech_state = world.get_resource::<crate::layer1::tech::TechState>();
    let resources = world.get_resource::<crate::layer1::resources::ColonyResources>();
    let knowledge = resources.map_or(0.0, |r| r.knowledge);

    // Centered Popup
    let popup_area = centered_rect(80, 80, area);
    frame.render_widget(Clear, popup_area);

    let main_block = Block::default()
        .title(" Technology Tree (Press T/Esc to close) ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .style(Style::default().bg(Color::Black));

    frame.render_widget(main_block.clone(), popup_area);

    // Inner area for content
    let inner_area = main_block.inner(popup_area);

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30), // List
            Constraint::Percentage(70), // Details
        ])
        .split(inner_area);

    let techs = get_tech_list();

    // --- Left Pane: Tech List ---
    let items: Vec<ListItem> = techs
        .iter()
        .map(|tech| {
            let is_unlocked = tech_state.is_some_and(|ts| ts.is_unlocked(*tech));
            let is_corrupted = tech_state.is_some_and(|ts| ts.techs.get(tech) == Some(&crate::layer1::tech::TechStatus::Corrupted));
            let cost = tech.cost();
            let affordable = knowledge >= cost;

            let (symbol, color) = if is_unlocked {
                ("[X]", Color::Green)
            } else if is_corrupted {
                ("[!]", Color::Red)
            } else if affordable {
                ("[ ]", Color::Yellow)
            } else {
                ("[ ]", Color::DarkGray)
            };

            let label = format!("{symbol} {}", tech.label());
            ListItem::new(label).style(Style::default().fg(color))
        })
        .collect();

    let list_block = Block::default()
        .borders(Borders::RIGHT)
        .border_style(Style::default().fg(Color::DarkGray));

    let list = List::new(items)
        .block(list_block)
        .highlight_style(Style::default().add_modifier(Modifier::BOLD).bg(Color::DarkGray));

    let mut state = ListState::default();
    state.select(Some(ui_state.selected_index));

    frame.render_stateful_widget(list, chunks[0], &mut state);

    // --- Right Pane: Details ---
    if let Some(tech) = techs.get(ui_state.selected_index) {
        render_tech_details(frame, chunks[1], *tech, tech_state, knowledge);
    }
}

fn render_tech_details(
    frame: &mut Frame,
    area: Rect,
    tech: Tech,
    tech_state: Option<&crate::layer1::tech::TechState>,
    current_knowledge: f32
) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Header
            Constraint::Length(4), // Description
            Constraint::Length(4), // Costs & Status
            Constraint::Min(5),    // Unlocks
            Constraint::Length(1), // Footer/Hazard
        ])
        .split(area);

    // 1. Header
    let title = Paragraph::new(tech.label())
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD | Modifier::UNDERLINED))
        .alignment(Alignment::Center);
    frame.render_widget(title, layout[0]);

    // 2. Description
    let desc = Paragraph::new(tech.description())
        .wrap(Wrap { trim: true })
        .style(Style::default().fg(Color::White));
    frame.render_widget(desc, layout[1]);

    // 3. Costs & Status
    let is_unlocked = tech_state.is_some_and(|ts| ts.is_unlocked(tech));
    let is_corrupted = tech_state.is_some_and(|ts| ts.techs.get(&tech) == Some(&crate::layer1::tech::TechStatus::Corrupted));
    let cost = tech.cost();

    let status_text = if is_unlocked {
        Span::styled("RESEARCHED", Style::default().fg(Color::Green))
    } else if is_corrupted {
        Span::styled("CORRUPTED (Needs Data Capacity)", Style::default().fg(Color::Red))
    } else if current_knowledge >= cost {
        Span::styled("AVAILABLE (Press Enter)", Style::default().fg(Color::Yellow))
    } else {
        Span::styled(format!("LOCKED (Need {:.0} Knowledge)", cost), Style::default().fg(Color::DarkGray))
    };

    let cost_text = format!("Cost: {:.0} Knowledge | Storage: {:.0} TB", cost, tech.storage_cost());

    let info_block = Block::default()
        .borders(Borders::TOP | Borders::BOTTOM)
        .border_style(Style::default().fg(Color::DarkGray));

    let info = Paragraph::new(vec![
        Line::from(status_text),
        Line::from(cost_text),
    ]).block(info_block);

    frame.render_widget(info, layout[2]);

    // 4. Unlocks
    let mut unlocks = Vec::new();
    for building in BuildingType::iter() {
        if building.required_tech() == Some(tech) {
            unlocks.push(building);
        }
    }

    let unlocks_text: Vec<ListItem> = if unlocks.is_empty() {
         vec![ListItem::new(Span::raw("Unlocks: Nothing directly (Passive Effect?)").style(Style::default().fg(Color::Gray)))]
    } else {
        let mut list = vec![ListItem::new(Span::styled("Unlocks Building Plans:", Style::default().add_modifier(Modifier::BOLD)))];
        for b in unlocks {
            list.push(ListItem::new(format!(" - {} ({})", b.label(), b.char())));
        }
        list
    };

    let unlocks_list = List::new(unlocks_text);
    frame.render_widget(unlocks_list, layout[3]);

    // 5. Hazard Warning
    if tech.is_hazardous() {
        let warning = Paragraph::new("⚠ HAZARDOUS TECHNOLOGY ⚠")
            .style(Style::default().fg(Color::Red).bg(Color::Black).add_modifier(Modifier::BOLD | Modifier::SLOW_BLINK))
            .alignment(Alignment::Center);
        frame.render_widget(warning, layout[4]);
    }
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

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::tech::{Tech, TechState};
    use crate::layer1::resources::ColonyResources;
    use crate::ui::tech::{TechUiState, get_tech_list};

    #[test]
    fn test_tech_ui_state_resource_defaults() {
        let state = TechUiState::default();
        assert!(!state.is_open);
        assert_eq!(state.selected_index, 0);
    }

    #[test]
    fn test_tech_ui_navigation() {
        let mut state = TechUiState::default();
        let tech_count = 3;

        // Down
        state.next(tech_count);
        assert_eq!(state.selected_index, 1);
        state.next(tech_count);
        assert_eq!(state.selected_index, 2);
        state.next(tech_count);
        assert_eq!(state.selected_index, 0); // Wrap

        // Up
        state.prev(tech_count);
        assert_eq!(state.selected_index, 2); // Wrap back
    }

    #[test]
    fn test_unlock_action_via_function() {
        let mut world = World::new();
        let mut tech_state = TechState::default();
        tech_state.total_capacity = 100.0;
        let mut res = ColonyResources::default();
        res.knowledge = 100.0;

        world.insert_resource(tech_state);
        world.insert_resource(res);
        world.insert_resource(crate::shared::log::MessageLog::default());

        // Setup UI State selecting first tech
        let _ui_state = TechUiState { is_open: true, selected_index: 0 };

        let techs = get_tech_list();
        if techs.is_empty() {
             assert!(!techs.is_empty(), "Tech list should not be empty");
        }
        let target_tech = techs[0];

        // Perform unlock logic (simulating input handler logic)
        crate::layer1::tech::unlock_tech(&mut world, target_tech);

        assert!(world.resource::<TechState>().is_unlocked(target_tech));
        assert!(world.resource::<ColonyResources>().knowledge < 100.0);
    }

    #[test]
    fn test_tech_list_completeness() {
        let list = get_tech_list();
        assert!(list.contains(&Tech::Masonry));
        assert!(list.contains(&Tech::MetalWorking));
        assert!(list.contains(&Tech::SocialStructures));
    }
}

use crate::layer1::tech::Tech;
use bevy_ecs::prelude::*;
use ratatui::prelude::*;

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

use ratatui::widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState};

/// Renders the Tech Tree UI.
pub fn render_tech_tree(frame: &mut Frame, area: Rect, world: &World) {
    let ui_state = world.resource::<TechUiState>();
    if !ui_state.is_open {
        return;
    }

    let tech_state = world.get_resource::<crate::layer1::tech::TechState>();
    let resources = world.get_resource::<crate::layer1::resources::ColonyResources>();
    let knowledge = resources.map_or(0.0, |r| r.knowledge);

    let popup_area = centered_rect(60, 60, area);
    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .title(" Technology (Press T/Esc to close) ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .style(Style::default().bg(Color::Black));

    let techs = get_tech_list();
    let items: Vec<ListItem> = techs
        .iter()
        .map(|tech| {
            let is_unlocked = tech_state.is_some_and(|ts| ts.is_unlocked(*tech));
            let is_corrupted = tech_state.is_some_and(|ts| {
                ts.techs.get(tech) == Some(&crate::layer1::tech::TechStatus::Corrupted)
            });
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

            let label = format!("{} {} (Cost: {:.0})", symbol, tech.label(), cost);
            let desc = if tech.is_hazardous() { " [HAZARD]" } else { "" };

            ListItem::new(format!("{label}{desc}")).style(Style::default().fg(color))
        })
        .collect();

    let list = List::new(items).block(block).highlight_style(
        Style::default()
            .add_modifier(Modifier::BOLD)
            .bg(Color::DarkGray),
    );

    let mut state = ListState::default();
    state.select(Some(ui_state.selected_index));

    frame.render_stateful_widget(list, popup_area, &mut state);
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
    use crate::layer1::resources::ColonyResources;
    use crate::layer1::tech::{Tech, TechState};
    use crate::ui::tech::{TechUiState, get_tech_list};
    use bevy_ecs::prelude::*;

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
        let _ui_state = TechUiState {
            is_open: true,
            selected_index: 0,
        };

        let techs = get_tech_list();
        // Assuming get_tech_list returns something for this test to be meaningful
        if techs.is_empty() {
            // If empty, we can't test unlock logic yet, but the test should fail if list is incomplete anyway
            // For now, let's force fail if empty to drive implementation
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

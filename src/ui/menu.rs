use crate::shared::menu::MenuState;
use ratatui::prelude::*;
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};

const LOGO: &str = r#"
  ____   ____    _    _     _____
 / ___| / ___|  / \  | |   | ____|
 \___ \| |     / _ \ | |   |  _|
  ___) | |___ / ___ \| |___| |___
 |____/ \____/_/   \_\_____|_____|
"#;

const SUBTITLE: &str = "From pebble to empire. Every world remembers.";

/// Renders the main menu screen.
///
/// Displays the game title and a selectable list of options (Start Game, Quit).
#[allow(clippy::cast_possible_truncation)]
pub fn render_main_menu(frame: &mut Frame, area: Rect, state: &MenuState) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(2),                                 // Top spacing
            Constraint::Length(9),                              // Title and Subtitle
            Constraint::Length(state.options.len() as u16 + 2), // Menu items
            Constraint::Min(2),                                 // Bottom spacing
            Constraint::Length(3),                              // Footer
        ])
        .split(area);

    // Title Block
    let mut title_text: Vec<Line> = LOGO
        .lines()
        .map(|line| {
            Line::from(Span::styled(
                line,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ))
        })
        .collect();

    title_text.push(Line::from(""));
    title_text.push(Line::from(Span::styled(
        SUBTITLE,
        Style::default()
            .fg(Color::Gray)
            .add_modifier(Modifier::ITALIC),
    )));

    let title = Paragraph::new(title_text).alignment(Alignment::Center);
    frame.render_widget(title, layout[1]);

    // Options Block
    let options_block = Block::default().borders(Borders::NONE);
    let menu_area = options_block.inner(layout[2]);
    frame.render_widget(options_block, layout[2]);

    let constraints: Vec<Constraint> = state
        .options
        .iter()
        .map(|_| Constraint::Length(1))
        .collect();

    let menu_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(menu_area);

    for (i, option) in state.options.iter().enumerate() {
        let style = if i == state.selected_index {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
                .add_modifier(Modifier::REVERSED)
        } else {
            Style::default().fg(Color::White)
        };

        let text = if i == state.selected_index {
            format!("  ▶ {option}  ")
        } else {
            format!("    {option}  ")
        };

        let p = Paragraph::new(text)
            .style(style)
            .alignment(Alignment::Center);
        frame.render_widget(p, menu_layout[i]);
    }

    // Footer
    let footer_text = "Use ↑/↓ to select | Enter to confirm";
    let footer = Paragraph::new(footer_text)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::TOP)
                .border_type(BorderType::Double)
                .border_style(Style::default().fg(Color::DarkGray)),
        );
    frame.render_widget(footer, layout[4]);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    use crate::shared::menu::MenuState;

    #[test]
    fn test_render_main_menu_empty_options() {
        let backend = TestBackend::new(40, 25);
        let mut terminal = Terminal::new(backend).unwrap();
        let state = MenuState {
            options: vec![],
            selected_index: 0,
        };

        terminal.draw(|f| {
            render_main_menu(f, f.area(), &state);
        }).unwrap();
    }

    #[test]
    fn test_render_main_menu_with_options() {
        let backend = TestBackend::new(40, 25);
        let mut terminal = Terminal::new(backend).unwrap();
        let state = MenuState {
            options: vec!["Start".to_string(), "Quit".to_string()],
            selected_index: 1,
        };

        terminal.draw(|f| {
            render_main_menu(f, f.area(), &state);
        }).unwrap();
    }
}

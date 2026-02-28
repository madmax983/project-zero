use bevy_ecs::archetype::Archetype;
use bevy_ecs::prelude::*;
use ratatui::{prelude::*, widgets::Paragraph};

use crate::layer1::admin::AdminStats;
use crate::layer1::seasons::{Season, SeasonState};
use crate::layer1::solar::{SolarCycle, SolarCycleState};
use crate::layer1::traits::Traits;
use crate::layer1::{
    BuildMode, ColonyPolicies, ColonyResources, DesignationMode, NamedLocations, Pop, Viewport,
};
use crate::shared::state::GameState;
use crate::shared::time::{SimSpeed, SimulationTime};

/// Renders the bottom status bar containing global simulation state.
///
/// Displays:
/// - **Time**: Current day and speed.
/// - **Population**: Total "Souls" count.
/// - **Morale**: Average colony morale (color-coded).
/// - **Resources**: Key resource totals (Food, Tools).
/// - **Location**: Name of the location under the viewport center.
/// - **Mode**: Current interaction mode (Build, Designate, or Hotkeys).
///
/// # Arguments
///
/// * `frame` - The `ratatui` frame to render into.
/// * `area` - The rectangular area allocated for the status bar.
/// * `world` - The ECS world to query resources and components from.
pub fn render_status_bar(frame: &mut Frame, area: Rect, world: &World) {
    let sim_time = world.resource::<SimulationTime>();
    let game_state = world.resource::<GameState>();
    let build_mode = world.resource::<BuildMode>();
    let designation_mode = world.resource::<DesignationMode>();
    let viewport = world.resource::<Viewport>();
    let locations = world.resource::<NamedLocations>();
    let resources = world.resource::<ColonyResources>();
    let policies = world.get_resource::<ColonyPolicies>();
    let cycle = world.get_resource::<crate::layer1::day_night::DayNightCycle>();

    // NOTE: Dual pause state check. GameState::Paused is controlled by spacebar,
    // SimSpeed::Paused exists but is currently not used (no key binds to it).
    // This allows for future distinction between "paused but still simulating at 0x"
    // vs "completely frozen". Current behavior: only GameState::Paused matters (main.rs:86).
    let paused = *game_state == GameState::Paused || sim_time.speed == SimSpeed::Paused;

    let screen_area = frame.area();
    let center_x = viewport.x + i32::from(screen_area.width / 2);
    let center_y = viewport.y + i32::from(screen_area.height / 2);
    let location_name = locations.get(center_x, center_y).map(String::as_str);

    // Count pops safely with immutable world access
    let pop_count = world.component_id::<Pop>().map_or(0, |pop_id| {
        world
            .archetypes()
            .iter()
            .filter(|archetype| archetype.contains(pop_id))
            .map(Archetype::len)
            .sum()
    });

    // Calculate average morale
    let (total_morale, morale_count) = world
        .iter_entities()
        .filter_map(|e| {
            let needs = e.get::<crate::layer1::Needs>()?;
            let memories = e.get::<crate::layer1::Memories>();
            let social_buff = e.get::<crate::layer1::social::SocialBuff>();
            let traits = e.get::<Traits>();
            let morale_comp = e.get::<crate::layer1::morale::Morale>();
            let morale = crate::layer1::memory::calculate_effective_morale(
                needs,
                memories,
                social_buff,
                policies,
                traits,
                cycle.map(|c| c.time_of_day),
                morale_comp,
            );
            Some(morale)
        })
        .fold((0.0, 0), |(sum, count), m| (sum + m, count + 1));

    #[allow(clippy::cast_precision_loss)]
    let avg_morale = if morale_count > 0 {
        total_morale / morale_count as f32
    } else {
        0.0
    };

    let season = world
        .get_resource::<SeasonState>()
        .map(|s| s.current_season);

    let solar_cycle = world
        .get_resource::<SolarCycleState>()
        .map(|s| s.current_cycle);

    let admin_stats = world.get_resource::<AdminStats>();
    let efficiency = admin_stats.map_or(1.0, |s| s.efficiency);

    let status = get_status_line(
        sim_time.tick,
        sim_time.speed,
        paused,
        build_mode,
        designation_mode,
        location_name,
        pop_count,
        resources.food,
        resources.rations,
        resources.tools,
        avg_morale,
        efficiency,
        season,
        solar_cycle,
    );

    let status = truncate_line(status, area.width);

    let bar = Paragraph::new(status).style(Style::default().bg(Color::DarkGray).fg(Color::White));
    frame.render_widget(bar, area);
}

/// Constructs the styled `Line` for the status bar.
///
/// This separates the formatting logic from the ECS querying logic, making it easier
/// to test the layout without mocking the entire `World`.
///
/// # Arguments
///
/// * `tick` - Current simulation tick (Day).
/// * `speed` - Current simulation speed ([`SimSpeed`]).
/// * `paused` - Whether the simulation is paused.
/// * `build_mode` - State of the building placement tool ([`BuildMode`]).
/// * `designation_mode` - State of the designation tool (e.g., Mining, [`DesignationMode`]).
/// * `location_name` - Optional name of the location being viewed.
/// * `pop_count` - Total number of colonists.
/// * `food_yield` - Current food resource amount.
/// * `rations` - Current rations resource amount.
/// * `tools` - Current tool resource amount.
/// * `morale` - Average morale (0.0 to 1.0).
#[must_use]
#[allow(
    clippy::too_many_arguments,
    clippy::too_many_lines,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
pub fn get_status_line<'a>(
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
) -> Line<'a> {
    let mut spans = Vec::new();

    // 1. Play/Pause
    if paused {
        spans.push(Span::styled(" ⏸ ", Style::default().fg(Color::Red)));
    } else {
        spans.push(Span::styled(" ▶ ", Style::default().fg(Color::Green)));
    }

    // 2. Day
    spans.push(Span::raw(format!("Day {tick} │ ")));

    // 2b. Season
    if let Some(s) = season {
        let color = match s {
            Season::Spring => Color::Green,
            Season::Summer => Color::Yellow,
            Season::Autumn => Color::Rgb(200, 100, 0),
            Season::Winter => Color::Cyan,
        };
        spans.push(Span::styled(
            format!("{} ", s.name()),
            Style::default().fg(color),
        ));
        spans.push(Span::raw("│ "));
    }

    // 2c. Solar Cycle
    if let Some(cycle) = solar_cycle {
        spans.push(Span::styled(
            format!("{} ", cycle.label()),
            Style::default().fg(Color::Yellow),
        ));
        spans.push(Span::raw("│ "));
    }

    // 3. Souls
    spans.push(Span::styled("Souls: ", Style::default().fg(Color::Cyan)));
    spans.push(Span::styled(
        format!("{pop_count} │ "),
        Style::default().fg(Color::White),
    ));

    // 4. Morale
    let morale_percent = (morale * 100.0).round() as u8;
    let morale_color = if morale < 0.3 {
        Color::Red
    } else if morale < 0.7 {
        Color::Yellow
    } else {
        Color::Green
    };
    spans.push(Span::styled("Morale: ", Style::default().fg(morale_color)));
    spans.push(Span::styled(
        format!("{morale_percent}% │ "),
        Style::default().fg(Color::White),
    ));

    // 4b. Admin Efficiency
    let eff_percent = (efficiency * 100.0).round() as u8;
    let eff_color = if efficiency < 0.5 {
        Color::Red
    } else if efficiency < 0.8 {
        Color::Yellow
    } else {
        Color::Green
    };
    spans.push(Span::styled("Admin: ", Style::default().fg(eff_color)));
    spans.push(Span::styled(
        format!("{eff_percent}% │ "),
        Style::default().fg(Color::White),
    ));

    // 5. Yield (Food)
    let total_food = food_yield + rations;
    let food_color = if total_food < 10.0 {
        Color::Red
    } else {
        Color::Green
    };
    spans.push(Span::styled("Food: ", Style::default().fg(food_color)));
    spans.push(Span::styled(
        format!("{food_yield:.0}+{rations:.0} │ "),
        Style::default().fg(Color::White),
    ));

    // 6. Tools
    spans.push(Span::styled("Tools: ", Style::default().fg(Color::Yellow)));
    spans.push(Span::styled(
        format!("{tools:.0} │ "),
        Style::default().fg(Color::White),
    ));

    // 7. Speed
    spans.push(Span::styled(
        format!("{} ", speed.label()),
        Style::default().fg(Color::DarkGray),
    ));

    // 8. Location
    if let Some(name) = location_name {
        spans.push(Span::raw("│ 📍 "));
        spans.push(Span::styled(
            format!("{name} "),
            Style::default().fg(Color::Magenta),
        ));
    }

    // 9. Mode
    // Add some padding before mode
    spans.push(Span::raw(" "));
    if build_mode.active {
        let cost = build_mode.selected.cost(build_mode.selected_material);
        let mut cost_parts: Vec<String> = Vec::new();
        if cost.wood > 0.0 {
            cost_parts.push(format!("{:.0}W", cost.wood));
        }
        if cost.stone > 0.0 {
            cost_parts.push(format!("{:.0}S", cost.stone));
        }
        if cost.ore > 0.0 {
            cost_parts.push(format!("{:.0}O", cost.ore));
        }
        if cost.metal > 0.0 {
            cost_parts.push(format!("{:.0}M", cost.metal));
        }
        let cost_str = if cost_parts.is_empty() {
            "Free".to_string()
        } else {
            cost_parts.join(" ")
        };

        let mut name = build_mode.selected.label().to_string();
        if build_mode.selected.supports_material() {
            name.push_str(" (");
            name.push_str(build_mode.selected_material.label());
            name.push(')');
        }

        spans.push(Span::styled(
            format!(
                "BUILD: {name} [{cost_str}] (Tab:switch M/BackTab:material Enter:place Esc:exit)",
            ),
            Style::default()
                .fg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        ));
    } else if designation_mode.active {
        let hint = if designation_mode.drag_start.is_some() {
            "Enter:confirm area Esc:exit"
        } else {
            "Enter:start area Esc:exit"
        };
        spans.push(Span::styled(
            format!("DESIGNATE: {} ({hint})", designation_mode.tool.label()),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ));
    } else {
        spans.push(Span::styled(
            "B:Build  M:Mine  X:Demolish  L:Chronicle  1-3:Speed  q:Quit",
            Style::default().fg(Color::Gray),
        ));
    }

    // Trailing space
    spans.push(Span::raw(" "));

    Line::from(spans)
}

/// Helper to get the status string for headless testing or assertions.
///
/// Wraps [`get_status_line`] and converts the `Line` to a plain `String`.
///
/// # Examples
///
/// ```
/// use scale::ui::status::get_status_string;
/// use scale::shared::time::SimSpeed;
/// use scale::layer1::{BuildMode, DesignationMode};
///
/// let status = get_status_string(
///     10,                 // tick
///     SimSpeed::Normal,   // speed
///     false,              // paused
///     &BuildMode::default(),
///     &DesignationMode::default(),
///     Some("Outpost"),    // location
///     5,                  // pop count
///     100.0,              // food
///     0.0,                // rations
///     10.0,               // tools
///     0.8,                // morale
///     None,               // season
/// );
///
/// assert!(status.contains("Day 10"));
/// assert!(status.contains("Souls: 5"));
/// assert!(status.contains("Outpost"));
/// ```
#[must_use]
#[allow(clippy::too_many_arguments)]
pub fn get_status_string(
    tick: u64,
    speed: SimSpeed,
    paused: bool,
    build_mode: &BuildMode,
    designation_mode: &DesignationMode,
    location_name: Option<&str>,
    pop_count: usize,
    food_yield: f32,
    rations: f32,
    tools: f32,
    morale: f32,
    efficiency: f32,
    season: Option<Season>,
    solar_cycle: Option<SolarCycle>,
) -> String {
    let line = get_status_line(
        tick,
        speed,
        paused,
        build_mode,
        designation_mode,
        location_name,
        pop_count,
        food_yield,
        rations,
        tools,
        morale,
        efficiency,
        season,
        solar_cycle,
    );

    line.spans
        .iter()
        .map(|s| s.content.as_ref())
        .collect::<String>()
}

/// Truncates a status `Line` to fit within `max_width` columns.
///
/// If the combined length of all spans exceeds `max_width`, the line is truncated
/// and the last visible span gets an ellipsis appended.
fn truncate_line(line: Line<'_>, max_width: u16) -> Line<'_> {
    let max = max_width as usize;
    let mut total = 0usize;
    let mut result = Vec::new();
    for span in line.spans {
        let len = span.content.len();
        if total + len <= max {
            result.push(span);
            total += len;
        } else {
            let remaining = max.saturating_sub(total);
            if remaining > 1 {
                let truncated: String = span.content.chars().take(remaining - 1).collect();
                result.push(Span::styled(format!("{truncated}\u{2026}"), span.style));
            }
            break;
        }
    }
    Line::from(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::seasons::Season;
    use crate::layer1::{BuildMode, DesignationMode};
    use crate::shared::time::SimSpeed;

    #[test]
    fn test_get_status_string_formatting() {
        let tick = 100;
        let speed = SimSpeed::Normal;
        let paused = false;
        let build_mode = BuildMode::default();
        let designation_mode = DesignationMode::default();

        // With location
        let status = get_status_string(
            tick,
            speed,
            paused,
            &build_mode,
            &designation_mode,
            Some("Test City"),
            42,    // Pops
            123.0, // Food
            50.0,  // Rations
            10.0,  // Tools
            0.85,  // Morale
            1.0,   // Efficiency
            None,  // Season
            None,  // Solar Cycle
        );

        assert!(status.contains("Day 100"));
        assert!(status.contains("Souls: 42"));
        assert!(status.contains("Morale: 85%"));
        assert!(status.contains("Food: 123+50"));
        assert!(status.contains("Tools: 10"));
        assert!(status.contains("1x"));
        assert!(status.contains("📍 Test City"));

        // Without location
        let status_none = get_status_string(
            tick,
            speed,
            paused,
            &build_mode,
            &designation_mode,
            None,
            0,
            0.0,
            0.0, // Rations
            0.0,
            0.0,
            1.0,  // Efficiency
            None, // Season
            None, // Solar Cycle
        );

        assert!(status_none.contains("Day 100"));
        assert!(!status_none.contains("📍"));
    }

    #[test]
    fn test_render_status_bar_calculates_morale() {
        use crate::layer1::{
            ColonyPolicies, ColonyResources, Memories, MemoryType, NamedLocations, Needs, Pop,
            Viewport,
        };
        use crate::shared::state::GameState;
        use crate::shared::time::SimulationTime;
        use ratatui::backend::TestBackend;
        use ratatui::Terminal;

        let mut world = World::new();
        // Setup resources needed by render_status_bar
        world.insert_resource(SimulationTime::default());
        world.insert_resource(GameState::Running);
        world.insert_resource(BuildMode::default());
        world.insert_resource(DesignationMode::default());
        world.insert_resource(Viewport::default());
        world.insert_resource(NamedLocations::default());
        world.insert_resource(ColonyResources::default());
        world.insert_resource(ColonyPolicies::default());

        world.spawn((
            Pop,
            Needs {
                hunger: 1.0,
                rest: 1.0,
                leisure: 1.0,
                hygiene: 0.8,
            },
            scale::layer1::morale::Morale::default(),
            crate::layer1::traits::Traits::default(),
        )); // Morale 1.0

        world.spawn((
            Pop,
            Needs {
                hunger: 0.0,
                rest: 0.0,
                leisure: 0.0,
                hygiene: 0.8,
            },
            scale::layer1::morale::Morale::default(),
            crate::layer1::traits::Traits::default(),
        )); // Morale 0.0

        // Spawn pop with memory
        // Base 1.0 + (-0.2 witness death) = 0.8
        let mut memories = Memories::default();
        memories.add(MemoryType::WitnessedDeath, 0);
        world.spawn((
            Pop,
            Needs {
                hunger: 1.0,
                rest: 1.0,
                leisure: 1.0,
                hygiene: 0.8,
            },
            memories,
            scale::layer1::morale::Morale::default(),
            crate::layer1::traits::Traits::default(),
        ));

        // Avg = (1.0 + 0.0 + 0.8) / 3 = 1.8 / 3 = 0.6

        let backend = TestBackend::new(100, 1);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|f| {
                render_status_bar(f, f.area(), &world);
            })
            .unwrap();

        // Convert buffer to string to check content
        let buffer = terminal.backend().buffer();
        let cells: Vec<String> = buffer
            .content
            .iter()
            .map(|c| c.symbol().to_string())
            .collect();
        let full_text = cells.join("");

        assert!(full_text.contains("Morale: 60%"));
    }

    #[test]
    fn test_season_display_in_status() {
        let status = get_status_string(
            50,
            SimSpeed::Normal,
            false,
            &BuildMode::default(),
            &DesignationMode::default(),
            None,
            5,
            100.0,
            0.0,
            10.0,
            0.8,
            1.0,
            Some(Season::Summer),
            None,
        );
        assert!(
            status.contains("Summer"),
            "Status should contain season name"
        );
        assert!(status.contains("Day 50"));
    }

    #[test]
    fn test_no_season_when_none() {
        let status = get_status_string(
            50,
            SimSpeed::Normal,
            false,
            &BuildMode::default(),
            &DesignationMode::default(),
            None,
            5,
            100.0,
            0.0,
            10.0,
            0.8,
            1.0,
            None,
            None,
        );
        assert!(!status.contains("Spring"));
        assert!(!status.contains("Summer"));
        assert!(!status.contains("Autumn"));
        assert!(!status.contains("Winter"));
    }

    #[test]
    fn test_solar_cycle_display_in_status() {
        let status = get_status_string(
            50,
            SimSpeed::Normal,
            false,
            &BuildMode::default(),
            &DesignationMode::default(),
            None,
            5,
            100.0,
            0.0,
            10.0,
            0.8,
            1.0,
            None,
            Some(SolarCycle::Maximum),
        );
        assert!(
            status.contains("Solar Maximum"),
            "Status should contain solar cycle name"
        );
    }

    #[test]
    fn test_truncate_line_no_truncation() {
        let line = Line::from(vec![Span::raw("Hello"), Span::raw(" World")]);
        let result = truncate_line(line, 20);
        let text: String = result.spans.iter().map(|s| s.content.as_ref()).collect();
        assert_eq!(text, "Hello World");
    }

    #[test]
    fn test_truncate_line_truncates() {
        let line = Line::from(vec![Span::raw("Hello"), Span::raw(" World, this is long")]);
        // max_width = 10, "Hello" = 5 fits, " World, this is long" = 20 doesn't
        // remaining = 10 - 5 = 5, take 4 chars + ellipsis = " Wor…"
        let result = truncate_line(line, 10);
        let text: String = result.spans.iter().map(|s| s.content.as_ref()).collect();
        assert_eq!(text, "Hello Wor\u{2026}");
    }
}

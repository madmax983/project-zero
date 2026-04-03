use super::{
    commands::{build_default_command_registry, CommandRegistry},
    config::ShellConfig,
    plugins::{
        register_default_plugins_with_runtime, SharedWorld, CHRONICLE_PLUGIN_TYPE,
        COLONY_MAP_PLUGIN_TYPE, INSPECTOR_PLUGIN_TYPE, STATUS_PLUGIN_TYPE, SYSTEM_MAP_PLUGIN_TYPE,
        TECH_PLUGIN_TYPE,
    },
};
use crate::platform::input::{GameKeyCode, GameKeyEvent};
use ratatui::{
    buffer::Buffer,
    layout::{Direction, Rect},
};
use ratatui_hypertile::{HypertileEvent, KeyChord, KeyCode, Modifiers, PaneId};
use ratatui_hypertile_extras::{HypertileRuntime, InputMode, SplitBehavior, WorkspaceRuntime};

const COLONY_OPS_WORKSPACE: &str = "Colony Ops";
const SYSTEM_SURVEY_WORKSPACE: &str = "System Survey";
const DIRECTOR_WORKSPACE: &str = "Director";

/// Bootstrapped shell state for the hypertile migration.
///
/// This stays outside the ECS world for now because the runtime's plugin factories capture the
/// world handle; storing that back inside `World` would create a self-referential knot.
pub struct UiShell {
    workspaces: WorkspaceRuntime,
    workspace_names: Vec<String>,
    config: ShellConfig,
    commands: CommandRegistry,
}

impl UiShell {
    #[must_use]
    pub fn has_workspace(&self, name: &str) -> bool {
        self.workspace_names
            .iter()
            .any(|workspace| workspace == name)
    }

    #[must_use]
    pub fn workspaces(&self) -> &WorkspaceRuntime {
        &self.workspaces
    }

    #[must_use]
    pub fn workspaces_mut(&mut self) -> &mut WorkspaceRuntime {
        &mut self.workspaces
    }

    #[must_use]
    pub fn active_workspace_name(&self) -> &str {
        &self.workspace_names[self.workspaces.active_tab_index()]
    }

    #[must_use]
    pub fn config(&self) -> &ShellConfig {
        &self.config
    }

    #[must_use]
    pub fn command_registry(&self) -> &CommandRegistry {
        &self.commands
    }

    #[must_use]
    pub fn is_layout_mode(&self) -> bool {
        self.workspaces.active_runtime().mode() == InputMode::Layout
    }

    #[must_use]
    pub fn switch_to_workspace(&mut self, name: &str) -> bool {
        if let Some(index) = self
            .workspace_names
            .iter()
            .position(|workspace| workspace == name)
        {
            self.workspaces.go_to_tab(index);
            true
        } else {
            false
        }
    }

    pub fn render(&mut self, area: Rect, buf: &mut Buffer) {
        self.workspaces.render(area, buf);
    }

    #[must_use]
    pub fn open_palette(&mut self) -> bool {
        let runtime = self.workspaces.active_runtime_mut();
        runtime.set_mode(InputMode::Layout);
        runtime
            .handle_event(HypertileEvent::Key(KeyChord::new(KeyCode::Char('p'))))
            .is_consumed()
    }

    #[must_use]
    pub fn handle_key(&mut self, key: GameKeyEvent) -> bool {
        let code = match key.code {
            GameKeyCode::Char(ch) => KeyCode::Char(ch),
            GameKeyCode::Enter => KeyCode::Enter,
            GameKeyCode::Esc => KeyCode::Escape,
            GameKeyCode::Tab => KeyCode::Tab,
            GameKeyCode::BackTab => KeyCode::BackTab,
            GameKeyCode::Backspace => KeyCode::Backspace,
            GameKeyCode::Delete => KeyCode::Delete,
            GameKeyCode::Up => KeyCode::Up,
            GameKeyCode::Down => KeyCode::Down,
            GameKeyCode::Left => KeyCode::Left,
            GameKeyCode::Right => KeyCode::Right,
        };

        let mut modifiers = Modifiers::NONE;
        if key.modifiers.shift {
            modifiers |= Modifiers::SHIFT;
        }
        if key.modifiers.ctrl {
            modifiers |= Modifiers::CTRL;
        }
        if key.modifiers.alt {
            modifiers |= Modifiers::ALT;
        }

        self.workspaces
            .handle_event(HypertileEvent::Key(KeyChord::with_modifiers(
                code, modifiers,
            )))
            .is_consumed()
    }
}

#[must_use]
pub fn build_default_shell(world: SharedWorld, config: ShellConfig) -> UiShell {
    let commands = build_default_command_registry();
    let workspace_names = vec![
        String::from(COLONY_OPS_WORKSPACE),
        String::from(SYSTEM_SURVEY_WORKSPACE),
        String::from(DIRECTOR_WORKSPACE),
    ];

    let mut workspaces = WorkspaceRuntime::new(|| {
        HypertileRuntime::builder()
            .with_split_behavior(SplitBehavior::PromptPalette)
            .with_default_split_plugin(COLONY_MAP_PLUGIN_TYPE)
    });

    register_default_plugins_with_runtime(workspaces.active_runtime_mut(), world.clone());
    workspaces.rename_tab(0, String::from(COLONY_OPS_WORKSPACE));
    apply_workspace_preset(workspaces.active_runtime_mut(), COLONY_OPS_WORKSPACE);
    workspaces
        .active_runtime_mut()
        .set_mode(InputMode::PluginInput);

    for workspace_name in [
        SYSTEM_SURVEY_WORKSPACE.to_string(),
        DIRECTOR_WORKSPACE.to_string(),
    ] {
        workspaces.new_tab();
        register_default_plugins_with_runtime(workspaces.active_runtime_mut(), world.clone());
        workspaces.rename_tab(workspaces.active_tab_index(), workspace_name.clone());
        apply_workspace_preset(workspaces.active_runtime_mut(), &workspace_name);
        workspaces
            .active_runtime_mut()
            .set_mode(InputMode::PluginInput);
    }

    if let Some(index) = workspace_names
        .iter()
        .position(|workspace| workspace == &config.startup_workspace)
    {
        workspaces.go_to_tab(index);
    } else {
        workspaces.go_to_tab(0);
    }

    UiShell {
        workspaces,
        workspace_names,
        config,
        commands,
    }
}

fn apply_workspace_preset(runtime: &mut HypertileRuntime, workspace_name: &str) {
    match workspace_name {
        COLONY_OPS_WORKSPACE => configure_colony_ops(runtime),
        SYSTEM_SURVEY_WORKSPACE => configure_system_survey(runtime),
        DIRECTOR_WORKSPACE => configure_director(runtime),
        _ => configure_colony_ops(runtime),
    }
}

fn configure_colony_ops(runtime: &mut HypertileRuntime) {
    runtime.reset();
    runtime
        .replace_focused_plugin(COLONY_MAP_PLUGIN_TYPE)
        .expect("colony ops should replace the placeholder root");

    let status_pane = runtime
        .split_focused(Direction::Vertical, STATUS_PLUGIN_TYPE)
        .expect("colony ops should add a status pane");
    runtime
        .focus_pane(PaneId::ROOT)
        .expect("colony ops should re-focus the primary map pane");
    let inspector_pane = runtime
        .split_focused(Direction::Horizontal, INSPECTOR_PLUGIN_TYPE)
        .expect("colony ops should add an inspector pane");
    runtime
        .focus_pane(inspector_pane)
        .expect("colony ops should focus the inspector side stack");
    let _chronicle_pane = runtime
        .split_focused(Direction::Vertical, CHRONICLE_PLUGIN_TYPE)
        .expect("colony ops should add a chronicle pane");
    runtime
        .focus_pane(status_pane)
        .expect("colony ops should focus the status strip after bootstrapping");
}

fn configure_system_survey(runtime: &mut HypertileRuntime) {
    runtime.reset();
    runtime
        .replace_focused_plugin(SYSTEM_MAP_PLUGIN_TYPE)
        .expect("system survey should replace the placeholder root");

    let status_pane = runtime
        .split_focused(Direction::Vertical, STATUS_PLUGIN_TYPE)
        .expect("system survey should add a status pane");
    runtime
        .focus_pane(PaneId::ROOT)
        .expect("system survey should re-focus the map pane");
    let _inspector_pane = runtime
        .split_focused(Direction::Horizontal, INSPECTOR_PLUGIN_TYPE)
        .expect("system survey should add an inspector pane");
    runtime
        .focus_pane(status_pane)
        .expect("system survey should focus the status strip after bootstrapping");
}

fn configure_director(runtime: &mut HypertileRuntime) {
    runtime.reset();
    runtime
        .replace_focused_plugin(CHRONICLE_PLUGIN_TYPE)
        .expect("director should replace the placeholder root");

    let tech_pane = runtime
        .split_focused(Direction::Horizontal, TECH_PLUGIN_TYPE)
        .expect("director should add a tech pane");
    runtime
        .focus_pane(PaneId::ROOT)
        .expect("director should re-focus the chronicle pane");
    let status_pane = runtime
        .split_focused(Direction::Vertical, STATUS_PLUGIN_TYPE)
        .expect("director should add a status pane");
    runtime
        .focus_pane(tech_pane)
        .expect("director should focus the tech pane");
    let _inspector_pane = runtime
        .split_focused(Direction::Vertical, INSPECTOR_PLUGIN_TYPE)
        .expect("director should add an inspector pane");
    runtime
        .focus_pane(status_pane)
        .expect("director should focus the status pane after bootstrapping");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::{setup_world_with_config, SetupConfig};
    use std::{cell::RefCell, rc::Rc};

    #[test]
    fn default_shell_contains_curated_workspaces() {
        let shell = build_default_shell_for_test();

        assert!(shell.has_workspace(COLONY_OPS_WORKSPACE));
        assert!(shell.has_workspace(SYSTEM_SURVEY_WORKSPACE));
        assert!(shell.has_workspace(DIRECTOR_WORKSPACE));
    }

    #[test]
    fn command_registry_exposes_pause_and_open_chronicle() {
        let registry = build_command_registry_for_test();

        assert!(registry.contains("Pause Simulation"));
        assert!(registry.contains("Open Chronicle"));
    }

    fn build_default_shell_for_test() -> UiShell {
        let world = Rc::new(RefCell::new(setup_world_with_config(SetupConfig {
            headless: true,
        })));
        build_default_shell(world, ShellConfig::default())
    }

    fn build_command_registry_for_test() -> CommandRegistry {
        build_default_command_registry()
    }
}

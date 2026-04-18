use bevy_ecs::prelude::Resource;
use ratatui_hypertile::raw::{Node as LayoutNode, PaneId};
use serde::{Deserialize, Serialize};

pub const SHELL_LAYOUT_VERSION: u32 = 2;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PersistedShellLayout {
    pub version: u32,
    pub active_workspace: String,
    pub workspaces: Vec<PersistedWorkspaceLayout>,
}

impl Default for PersistedShellLayout {
    fn default() -> Self {
        Self {
            version: SHELL_LAYOUT_VERSION,
            active_workspace: String::from("Colony Ops"),
            workspaces: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PersistedWorkspaceLayout {
    pub name: String,
    pub root: LayoutNode,
    pub focused_pane: Option<PaneId>,
    pub panes: Vec<PersistedPaneBinding>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersistedPaneBinding {
    pub pane_id: PaneId,
    pub plugin_type: String,
}

/// Persisted shell configuration for the hypertile UI.
#[derive(Clone, Debug, PartialEq, Eq, Resource, Serialize, Deserialize)]
pub struct ShellConfig {
    /// Workspace opened at startup.
    pub startup_workspace: String,
    /// Theme or palette name.
    pub theme: String,
    /// Recently used commands.
    pub recent_commands: Vec<String>,
    /// Placeholder for persisted layout/workspace state.
    pub persisted_layout: Option<String>,
}

impl Default for ShellConfig {
    fn default() -> Self {
        Self {
            startup_workspace: String::from("Colony Ops"),
            theme: String::from("default"),
            recent_commands: Vec::new(),
            persisted_layout: None,
        }
    }
}

impl ShellConfig {
    #[must_use]
    pub fn from_json_or_default(json: &str) -> Self {
        load_shell_config(json)
    }

    #[must_use]
    pub fn decoded_persisted_layout(&self) -> Option<PersistedShellLayout> {
        self.persisted_layout
            .as_ref()
            .and_then(|json| serde_json::from_str(json).ok())
            .filter(|layout: &PersistedShellLayout| layout.version == SHELL_LAYOUT_VERSION)
    }

    pub fn store_persisted_layout(
        &mut self,
        layout: &PersistedShellLayout,
    ) -> Result<(), serde_json::Error> {
        self.persisted_layout = Some(serde_json::to_string(layout)?);
        Ok(())
    }
}

#[must_use]
pub fn load_shell_config(json: &str) -> ShellConfig {
    if json.len() > 1024 * 1024 { // 1MB limit
        return ShellConfig::default();
    }
    serde_json::from_str(json).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::layout::Direction;
    use ratatui_hypertile::raw::Node as LayoutNode;

    #[test]
    fn shell_config_defaults_to_colony_ops() {
        let cfg = ShellConfig::default();
        assert_eq!(cfg.startup_workspace, "Colony Ops");
        assert_eq!(cfg.theme, "default");
        assert!(cfg.recent_commands.is_empty());
        assert!(cfg.persisted_layout.is_none());
    }

    #[test]
    fn shell_config_round_trips() {
        let mut cfg = ShellConfig {
            startup_workspace: String::from("Director"),
            theme: String::from("midnight"),
            recent_commands: vec![String::from("Open Chronicle")],
            persisted_layout: None,
        };
        cfg.store_persisted_layout(&PersistedShellLayout {
            version: SHELL_LAYOUT_VERSION,
            active_workspace: String::from("Director"),
            workspaces: vec![PersistedWorkspaceLayout {
                name: String::from("Director"),
                root: LayoutNode::Split {
                    direction: Direction::Horizontal,
                    ratio: 0.5,
                    first: Box::new(LayoutNode::Pane(PaneId::ROOT)),
                    second: Box::new(LayoutNode::Pane(PaneId::new(1))),
                },
                focused_pane: Some(PaneId::new(1)),
                panes: vec![
                    PersistedPaneBinding {
                        pane_id: PaneId::ROOT,
                        plugin_type: String::from("chronicle"),
                    },
                    PersistedPaneBinding {
                        pane_id: PaneId::new(1),
                        plugin_type: String::from("tech"),
                    },
                ],
            }],
        })
        .unwrap();

        let json = serde_json::to_string(&cfg).unwrap();
        let decoded: ShellConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(decoded.startup_workspace, cfg.startup_workspace);
        assert_eq!(decoded.theme, cfg.theme);
        assert_eq!(decoded.recent_commands, cfg.recent_commands);
        assert!(decoded.decoded_persisted_layout().is_some());
    }

    #[test]
    fn invalid_shell_config_json_falls_back_to_default() {
        let cfg = load_shell_config("{broken");

        assert_eq!(cfg.startup_workspace, "Colony Ops");
        assert!(cfg.persisted_layout.is_none());
    }

    #[test]
    fn persisted_layout_with_old_version_is_ignored() {
        let cfg = ShellConfig {
            persisted_layout: Some(
                r#"{"version":1,"active_workspace":"Colony Ops","workspaces":[]}"#.to_string(),
            ),
            ..Default::default()
        };

        assert!(cfg.decoded_persisted_layout().is_none());
    }
}

use bevy_ecs::prelude::Resource;
use serde::{Deserialize, Serialize};

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

#[cfg(test)]
mod tests {
    use super::*;

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
        let cfg = ShellConfig {
            startup_workspace: String::from("Director"),
            theme: String::from("midnight"),
            recent_commands: vec![String::from("Open Chronicle")],
            persisted_layout: Some(String::from("layout-v1")),
        };

        let json = serde_json::to_string(&cfg).unwrap();
        let decoded: ShellConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(decoded, cfg);
    }
}

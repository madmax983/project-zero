//! Hypertile shell entry points.
//!
//! This module will host the runtime shell, plugin registry, and palette wiring.

pub mod commands;
pub mod config;
pub mod plugins;
pub mod runtime;

pub use commands::{
    build_default_command_registry, CommandRegistry, ShellCommand, ShellCommandAction,
    ShellCommandDomain,
};
pub use config::{
    load_shell_config, PersistedPaneBinding, PersistedShellLayout, PersistedWorkspaceLayout,
    ShellConfig, SHELL_LAYOUT_VERSION,
};
pub use runtime::{build_default_shell, UiShell};

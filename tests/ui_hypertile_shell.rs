#![allow(missing_docs)]

use scale::{
    prelude::{setup_world_with_config, SetupConfig},
    ui::shell::{build_default_shell, ShellConfig},
};
use std::{cell::RefCell, rc::Rc};

#[test]
fn invalid_shell_layout_falls_back_to_colony_ops() {
    let world = Rc::new(RefCell::new(setup_world_with_config(SetupConfig {
        headless: true,
    })));
    let config = ShellConfig::from_json_or_default("{\"broken\":true}");

    let shell = build_default_shell(world, config);

    assert_eq!(shell.active_workspace_name(), "Colony Ops");
}

#[test]
fn persisted_shell_layout_restores_active_workspace_and_panes() {
    let world = Rc::new(RefCell::new(setup_world_with_config(SetupConfig {
        headless: true,
    })));
    let base_config = world.borrow().resource::<ShellConfig>().clone();
    let mut shell = build_default_shell(Rc::clone(&world), base_config);

    assert!(shell.execute_command_by_label("Switch to System Survey"));
    assert!(shell.execute_command_by_label("Open Chronicle"));
    assert!(shell.active_workspace_contains_plugin("chronicle"));

    let persisted = shell.snapshot_config();
    let json = serde_json::to_string(&persisted).expect("shell config should serialize");

    let restored_world = Rc::new(RefCell::new(setup_world_with_config(SetupConfig {
        headless: true,
    })));
    let restored_config = ShellConfig::from_json_or_default(&json);
    let restored_shell = build_default_shell(restored_world, restored_config);

    assert_eq!(restored_shell.active_workspace_name(), "System Survey");
    assert!(restored_shell.active_workspace_contains_plugin("chronicle"));
}

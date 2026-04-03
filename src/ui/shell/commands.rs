#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ShellCommandDomain {
    Shell,
    Gameplay,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ShellCommandAction {
    SwitchWorkspace(&'static str),
    OpenPane(&'static str),
    PauseSimulation,
    ResumeSimulation,
    SetSimulationSpeed(u8),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShellCommand {
    pub id: &'static str,
    pub label: &'static str,
    pub domain: ShellCommandDomain,
    pub action: ShellCommandAction,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CommandRegistry {
    commands: Vec<ShellCommand>,
}

impl CommandRegistry {
    #[must_use]
    pub fn new(commands: Vec<ShellCommand>) -> Self {
        Self { commands }
    }

    #[must_use]
    pub fn commands(&self) -> &[ShellCommand] {
        &self.commands
    }

    #[must_use]
    pub fn contains(&self, label: &str) -> bool {
        self.commands.iter().any(|command| command.label == label)
    }
}

#[must_use]
pub fn build_default_command_registry() -> CommandRegistry {
    CommandRegistry::new(vec![
        ShellCommand {
            id: "workspace.colony_ops",
            label: "Switch to Colony Ops",
            domain: ShellCommandDomain::Shell,
            action: ShellCommandAction::SwitchWorkspace("Colony Ops"),
        },
        ShellCommand {
            id: "workspace.system_survey",
            label: "Switch to System Survey",
            domain: ShellCommandDomain::Shell,
            action: ShellCommandAction::SwitchWorkspace("System Survey"),
        },
        ShellCommand {
            id: "workspace.director",
            label: "Switch to Director",
            domain: ShellCommandDomain::Shell,
            action: ShellCommandAction::SwitchWorkspace("Director"),
        },
        ShellCommand {
            id: "pane.colony_map",
            label: "Open Colony Map",
            domain: ShellCommandDomain::Shell,
            action: ShellCommandAction::OpenPane("colony-map"),
        },
        ShellCommand {
            id: "pane.system_map",
            label: "Open System Map",
            domain: ShellCommandDomain::Shell,
            action: ShellCommandAction::OpenPane("system-map"),
        },
        ShellCommand {
            id: "pane.inspector",
            label: "Open Inspector",
            domain: ShellCommandDomain::Shell,
            action: ShellCommandAction::OpenPane("inspector"),
        },
        ShellCommand {
            id: "pane.status",
            label: "Open Status",
            domain: ShellCommandDomain::Shell,
            action: ShellCommandAction::OpenPane("status"),
        },
        ShellCommand {
            id: "pane.chronicle",
            label: "Open Chronicle",
            domain: ShellCommandDomain::Gameplay,
            action: ShellCommandAction::OpenPane("chronicle"),
        },
        ShellCommand {
            id: "pane.tech",
            label: "Open Tech Tree",
            domain: ShellCommandDomain::Gameplay,
            action: ShellCommandAction::OpenPane("tech"),
        },
        ShellCommand {
            id: "sim.pause",
            label: "Pause Simulation",
            domain: ShellCommandDomain::Gameplay,
            action: ShellCommandAction::PauseSimulation,
        },
        ShellCommand {
            id: "sim.resume",
            label: "Resume Simulation",
            domain: ShellCommandDomain::Gameplay,
            action: ShellCommandAction::ResumeSimulation,
        },
        ShellCommand {
            id: "sim.speed_1",
            label: "Set Simulation Speed 1x",
            domain: ShellCommandDomain::Gameplay,
            action: ShellCommandAction::SetSimulationSpeed(1),
        },
        ShellCommand {
            id: "sim.speed_2",
            label: "Set Simulation Speed 2x",
            domain: ShellCommandDomain::Gameplay,
            action: ShellCommandAction::SetSimulationSpeed(2),
        },
        ShellCommand {
            id: "sim.speed_3",
            label: "Set Simulation Speed 3x",
            domain: ShellCommandDomain::Gameplay,
            action: ShellCommandAction::SetSimulationSpeed(3),
        },
    ])
}

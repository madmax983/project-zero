use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use crate::layer1::energy::PowerConsumer;
use crate::layer1::core::ai_core::RaidDetected;

/// Defines the physical state of a SmartMatter entity.
#[derive(Component, PartialEq, Debug)]
pub enum MatterState {
    /// Default state, functioning as a window or basic wall.
    Window,
    /// Hardened state during hostile events.
    Bunker,
    /// Logistics state for automated transport.
    Conveyor,
}

/// Marks an entity as composed of smart matter, allowing state transitions.
#[derive(Component)]
pub struct SmartMatter {
    /// The current state of the smart matter.
    pub state: MatterState,
}

/// Marker component for smart buildings.
#[derive(Component)]
pub struct SmartBuilding;

/// Evaluates conditions (like raids) and changes the state of powered SmartMatter entities.
pub fn update_smart_matter_state(
    mut query: Query<(&mut SmartMatter, &PowerConsumer)>,
    raid_detected: Option<Res<RaidDetected>>,
) {
    let raid_active = raid_detected.is_some_and(|r| r.active);

    for (mut smart_matter, power) in query.iter_mut() {
        if !power.active {
            continue;
        }

        if raid_active && smart_matter.state != MatterState::Bunker {
            smart_matter.state = MatterState::Bunker;
        } else if !raid_active && smart_matter.state != MatterState::Window {
            // Revert back to default state
            smart_matter.state = MatterState::Window;
        }
    }
}

/// Registers the smart matter systems to the application.
pub struct SmartMatterPlugin;

impl Plugin for SmartMatterPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_smart_matter_state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::architecture::Building;

    #[test]
    fn test_smart_matter_state_change() {
        // Arrange: Setup app, add smart matter building, power grid, and raid event
        let mut app = App::new();
        app.add_plugins(SmartMatterPlugin);
        app.insert_resource(RaidDetected { active: true });
        let entity = app
            .world_mut()
            .spawn((
                Building {
                    building_type: crate::layer1::architecture::BuildingType::Wall,
                },
                SmartMatter {
                    state: MatterState::Window,
                },
                PowerConsumer { active: true, demand: 10.0 },
            ))
            .id();

        // Act: Trigger raid and run system
        app.update();

        // Assert: State should change to Bunker
        let smart_matter = app.world().get::<SmartMatter>(entity).unwrap();
        assert_eq!(smart_matter.state, MatterState::Bunker);
    }

    #[test]
    fn test_smart_matter_power_failure() {
        // Arrange: Setup app, add smart matter building without power
        let mut app = App::new();
        app.add_plugins(SmartMatterPlugin);
        app.insert_resource(RaidDetected { active: true });
        let entity = app
            .world_mut()
            .spawn((
                Building {
                    building_type: crate::layer1::architecture::BuildingType::Wall,
                },
                SmartMatter {
                    state: MatterState::Window,
                },
                PowerConsumer { active: false, demand: 10.0 },
            ))
            .id();

        // Act: Trigger raid and run system
        app.update();

        // Assert: State should remain Window due to lack of power
        let smart_matter = app.world().get::<SmartMatter>(entity).unwrap();
        assert_eq!(smart_matter.state, MatterState::Window);
    }
}

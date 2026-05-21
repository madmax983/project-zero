use bevy::prelude::*;

#[derive(Component)]
pub struct PirateHaven {
    pub level: u32,
    pub wealth: u32,
}

#[derive(Component)]
pub struct PirateRepublic;

#[derive(Component)]
pub struct Faction {
    pub name: String,
    pub relationship_score: i32,
}

#[derive(Event)]
pub struct RaidSuccessEvent {
    pub haven_entity: Entity,
    pub loot_value: u32,
}

const UPGRADE_THRESHOLD_BASE: u32 = 500;
const MAX_HAVEN_LEVEL: u32 = 5;

pub fn process_raid_success_system(
    mut events: EventReader<RaidSuccessEvent>,
    mut havens: Query<&mut PirateHaven>,
) {
    for event in events.read() {
        if let Ok(mut haven) = havens.get_mut(event.haven_entity) {
            haven.wealth += event.loot_value;
        }
    }
}

pub fn haven_upgrade_system(mut havens: Query<&mut PirateHaven>) {
    for mut haven in havens.iter_mut() {
        let upgrade_cost = UPGRADE_THRESHOLD_BASE * haven.level;
        if haven.wealth >= upgrade_cost && haven.level < MAX_HAVEN_LEVEL {
            haven.wealth -= upgrade_cost;
            haven.level += 1;
        }
    }
}

pub fn haven_to_republic_system(mut commands: Commands, havens: Query<(Entity, &PirateHaven)>) {
    for (entity, haven) in havens.iter() {
        if haven.level >= MAX_HAVEN_LEVEL {
            commands
                .entity(entity)
                .remove::<PirateHaven>()
                .insert(PirateRepublic)
                .insert(Faction {
                    name: "New Pirate Republic".to_string(),
                    relationship_score: -50, // Initially hostile
                });
        }
    }
}

pub struct PiracyPlugin;

impl Plugin for PiracyPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<RaidSuccessEvent>();
        app.add_systems(
            Update,
            (
                process_raid_success_system,
                haven_upgrade_system,
                haven_to_republic_system,
            ),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_raid_success_increases_haven_wealth() {
        // Arrange
        let mut app = App::new();
        app.add_event::<RaidSuccessEvent>();
        app.add_systems(Update, process_raid_success_system);

        let haven = app
            .world_mut()
            .spawn((PirateHaven {
                level: 1,
                wealth: 100,
            },))
            .id();

        // Act
        app.world_mut().send_event(RaidSuccessEvent {
            haven_entity: haven,
            loot_value: 50,
        });
        app.update();

        // Assert
        let haven_comp = app.world().get::<PirateHaven>(haven).unwrap();
        assert_eq!(haven_comp.wealth, 150);
    }

    #[test]
    fn test_haven_upgrades_when_wealth_threshold_met() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, haven_upgrade_system);

        let haven = app
            .world_mut()
            .spawn((PirateHaven {
                level: 1,
                wealth: 550,
            },))
            .id();

        // Act
        app.update();

        // Assert
        let haven_comp = app.world().get::<PirateHaven>(haven).unwrap();
        assert_eq!(haven_comp.level, 2);
        assert_eq!(haven_comp.wealth, 50); // Wealth consumed for upgrade (threshold 500)
    }

    #[test]
    fn test_max_level_haven_becomes_republic() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, haven_to_republic_system);

        let haven = app
            .world_mut()
            .spawn((PirateHaven {
                level: 5,
                wealth: 1000,
            },))
            .id();

        // Act
        app.update();

        // Assert
        let haven_comp = app.world().get::<PirateHaven>(haven);
        assert!(haven_comp.is_none());

        let republic = app.world().get::<PirateRepublic>(haven);
        assert!(republic.is_some());

        let faction = app.world().get::<Faction>(haven);
        assert!(faction.is_some());
    }
}

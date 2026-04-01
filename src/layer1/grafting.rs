use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct TechLevel {
    pub level: u32,
}

#[derive(Component)]
pub struct MaintenanceDebt {
    pub amount: f32,
}

#[derive(Component)]
pub struct Quirks {
    pub list: Vec<String>,
}

#[derive(Component)]
pub struct GraftedModule {
    pub efficiency_bonus: f32,
}

#[derive(Event)]
pub struct GraftBuildingEvent {
    pub target: Entity,
    pub new_module_tech_level: u32,
    pub efficiency_bonus: f32,
}

pub fn process_grafting(
    mut commands: Commands,
    mut events: EventReader<GraftBuildingEvent>,
    mut buildings: Query<(&mut TechLevel, &mut MaintenanceDebt, &mut Quirks)>,
) {
    for event in events.read() {
        if let Ok((mut tech, mut debt, mut quirks)) = buildings.get_mut(event.target) {
            tech.level = event.new_module_tech_level;
            debt.amount += 20.0; // Grafting penalty
            quirks.list.push("Frankenstein Architecture".to_string());

            commands.entity(event.target).insert(GraftedModule {
                efficiency_bonus: event.efficiency_bonus,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::App;
    use bevy_app::Update;

    #[test]
    fn test_grafting_inherits_quirks_and_debt() {
        let mut app = App::new();
        app.add_event::<GraftBuildingEvent>();
        app.add_systems(Update, process_grafting);

        let base_building = app
            .world_mut()
            .spawn((
                TechLevel { level: 1 },
                MaintenanceDebt { amount: 50.0 },
                Quirks {
                    list: vec!["Leaky Vents".to_string()],
                },
            ))
            .id();

        app.world_mut().send_event(GraftBuildingEvent {
            target: base_building,
            new_module_tech_level: 3,
            efficiency_bonus: 2.0,
        });

        app.update();

        // Target should now have grafted component and increased tech level, retaining old quirks and debt
        let tech = app.world().get::<TechLevel>(base_building).unwrap();
        assert_eq!(tech.level, 3);

        let graft = app.world().get::<GraftedModule>(base_building).unwrap();
        assert_eq!(graft.efficiency_bonus, 2.0);

        let quirks = app.world().get::<Quirks>(base_building).unwrap();
        assert!(quirks.list.contains(&"Leaky Vents".to_string()));
        assert!(
            quirks
                .list
                .contains(&"Frankenstein Architecture".to_string()),
            "Should add grafting quirk"
        );

        let debt = app.world().get::<MaintenanceDebt>(base_building).unwrap();
        assert!(
            debt.amount > 50.0,
            "Grafting should increase maintenance debt baseline"
        );
    }
}

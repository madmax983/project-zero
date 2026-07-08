//! Cascade
//!
//! Handles cascading failure scenarios and systemic collapses.

pub mod failure {
    #[derive(Event, Debug)]
    pub struct LogisticsStrainedEvent {
        pub entity: Entity,
        pub capacity: u32,
        pub utilized: u32,
    }

    #[derive(Event, Debug)]
    pub struct DefenseWeakenedEvent {
        pub entity: Entity,
        pub power: u32,
    }

    use bevy::prelude::*;

    #[derive(Component)]
    pub struct ColonyOutput {
        pub expected_food: u32,
        pub actual_food: u32,
        pub expected_parts: u32,
        pub actual_parts: u32,
    }

    #[derive(Component)]
    pub struct SystemLogistics {
        pub capacity: u32,
        pub utilized: u32,
    }

    #[derive(Component)]
    pub struct SectorDefense {
        pub power: u32,
    }

    #[derive(Component)]
    pub struct InvasionThreat {
        pub level: f32,
    }

    pub fn evaluate_system_logistics(
        mut query: Query<(Entity, &mut SystemLogistics, &ColonyOutput)>,
        mut events: EventWriter<LogisticsStrainedEvent>,
    ) {
        for (entity, mut logistics, output) in query.iter_mut() {
            let expected_total = output.expected_food + output.expected_parts;
            let actual_total = output.actual_food + output.actual_parts;
            if expected_total > 0 {
                let ratio = actual_total as f32 / expected_total as f32;
                logistics.capacity = (100.0 * ratio) as u32; // Assuming base capacity of 100

                if logistics.utilized > logistics.capacity {
                    events.send(LogisticsStrainedEvent {
                        entity,
                        capacity: logistics.capacity,
                        utilized: logistics.utilized,
                    });
                }
            }
        }
    }

    pub fn update_sector_defenses(
        mut query: Query<(Entity, &mut SectorDefense, &SystemLogistics)>,
        mut events: EventWriter<DefenseWeakenedEvent>,
    ) {
        for (entity, mut defense, logistics) in query.iter_mut() {
            if logistics.capacity > 0 {
                let strain = logistics.utilized as f32 / logistics.capacity as f32;
                if strain >= 1.0 {
                    defense.power = (defense.power as f32 * 0.95) as u32; // Lose 5% power per tick if strained
                    events.send(DefenseWeakenedEvent {
                        entity,
                        power: defense.power,
                    });
                }
            } else {
                defense.power = (defense.power as f32 * 0.75) as u32; // Extreme penalty if no capacity
                events.send(DefenseWeakenedEvent {
                    entity,
                    power: defense.power,
                });
            }
        }
    }

    pub fn calculate_invasion_threat(mut query: Query<(&SectorDefense, &mut InvasionThreat)>) {
        for (defense, mut threat) in query.iter_mut() {
            if defense.power < 500 {
                threat.level += 1.0;
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_layer1_shortage_reduces_layer2_logistics() {
            // Arrange
            let mut app = App::new();
            app.add_systems(Update, evaluate_system_logistics);
            app.add_event::<LogisticsStrainedEvent>();

            let entity = app
                .world_mut()
                .spawn((
                    SystemLogistics {
                        capacity: 100,
                        utilized: 50,
                    },
                    ColonyOutput {
                        expected_food: 50,
                        actual_food: 25,
                        expected_parts: 50,
                        actual_parts: 25,
                    }, // 50% output due to shortage
                ))
                .id();

            // Act
            app.update();

            // Assert
            let logistics = app
                .world()
                .get::<SystemLogistics>(entity)
                .expect("SystemLogistics should exist");
            assert_eq!(logistics.capacity, 50); // Capacity drops to match colony output ratio
        }

        #[test]
        fn test_logistics_strain_weakens_defenses() {
            // Arrange
            let mut app = App::new();
            app.add_systems(Update, update_sector_defenses);
            app.add_event::<DefenseWeakenedEvent>();

            let entity = app
                .world_mut()
                .spawn((
                    SectorDefense { power: 1000 },
                    SystemLogistics {
                        capacity: 50,
                        utilized: 50,
                    }, // 100% utilized/strained
                ))
                .id();

            // Act
            app.update();

            // Assert
            let defense = app
                .world()
                .get::<SectorDefense>(entity)
                .expect("SectorDefense should exist");
            assert!(defense.power < 1000); // Defenses should weaken under strain
        }

        #[test]
        fn test_weak_defenses_increase_invasion_threat() {
            // Arrange
            let mut app = App::new();
            app.add_systems(Update, calculate_invasion_threat);

            let entity = app
                .world_mut()
                .spawn((
                    SectorDefense { power: 200 }, // Weak
                    InvasionThreat { level: 0.0 },
                ))
                .id();

            // Act
            app.update();

            // Assert
            let threat = app
                .world()
                .get::<InvasionThreat>(entity)
                .expect("InvasionThreat should exist");
            assert!(threat.level > 0.0);
        }

        #[test]
        fn test_evaluate_system_logistics_no_expected_output() {
            let mut app = App::new();
            app.add_systems(Update, evaluate_system_logistics);
            app.add_event::<LogisticsStrainedEvent>();

            let entity = app
                .world_mut()
                .spawn((
                    SystemLogistics {
                        capacity: 100,
                        utilized: 50,
                    },
                    ColonyOutput {
                        expected_food: 0,
                        actual_food: 0,
                        expected_parts: 0,
                        actual_parts: 0,
                    },
                ))
                .id();

            app.update();

            let logistics = app.world().get::<SystemLogistics>(entity).expect("Component should exist");
            assert_eq!(
                logistics.capacity, 100,
                "Capacity should remain unchanged when expected is 0"
            );

            let events = app.world().resource::<Events<LogisticsStrainedEvent>>();
            assert!(
                events.get_cursor().read(events).next().is_none(),
                "No event should be sent"
            );
        }

        #[test]
        fn test_evaluate_system_logistics_under_capacity() {
            let mut app = App::new();
            app.add_systems(Update, evaluate_system_logistics);
            app.add_event::<LogisticsStrainedEvent>();

            let entity = app
                .world_mut()
                .spawn((
                    SystemLogistics {
                        capacity: 100,
                        utilized: 10,
                    },
                    ColonyOutput {
                        expected_food: 50,
                        actual_food: 50,
                        expected_parts: 50,
                        actual_parts: 50,
                    },
                ))
                .id();

            app.update();

            let logistics = app.world().get::<SystemLogistics>(entity).expect("Component should exist");
            assert_eq!(
                logistics.capacity, 100,
                "Capacity should match ratio (1.0 * 100)"
            );

            let events = app.world().resource::<Events<LogisticsStrainedEvent>>();
            assert!(
                events.get_cursor().read(events).next().is_none(),
                "No event should be sent since utilized (10) < capacity (100)"
            );
        }

        #[test]
        fn test_update_sector_defenses_zero_capacity() {
            let mut app = App::new();
            app.add_systems(Update, update_sector_defenses);
            app.add_event::<DefenseWeakenedEvent>();

            let entity = app
                .world_mut()
                .spawn((
                    SectorDefense { power: 1000 },
                    SystemLogistics {
                        capacity: 0,
                        utilized: 10,
                    },
                ))
                .id();

            app.update();

            let defense = app.world().get::<SectorDefense>(entity).expect("Component should exist");
            assert_eq!(
                defense.power, 750,
                "Power should drop by 25% when capacity is 0"
            );

            let events = app.world().resource::<Events<DefenseWeakenedEvent>>();
            assert!(
                events.get_cursor().read(events).next().is_some(),
                "Event should be sent"
            );
        }

        #[test]
        fn test_update_sector_defenses_under_strain() {
            let mut app = App::new();
            app.add_systems(Update, update_sector_defenses);
            app.add_event::<DefenseWeakenedEvent>();

            let entity = app
                .world_mut()
                .spawn((
                    SectorDefense { power: 1000 },
                    SystemLogistics {
                        capacity: 100,
                        utilized: 50,
                    },
                ))
                .id();

            app.update();

            let defense = app.world().get::<SectorDefense>(entity).expect("Component should exist");
            assert_eq!(
                defense.power, 1000,
                "Power should not drop when strain < 1.0"
            );

            let events = app.world().resource::<Events<DefenseWeakenedEvent>>();
            assert!(
                events.get_cursor().read(events).next().is_none(),
                "No event should be sent"
            );
        }

        #[test]
        fn test_calculate_invasion_threat_strong_defense() {
            let mut app = App::new();
            app.add_systems(Update, calculate_invasion_threat);

            let entity = app
                .world_mut()
                .spawn((SectorDefense { power: 1000 }, InvasionThreat { level: 0.0 }))
                .id();

            app.update();

            let threat = app.world().get::<InvasionThreat>(entity).expect("Component should exist");
            assert_eq!(
                threat.level, 0.0,
                "Threat should not increase with strong defense"
            );
        }
    }
}
pub use failure::*;

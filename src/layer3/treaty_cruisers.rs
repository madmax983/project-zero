use crate::layer2::ship::Ship;
use bevy::prelude::*;

#[derive(Component)]
pub struct Hull {
    pub size: u32,
}

#[derive(Clone, PartialEq, Eq)]
pub enum ModuleType {
    MilitaryWeapon,
    CivilianMiningLaser,
    Engine,
}

#[derive(Clone)]
pub struct Module {
    pub m_type: ModuleType,
    pub power: u32,
}

#[derive(Component)]
pub struct ShipModules(pub Vec<Module>);

#[derive(Clone, PartialEq, Eq)]
pub enum Restriction {
    MaxHull(u32),
    BannedModuleType(ModuleType),
}

#[derive(Resource, Default)]
pub struct ActiveTreaties {
    pub treaties: Vec<Treaty>,
}

#[derive(Clone)]
pub struct Treaty {
    pub restrictions: Vec<Restriction>,
}

#[derive(Component, Debug, PartialEq, Eq)]
pub enum ComplianceStatus {
    Unknown,
    Compliant,
    Violating,
}

#[derive(Event)]
pub struct InspectionEvent {
    pub target: Entity,
}

type ShipQuery<'w, 's> = Query<
    'w,
    's,
    (
        Option<&'static Hull>,
        Option<&'static ShipModules>,
        &'static mut ComplianceStatus,
    ),
    With<Ship>,
>;

pub fn compliance_check_system(
    active_treaties: Res<ActiveTreaties>,
    mut events: EventReader<InspectionEvent>,
    mut ships: ShipQuery<'_, '_>,
) {
    for event in events.read() {
        if let Ok((hull_opt, modules_opt, mut status)) = ships.get_mut(event.target) {
            let mut is_violating = false;

            for treaty in &active_treaties.treaties {
                for restriction in &treaty.restrictions {
                    match restriction {
                        Restriction::MaxHull(max_size) => {
                            if let Some(hull) = hull_opt {
                                if hull.size > *max_size {
                                    is_violating = true;
                                }
                            }
                        }
                        Restriction::BannedModuleType(banned_type) => {
                            if let Some(modules) = modules_opt {
                                for module in &modules.0 {
                                    if module.m_type == *banned_type {
                                        is_violating = true;
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if is_violating {
                *status = ComplianceStatus::Violating;
            } else {
                *status = ComplianceStatus::Compliant;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer2::ship::Ship;
    // We will define these in the GREEN phase but we need them in tests
    // so we mock/import them here (actually they'll be in super)

    #[test]
    fn test_ship_complies_with_hull_limit() {
        let mut app = App::new();
        app.add_event::<InspectionEvent>();
        app.insert_resource(ActiveTreaties {
            treaties: vec![Treaty {
                restrictions: vec![Restriction::MaxHull(500)],
            }],
        });
        app.add_systems(Update, compliance_check_system);

        let compliant_ship = app
            .world_mut()
            .spawn((
                Ship {
                    ship_type: crate::layer2::ship::ShipType::Scout,
                    health: 10.0,
                    max_health: 10.0,
                },
                Hull { size: 499 },
                ComplianceStatus::Unknown,
            ))
            .id();

        let violating_ship = app
            .world_mut()
            .spawn((
                Ship {
                    ship_type: crate::layer2::ship::ShipType::Scout,
                    health: 10.0,
                    max_health: 10.0,
                },
                Hull { size: 501 },
                ComplianceStatus::Unknown,
            ))
            .id();

        app.world_mut().send_event(InspectionEvent {
            target: compliant_ship,
        });
        app.world_mut().send_event(InspectionEvent {
            target: violating_ship,
        });
        app.update(); // Run compliance

        assert_eq!(
            app.world().get::<ComplianceStatus>(compliant_ship).unwrap(),
            &ComplianceStatus::Compliant
        );
        assert_eq!(
            app.world().get::<ComplianceStatus>(violating_ship).unwrap(),
            &ComplianceStatus::Violating
        );
    }

    #[test]
    fn test_ship_evades_weapon_restriction_via_classification() {
        let mut app = App::new();
        app.add_event::<InspectionEvent>();
        app.insert_resource(ActiveTreaties {
            treaties: vec![Treaty {
                restrictions: vec![Restriction::BannedModuleType(ModuleType::MilitaryWeapon)],
            }],
        });
        app.add_systems(Update, compliance_check_system);

        let evasive_ship = app
            .world_mut()
            .spawn((
                Ship {
                    ship_type: crate::layer2::ship::ShipType::Scout,
                    health: 10.0,
                    max_health: 10.0,
                },
                ShipModules(vec![
                    Module {
                        m_type: ModuleType::CivilianMiningLaser,
                        power: 100,
                    }, // Legally civilian, practically weapon
                ]),
                ComplianceStatus::Unknown,
            ))
            .id();

        app.world_mut().send_event(InspectionEvent {
            target: evasive_ship,
        });
        app.update();

        // The ship technically passes inspection
        assert_eq!(
            app.world().get::<ComplianceStatus>(evasive_ship).unwrap(),
            &ComplianceStatus::Compliant
        );
    }

    #[test]
    fn test_ship_violates_weapon_restriction() {
        let mut app = App::new();
        app.add_event::<InspectionEvent>();
        app.insert_resource(ActiveTreaties {
            treaties: vec![Treaty {
                restrictions: vec![Restriction::BannedModuleType(ModuleType::MilitaryWeapon)],
            }],
        });
        app.add_systems(Update, compliance_check_system);

        let violating_ship = app
            .world_mut()
            .spawn((
                Ship {
                    ship_type: crate::layer2::ship::ShipType::Scout,
                    health: 10.0,
                    max_health: 10.0,
                },
                ShipModules(vec![Module {
                    m_type: ModuleType::MilitaryWeapon,
                    power: 100,
                }]),
                ComplianceStatus::Unknown,
            ))
            .id();

        app.world_mut().send_event(InspectionEvent {
            target: violating_ship,
        });
        app.update();

        // The ship fails inspection
        assert_eq!(
            app.world().get::<ComplianceStatus>(violating_ship).unwrap(),
            &ComplianceStatus::Violating
        );
    }
}

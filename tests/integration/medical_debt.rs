use scale::layer1::actions::{AssignedTo, AssignmentType};
use scale::layer1::building::{Building, BuildingType};
use scale::layer1::health::Health;
use scale::layer1::integration::medical_debt_bridge_system;
use scale::layer1::medical::{Hospital, PatientTreated, healing_system};
use scale::layer1::pop::{Job, Pop};
use scale::layer1::social::SocialDebt;
use bevy_ecs::prelude::*;

#[test]
fn test_medical_treatment_creates_debt() {
    let mut world = World::new();
    let mut schedule = Schedule::default();

    // Register events
    world.init_resource::<Events<PatientTreated>>();
    world.init_resource::<Events<scale::layer1::social::FavorChange>>();

    // Register systems
    schedule.add_systems((
        healing_system,
        medical_debt_bridge_system,
        scale::layer1::social::accrue_debt_system,
    ).chain());

    // 1. Spawn Hospital
    let hospital = world.spawn((
        Building { building_type: BuildingType::Hospital },
        Hospital { healing_rate: 10.0, ..Default::default() },
        scale::layer1::map::GridPosition { x: 0, y: 0 },
    )).id();

    // 2. Spawn Doctor
    let doctor = world.spawn((
        Pop,
        Job {
            workplace: hospital,
            job_type: AssignmentType::Doctor,
        },
    )).id();

    // 3. Spawn Patient (Injured)
    let patient = world.spawn((
        Pop,
        Health { current: 50.0, max: 100.0 },
        AssignedTo {
            entity: hospital,
            assignment_type: AssignmentType::Patient,
        },
        SocialDebt::default(),
    )).id();

    // 4. Run Schedule
    schedule.run(&mut world);

    // 5. Assert Health Improved
    let health = world.get::<Health>(patient).unwrap();
    assert!(health.current > 50.0, "Patient should be healed");

    // 6. Assert Debt Created
    let debt = world.get::<SocialDebt>(patient).unwrap();
    let owed = debt.get_debt(doctor);

    assert!(owed > 0.0, "Patient should owe doctor debt");
    // Ideally debt equals healing amount, but bridge might scale it.
    // For now, assume 1:1 or at least positive.
}

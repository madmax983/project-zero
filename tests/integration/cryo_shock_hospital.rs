use bevy::prelude::*;
use scale::layer1::actions::{AssignedTo, AssignmentType};
use scale::layer1::cryo_shock::CryoShock;
use scale::layer1::health::Health;
use scale::layer1::medical::{healing_system, Hospital};

#[test]
fn test_cryo_shock_treated_at_hospital() {
    let mut world = World::new();

    // Spawn Hospital
    let hospital_ent = world.spawn(Hospital::default()).id();

    // Spawn patient with CryoShock
    let patient_ent = world
        .spawn((
            Health::default(),
            CryoShock {
                duration_ticks: 100,
                severity: 0.5,
            },
            AssignedTo {
                entity: hospital_ent,
                assignment_type: AssignmentType::Patient,
            },
        ))
        .id();

    // Run system once
    let mut schedule = Schedule::default();
    schedule.add_systems(healing_system);
    schedule.run(&mut world);

    // Duration should have decreased by 50
    let shock = world.get::<CryoShock>(patient_ent).unwrap();
    assert_eq!(
        shock.duration_ticks, 50,
        "CryoShock should be reduced by 50 per tick at hospital"
    );

    // Run again, should remove CryoShock
    schedule.run(&mut world);
    assert!(
        world.get::<CryoShock>(patient_ent).is_none(),
        "CryoShock should be removed when duration reaches 0"
    );
}

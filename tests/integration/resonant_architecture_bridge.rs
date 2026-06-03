use bevy::prelude::*;
use scale::layer1::architecture::resonant_architecture::PopResonanceTraits;
use scale::layer1::combat::{execute_attack, Weapon, AttackProperties};
use scale::layer1::entities::pop::Pop;
use scale::layer1::health::Health;
use scale::layer1::items::Equipment;
use scale::layer1::needs::Needs;
use scale::layer1::psychology::stress::{check_stress_breakdown_system, StressTracker};
use scale::layer1::tech::{process_research_system, Library};
use scale::layer1::actions::{AssignedTo, AssignmentType};
use scale::layer1::economy::resources::ColonyResources;

#[test]
fn test_resonance_bridge_stress() {
    let mut app = App::new();
    app.add_systems(Update, check_stress_breakdown_system);

    // Give pop a trait that doubles stress
    let pop = app
        .world_mut()
        .spawn((
            Pop,
            Needs {
                hunger: 0.0,
                rest: 0.0,
                leisure: 0.0, // Low morale
                hygiene: 0.0,
            },
            StressTracker {
                accumulated_stress: 0.0,
            },
            PopResonanceTraits {
                stress_gain_mult: 2.0,
                ..Default::default()
            },
        ))
        .id();

    app.update();

    let tracker = app.world().get::<StressTracker>(pop).unwrap();
    // Normally +1.0, with mult 2.0 -> +2.0
    assert_eq!(tracker.accumulated_stress, 2.0, "Stress should be doubled by resonant architecture");
}

#[test]
fn test_resonance_bridge_research() {
    let mut app = App::new();
    app.init_resource::<ColonyResources>();
    app.add_systems(Update, process_research_system);

    let library = app.world_mut().spawn(Library).id();

    let _pop = app
        .world_mut()
        .spawn((
            Pop,
            AssignedTo {
                entity: library,
                assignment_type: AssignmentType::LibraryWorker,
            },
            PopResonanceTraits {
                research_speed_mult: 2.0,
                ..Default::default()
            },
        ))
        .id();

    app.update();

    let resources = app.world().resource::<ColonyResources>();
    // Normally 1 worker = 0.01 knowledge. With mult 2.0 -> 0.02
    assert!((resources.knowledge - 0.02).abs() < f32::EPSILON, "Research should be doubled by resonant architecture");
}

#[test]
fn test_resonance_bridge_combat() {
    let mut app = App::new();

    let weapon = app.world_mut().spawn(Weapon {
        properties: AttackProperties {
            damage: 10.0,
            cooldown: 0,
            range: 1.0,
            accuracy: 1.0,
        },
    }).id();

    let attacker = app.world_mut().spawn((
        Pop,
        Equipment {
            weapon: Some(weapon),
            ..Default::default()
        },
        PopResonanceTraits {
            aggression_mult: 1.5,
            ..Default::default()
        },
    )).id();

    let target = app.world_mut().spawn((
        Health {
            current: 100.0,
            max: 100.0,
            has_rust_lung: false,
        },
        scale::layer1::map::GridPosition { x: 0, y: 0 },
    )).id();

    execute_attack(app.world_mut(), attacker, target);

    let target_health = app.world().get::<Health>(target).unwrap().current;
    // Base damage 10, mult 1.5 = 15 damage. 100 - 15 = 85.
    // Crits could occur, so damage is AT LEAST 15.
    assert!(target_health <= 85.0, "Target should have taken at least 15 damage due to aggression mult. Health: {}", target_health);
}

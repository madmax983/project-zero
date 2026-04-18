import re

def replace_in_file(filepath, pattern, replacement):
    with open(filepath, 'r') as f:
        content = f.read()
    new_content = re.sub(pattern, replacement, content, count=1, flags=re.MULTILINE|re.DOTALL)
    if content != new_content:
        with open(filepath, 'w') as f:
            f.write(new_content)
        print(f"Updated {filepath}")

# 1. src/layer1/administration/bureaucracy_of_sleep.rs
replace_in_file(
    "src/layer1/administration/bureaucracy_of_sleep.rs",
    r"/// Assigns sleep permits to Pops based on their job\.\n#\[allow\(clippy::type_complexity\)\]\npub fn assign_sleep_permits_system\(",
    r"""/// Assigns the proper sleep permit and initializes a fatigue tracker based on the Pop's assignment.
///
/// Ensures critical workers (like Administrators and Surgeons) receive sufficient rest (`PermitTier::Gold`)
/// to prevent life-threatening errors, while general laborers (like Miners) are allocated fewer hours
/// to maintain peak colony throughput.
///
/// # Examples
/// ```
/// use scale::layer1::administration::bureaucracy_of_sleep::{assign_sleep_permits_system, SleepPermit, PermitTier};
/// use scale::layer1::pop::{Job, Pop};
/// use scale::layer1::utility_types::AssignmentType;
/// use bevy_ecs::prelude::*;
///
/// let mut world = World::new();
/// let pop_entity = world.spawn((Pop, Job { job_type: AssignmentType::Administrator, ..Default::default() })).id();
///
/// let mut schedule = Schedule::default();
/// schedule.add_systems(assign_sleep_permits_system);
/// schedule.run(&mut world);
///
/// let permit = world.get::<SleepPermit>(pop_entity).unwrap();
/// assert_eq!(permit.tier, PermitTier::Gold);
/// ```
#[allow(clippy::type_complexity)]
pub fn assign_sleep_permits_system("""
)

replace_in_file(
    "src/layer1/administration/bureaucracy_of_sleep.rs",
    r"/// Processes sleep deprivation, penalizing Pops who lack adequate sleep permits\.\npub fn process_sleep_deprivation_system\(",
    r"""/// Evaluates a Pop's accrued fatigue against their assigned `SleepPermit` threshold.
///
/// Once a Pop exceeds their allowed waking hours, this system forces them into an involuntary
/// sleep state or applies severe productivity penalties until their debt is paid.
///
/// # Examples
/// ```
/// use scale::layer1::administration::bureaucracy_of_sleep::{process_sleep_deprivation_system, FatigueTracker, SleepPermit, PermitTier};
/// use scale::layer1::stress::StressTracker;
/// use scale::layer1::pop::Pop;
/// use bevy_ecs::prelude::*;
///
/// let mut world = World::new();
/// let pop_entity = world.spawn((
///     Pop,
///     FatigueTracker { current: 15.0 },
///     SleepPermit { tier: PermitTier::Bronze, allotted_hours: 4.0 },
///     StressTracker { current: 0.0, ..Default::default() }
/// )).id();
///
/// let mut schedule = Schedule::default();
/// schedule.add_systems(process_sleep_deprivation_system);
/// schedule.run(&mut world);
///
/// let stress = world.get::<StressTracker>(pop_entity).unwrap();
/// assert!(stress.current > 0.0, "Pop should accrue stress from severe fatigue.");
/// ```
pub fn process_sleep_deprivation_system("""
)



replace_in_file(
    "src/layer1/economy/beacon.rs",
    r"/// Processes the effects of an active colony beacon\.\n///\n/// When the beacon is active, it significantly increases the chances of:\n/// - Trade ships arriving\n/// - Migrants arriving \(with a high chance of criminals/low-skill\)\n/// - Pirate raids\npub fn process_colony_beacon_system\(",
    r"""/// Evaluates the probabilities of external events triggered by the `ColonyBeacon`.
///
/// An active beacon serves as a lighthouse in the dark sector, significantly increasing the influx
/// of independent trade ships and desperate migrants. However, this same visibility attracts pirate
/// raids. This system uses randomized rolls each tick to determine if a specific event is dispatched.
///
/// # Examples
/// ```
/// use scale::layer1::economy::beacon::{ColonyBeacon, process_colony_beacon_system};
/// use scale::layer2::trade::blockade::TradeShipArrivalEvent;
/// use scale::layer1::economy::remittances::MigrantArrivalEvent;
/// use scale::layer1::void_weed::PirateRaidEvent;
/// use bevy_ecs::prelude::*;
///
/// let mut world = World::new();
/// world.insert_resource(ColonyBeacon { is_active: true });
/// world.insert_resource(Events::<TradeShipArrivalEvent>::default());
/// world.insert_resource(Events::<MigrantArrivalEvent>::default());
/// world.insert_resource(Events::<PirateRaidEvent>::default());
///
/// let mut schedule = Schedule::default();
/// schedule.add_systems(process_colony_beacon_system);
/// schedule.run(&mut world);
/// // External events may or may not be spawned based on RNG.
/// ```
pub fn process_colony_beacon_system("""
)

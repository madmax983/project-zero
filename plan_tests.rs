use crate::layer1::memory::Grudge;
use crate::layer1::pop::PopBorn;
use crate::layer3::diplomacy::{ActiveNegotiation, DiplomaticStance};

#[test]
fn test_grudge_memory_inheritance() {
    let mut app = App::new();
    // Setup Parent Pop with Grudge(Faction::Pirates)
    // act: spawn child Pop
    // assert: child Pop inherits Grudge(Faction::Pirates)
}

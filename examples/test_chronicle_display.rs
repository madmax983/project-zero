#[allow(unused_imports)]
use scale::prelude::*;
use scale::layer1::core::chronicle::{Chronicle, EventImportance};

fn main() {
    let mut chronicle = Chronicle::default();

    chronicle.add_event(0, "Colony Founded".to_string(), EventImportance::Legendary);
    chronicle.add_event(100, "First Winter".to_string(), EventImportance::Major);
    chronicle.add_event(200, "Built a house".to_string(), EventImportance::Standard);
    chronicle.add_event(250, "Bob ate a berry".to_string(), EventImportance::Minor);

    println!("{}", chronicle);
}

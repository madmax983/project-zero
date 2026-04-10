pub mod transit;
pub use transit::*;
pub mod building;
pub mod construction;
pub mod housing;
pub mod parasitic_architecture;
pub mod room_quality;
pub mod ruins;
pub mod spontaneous_architecture;
pub mod structure;
pub mod window;

pub use building::*;
pub use construction::*;
pub use housing::*;
pub use parasitic_architecture::*;
pub use room_quality::*;
pub use ruins::*;
pub use spontaneous_architecture::*;
pub use structure::*;
pub use window::*;

#[cfg(test)]
mod building_gate_test {
    use super::*;
    use crate::layer1::*;

    include!("building_gate_test.rs");
}

#[cfg(test)]
mod structure_fragile_tests {

    include!("structure_fragile_tests.rs");
}

#[cfg(test)]
mod structure_jury_rig_tests {

    include!("structure_jury_rig_tests.rs");
}

#[cfg(test)]
mod structure_maintenance_tests {

    include!("structure_maintenance_tests.rs");
}

#[cfg(test)]
mod work_building_tests {

    include!("work_building_tests.rs");
}

#[cfg(test)]
mod shift_integration_tests {

    include!("shift_integration_tests.rs");
}

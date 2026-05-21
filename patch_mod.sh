sed -i 's/pub mod hoarder_sleepwalking;/pub mod hoarder_sleepwalking;\n#[cfg(feature = "nova")]\npub mod radioactive_rats;/g' src/experimental/mod.rs

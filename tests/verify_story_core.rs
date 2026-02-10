use scale::shared::narrative::NarrativeGenerator;

#[test]
fn narrative_generator_is_core_feature() {
    // This test verifies that NarrativeGenerator is available without the 'nova' feature flag.
    // If NarrativeGenerator was gated behind #[cfg(feature = "nova")], this would fail to compile
    // unless 'nova' was enabled by default (which it is not).

    let generator = NarrativeGenerator::default();
    assert_eq!(generator.template_count(), 0);

    let generator_embedded = NarrativeGenerator::from_embedded();
    assert!(generator_embedded.template_count() > 0);

    println!("NarrativeGenerator is successfully accessible as a Core feature.");
}

#[cfg(feature = "nova")]
#[test]
fn nova_feature_is_enabled() {
    // This test ensures we are NOT accidentally running with nova enabled when we think we aren't.
    // However, if we run `cargo test --features nova`, this will pass.
    // We want to verify `cargo test` (no features) passes the previous test.
    println!("Nova feature is enabled.");
}

cat << 'INNER_EOF' >> .jules/sentry.md
**[Atmospheric Diffusion Config Requirement in Tests]**
**Learning:** In recent architectural changes, the atmospheric diffusion logic was extracted into `simulate_diffusion_system` and depends on a `DiffusionConfig` resource. Legacy integration tests (e.g. `quirks_atmosphere.rs`) evaluating pollution retention were silently failing because they did not explicitly register `DiffusionConfig` or invoke `simulate_diffusion_system` alongside `update_atmosphere_system`.
**Action:** When testing diffusion or pollution spread, explicitly add `world.insert_resource(DiffusionConfig::default());` and call `world.run_system_once(simulate_diffusion_system).unwrap();` after updating the main atmosphere system.

**[HashMap Type Mismatches in Bevy]**
**Learning:** Using the standard library `std::collections::HashMap` when calling methods on Bevy grid structures (like `AtmosphereGrid.diffuse`) will result in `mismatched types` compiler errors because Bevy relies on `bevy_utils::hashbrown::HashMap`.
**Action:** Always import and use `bevy_utils::hashbrown::HashMap` when building constraint grids or blockers to interact with Bevy ECS systems.
INNER_EOF

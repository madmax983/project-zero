# Sentry's Journal

**[Testing Headless CLI Commands]**
**Learning:** `headless.rs` processes commands interactively for development. Testing these CLI wrappers (like `handle_scan_command`, `handle_bio_command`, `handle_find_command`, `handle_terrain_command`, `handle_map_command`) simply requires ensuring they handle arguments gracefully without panicking, even if `setup_minimal_world` doesn't populate all edge-case resources. When mocking global inputs (like empty command arguments), verify it falls back to defaults or prints an error to the dashboard panel instead of `unwrap()`panicking.
**Action:** When working on CLI command parsers or generic debugging endpoints, focus on testing bound constraints, missing parameters, and empty worlds to guarantee the developer tools don't crash when misconfigured.

**[Memetic Broadcasts Coverage Gaps]**
**Learning:** Discovered untested branches in `evaluate_scrawl_memetic_sigil` specifically when a Pop Eval Data states it's *not* a carrier, or if the `walls` buffer is entirely empty.
**Action:** The Utility AI buffer gather phase is naturally decoupled, making edge-case tests very straightforward to write. Manually construct `PopEvalData` variations and inject fake/empty `ScorableCandidate` vectors to fully cover evaluation logic.

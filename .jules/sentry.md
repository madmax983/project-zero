# Sentry's Journal

**2024-05-22 - [Flaky Test]**
**Insight:** `layer1::pop::tests::test_spawn_initial_pops_retries` is flaky. It panicked with assertion failed: `left == right` (4 != 5).
**Strategy:** Needs investigation into random seed handling or retry logic.

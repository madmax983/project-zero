**[Flaky Tests]
**Learning:** `layer1::mining_tests::tests::test_mine_rock_spawns_anomaly_probabilistically` is a flaky test. It relies on a 5% RNG chance over 100 iterations, resulting in a ~0.6% chance of arbitrary CI failure.
**Action:** Always verify test iterations when dealing with RNG-based spawning to ensure <0.01% chance of flake.
**[Flaky Tests]**
**Learning:** `layer1::mining_tests::tests::test_mine_rock_spawns_anomaly_probabilistically` is a flaky test. It relies on a 5% RNG chance over 100 iterations, resulting in a ~0.6% chance of arbitrary CI failure.
**Action:** Always verify test iterations when dealing with RNG-based spawning to ensure <0.01% chance of flake.

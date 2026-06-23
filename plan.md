1.  **Analyze the task**:
    - The task is `INT-1305` Integration: The Cassandra Syndrome -> Chronicle.
    - We need to bridge the output of the Cassandra Syndrome (DoomsdayWarningEvent and DisasterOccurredEvent leading to Cult Formation) to the `AddChronicleEvent` in `src/layer1/core/integration.rs`.
    - We must update `design/COMPLETED.md` with the newly integrated feature (using the `INT-1305` ID).
    - We must add it to `design/SEAM_MAP.md` tracking the connection.
    - We must write integration tests in `tests/integration/cassandra_syndrome_chronicle.rs` verifying the bridge systems work correctly (red phase) and update `tests/integration.rs`.

2.  **Steps**:
    - Add `cassandra_syndrome_chronicle_bridge` to `src/layer1/core/integration.rs`. It will:
        - Listen for `DoomsdayWarningEvent` and write an `AddChronicleEvent` (Standard/Major).
        - Listen for `DisasterOccurredEvent` (or maybe listen for `CultLeader` component addition to trigger the cult formation chronicle? Let's check `tests/integration/cassandra_syndrome_chronicle.rs` for the best approach, but listening for the events is best). Wait, `CultLeader` is added upon `DisasterOccurredEvent` in `validate_prophecy`.
    - Create `tests/integration/cassandra_syndrome_chronicle.rs` and verify the `AddChronicleEvent` is triggered when `DoomsdayWarningEvent` and `DisasterOccurredEvent` are fired. Wait, the prompt says "Cult Formation events". We should query for `Added<CultLeader>` to generate the chronicle event when a cult forms to ensure schedule independence.
    - Let's make sure we test `Added<CultLeader>` for Cult Formation, as it's more accurate than `DisasterOccurredEvent` alone since the cult only forms if the prophecy was active.
    - Let's create `tests/integration/cassandra_syndrome_chronicle.rs`.
    - Include `cassandra_syndrome_chronicle_bridge` in `src/layer1/systems/execution.rs` (or where the other integration bridges are added). Wait, looking at `src/layer1/systems/observation.rs` or `src/simulation.rs`, integration bridges are registered in `src/layer1/systems/execution.rs` or `src/layer1/systems/observation.rs`. We'll just add it to `src/layer1/systems/execution.rs` near the other chronicle bridges.
    - Actually, `simulation.rs` registers `crate::layer1::core::integration::*` systems. Let's see how `simulation.rs` handles it.
    - Update `design/COMPLETED.md` and `design/SEAM_MAP.md`.
    - Run pre-commit checks.

## [Reduction]
**Bloat:** `AssignmentType` enum contained 12 unused variants (Miner, Hauler, Builder, etc.) that were never assigned to any Pop.
**Cut:** Deleted the unused variants and their associated match arms in `biography.rs`, `economy.rs`, `arrival.rs`, `movement.rs`, and `social_stratification.rs`.
**Saved:** ~50 lines of code / Cognitive load of wondering "How does a pop become a Chef?".

## [Fix]
**Bloat:** `ActionType::Farm` was missing from `process_arrival`, meaning farmers never actually got their "Job" assigned.
**Cut:** Added mapping `ActionType::Farm` -> `AssignmentType::FarmWorker`.
**Saved:** Bugs where farmers didn't have the correct social class or wage.

## [Reduction]
**Bloat:** `AssignmentType` enum contained 12 unused variants (Miner, Hauler, Builder, etc.) that were never assigned to any Pop.
**Cut:** Deleted the unused variants and their associated match arms in `biography.rs`, `economy.rs`, `arrival.rs`, `movement.rs`, and `social_stratification.rs`.
**Saved:** ~50 lines of code / Cognitive load of wondering "How does a pop become a Chef?".

## [Fix]
**Bloat:** `ActionType::Farm` was missing from `process_arrival`, meaning farmers never actually got their "Job" assigned.
**Cut:** Added mapping `ActionType::Farm` -> `AssignmentType::FarmWorker`.
**Saved:** Bugs where farmers didn't have the correct social class or wage.

## [Cleanup]
**Bloat:** `CustomsOfficer` struct and associated query in `vetting_work_system` were unused/speculative (green phase simulation).
**Cut:** Deleted the struct and the query argument.
**Saved:** 1 struct definition, 1 ECS query, 5 lines of test setup.

## [Simplify]
**Bloat:** Using `Pop::default()` for a unit struct, and `..Default::default()` when all fields are explicitly set.
**Cut:** Replaced with `Pop` and removed redundant default updates.
**Saved:** Verbosity and cognitive load regarding "what else is in this struct?".

## [Safety]
**Bloat:** Strict float equality checks (`assert_eq!(x, 0.0)`) in tests.
**Cut:** Replaced with epsilon checks (`x.abs() < f32::EPSILON`).
**Saved:** Potential flakiness and clippy warnings.

## [Simplify]
**Bloat:** `ColonyResources` tracked `wheat`, `potato`, and `rice` as separate float fields, even though 90% of the game treated them as generic `Food`. Logic in `produce_food_system` and `consume_food_system` had complex branching to sync them.
**Cut:** Removed the fields. Farms now output directly to `resources.food`. Consumption draws from `resources.food`. `DietaryHistory` infers flavor from active farms rather than silo contents.
**Saved:** 3 resource fields, ~30 lines of redundant branching logic, and inconsistent "some crops are real, some are flavor" design.

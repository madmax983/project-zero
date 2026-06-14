1. **RED Phase (Failing Tests)**:
    - Create a new module `src/layer2/auction.rs`.
    - Create a `cat << 'EOF' > src/layer2/auction.rs` command containing the RED Phase failing tests specified in the spec file, adjusted to use the confirmed module paths: `crate::layer1::economy::{ColonyResources, ResourceType}` and `crate::layer1::void_weed::MerchantArrivalEvent`. In the tests, the `test_bidding_subtracts_resources` needs to use `stone` on `ColonyResources` (which is confirmed in the struct definition) and `ResourceType::Stone`.
    - Run `cargo test --lib layer2::auction` to verify the tests fail.
2. **GREEN Phase (Minimal Implementation)**:
    - Add `Enigmatic` to the `MerchantType` enum in `src/layer1/void_weed.rs` using `sed`.
    - Update `src/layer2/auction.rs` using a python script to insert the implementation logic:
      - `BlindAuctionTriggeredEvent` (struct with no fields).
      - `PlaceBidEvent` (struct with `amount: f32` and `resource_type: ResourceType`).
      - `VaultOutcome` (enum with `TechBoost`, `CatastrophicAnomaly`).
      - `VaultOpenedEvent` (struct with `outcome: VaultOutcome`).
      - `TemporalAnomalyEvent` (struct with no fields).
      - Implement `check_for_blind_auction_trigger` to read `MerchantArrivalEvent` and send `BlindAuctionTriggeredEvent` if `merchant_type == MerchantType::Enigmatic`.
      - Implement `handle_blind_auction_bids` to read `PlaceBidEvent`, check if `ColonyResources.stone >= bid.amount`, and deduct `amount` from `stone`.
      - Implement `process_vault_outcome` to read `VaultOpenedEvent`, check the `outcome`, and if it's `CatastrophicAnomaly`, spawn a `TemporalAnomalyEvent`.
    - Verify modification of `void_weed.rs` and `auction.rs` using `cat` / `git diff`.
    - Register `auction` as a module in `src/layer2/mod.rs` by echoing `pub mod auction;` and `pub use auction::*;` to the file using a python script. Verify using `cat src/layer2/mod.rs`.
    - Run `cargo test --lib layer2::auction` to verify tests pass.
3. **REFACTOR Phase (Quality & Design)**:
    - Add an empty refactor commit or minor constant extraction for the accepted resource type to comply with the TDD workflow, ensuring tests remain passing.
4. **Verification**:
    - Run `cargo fmt`, `cargo check`, `cargo clippy -- -D warnings`, `cargo test`, and `cargo llvm-cov --lib` to ensure all rules and coverage minimums are met.
5. **Pre-commit**:
    - Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.
6. **Finalize**:
    - Use `default_api:submit` to commit with the message "feat(layer2): implement blind auction" and push.

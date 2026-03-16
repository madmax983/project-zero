# 481: Subterranean Mycelial Network

## 1. Overview
Growing a living, pulsating biological transit system beneath your colony that is highly efficient but completely alien. You discover a vast fungal network underground. By feeding it specific nutrients, you can "train" it to act as a hyper-fast, frictionless conveyor belt for raw resources, completely replacing mechanical haulers and pneumatic tubes. The network works perfectly for years, moving thousands of tons of ore. But when a disease breaks out in your agricultural sector, the mycelium inadvertently transports the pathogen directly into every connected stockpile and residential zone simultaneously, turning a localized outbreak into a colony-wide pandemic in minutes. This creates unparalleled, free logistical throughput vs. a terrifyingly efficient vector for disease and invasive species.

## 2. Dependencies
- `111` Conveyor Logistics (Implemented)
- `457` Subterranean Mycelial Network (Idea/Backlog)

## 3. RED Phase: Tests First
```rust
#[test]
fn test_mycelial_network_transports_resources() {
    let mut app = App::new();
    // Arrange: Setup connected Mycelial nodes and resource at input
    // Act: Process mycelial transport tick
    // Assert: Resource arrives at destination stockpile instantaneously or very fast
}

#[test]
fn test_mycelial_network_upkeep() {
    let mut app = App::new();
    // Arrange: Setup active Mycelial network
    // Act: Process metabolism/upkeep tick
    // Assert: Nutrient resources are consumed, or network degrades
}

#[test]
fn test_mycelial_network_spreads_disease() {
    let mut app = App::new();
    // Arrange: Setup connected network, introduce infected item/pop at one node
    // Act: Trigger pathogen spread logic
    // Assert: All nodes in the network are marked as contaminated and nearby pops become infected
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// Minimal systems for MycelialNode linking, resource routing over network, upkeep/nutrient consumption, and pathogen propagation.
```

## 5. REFACTOR Phase: Quality & Design
- Share routing logic with `111 Conveyor Logistics` if applicable, but handle instant/frictionless transport.
- Ensure the pathogen propagation utilizes a generic "Contagion" event or trait that can hook into existing medical and disease systems.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Mycelial network transports items efficiently and correctly propagates infections to all connected nodes.

## 7. Technical Guidance
- A `MycelialNode` component can be placed on stockpiles or specific buildings.
- A central `MycelialNetwork` resource can maintain a graph of connected nodes to simplify pathfinding and instantaneous transport.
- Pathogen spread should broadcast a `ContaminationEvent` to all connected nodes in the graph immediately upon exposure.

## 8. Questions
*Builder: add questions here if spec is unclear. Architect will address.*

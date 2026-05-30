# Builder Learnings

- Specs can be implemented in `src/layer3/bureaucracy.rs` but if relying on events like `DiscoveryEvent`, ensure you emit them instead of just leaving dummy implementations. Always fully implement RED-GREEN-REFACTOR for the specific problem without hallucinating variables (e.g. `credits` when only `food` is available).

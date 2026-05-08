1.  **Draft a Specification for "Sentient Trade Routes"**:
    *   Find the next available spec number (`1255`).
    *   Create a file `specs/1255-sentient-trade-routes.md` following the TDD structure:
        *   **Overview**: Describe the mechanic where trade routes accumulate algorithmic complexity and demand tolls (Layer 1 resources like `Art` or `RareData`) from Layer 1 colonies to prioritize shipments.
        *   **Dependencies**: Mention `src/layer2/trade.rs`, `src/layer1/resources.rs`.
        *   **RED Phase**: Write failing tests for algorithmic complexity accumulating on a route, an AI demanding a toll, and a shipment being delayed/prioritized based on toll payment.
        *   **GREEN Phase**: Minimal implementation (e.g., adding `Complexity` to `TradeRoute`, an event for `SentientTollDemand`, and adjusting shipment times based on toll fulfillment).
        *   **REFACTOR Phase**: Identify cleanup opportunities.
        *   **Acceptance Criteria**: Mention required test passage, code quality checks.
        *   **Technical Guidance**: Explain where to register the event.
    *   *Note: Using a tool command for generating the file content is essential.*

2.  **Add the Specification to the Backlog (`design/BACKLOG.md`)**:
    *   Append `- [ ] \`1255\` Sentient Trade Routes — \`specs/1255-sentient-trade-routes.md\`` to `design/BACKLOG.md`.

3.  **Update `design/IDEAS.md`**:
    *   Mark `## Sentient Trade Routes` with `[SPECCED]`.

4.  **Pre-commit Steps**:
    *   Ensure proper testing, verification, review, and reflection are done. Call `pre_commit_instructions` tool.

5.  **Submit Changes**:
    *   Submit the commit with `spec(layer2): add sentient trade routes specification (TDD)`.

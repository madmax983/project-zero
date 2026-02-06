# 5. Adopt Emergent Utility AI

Date: 2024-05-22

## Status

Accepted

## Context

Early iterations of the SCALE simulation used a traditional "Job Slot" system (Spec 009), where agents were explicitly assigned to buildings (e.g., "Assign to Farm"). This approach presented several architectural challenges:

1.  **Rigidity:** It was difficult to interrupt work for vital needs (eating, sleeping) without complex state machine overrides.
2.  **Lack of Agency:** Agents felt like "workers" rather than "inhabitants." They had no autonomy to prioritize their own survival over their assigned job.
3.  **Scalability:** Adding new behaviors (socializing, exploring) required patching the core job loop, leading to a monolithic "AI God Class."

We needed a system that allowed for *emergent behavior*, where agents could dynamically weigh their physiological needs against their colony duties.

## Decision

We have adopted an **Emergent Utility AI System** (based on Spec 016) to drive all agent decision-making.

The core logic is:
`Utility = f(Internal Needs, External Context, Learned Weights)`

1.  **Evaluation Loop:** Periodically, every agent evaluates a list of possible actions (Eat, Sleep, Work, Idle).
2.  **Scoring:** Each action generates a "Utility Score" (0.0 - 1.0).
    *   *Need-based actions* (Eat) score based on urgency (Need Response Curve).
    *   *Work actions* score based on distance, skill, and priority.
3.  **Selection:** The agent greedily selects the action with the highest Utility.
4.  **Reinforcement Learning:** Agents track the outcome of their plans. Successful actions increase the weights for that action type (e.g., "Farm A is a good place to eat"), while failures decrease them.

## Consequences

### Positive
*   **Decoupled Architecture:** New behaviors are added simply by defining a new `ActionType` and a corresponding `evaluate_X` function. The core loop remains unchanged.
*   **Emergent Life:** Agents naturally create schedules (sleep at night, eat when hungry) without explicit scripting.
*   **Resilience:** If a job site is destroyed, agents naturally fallback to other behaviors instead of crashing or getting stuck in a null reference.

### Negative
*   **Performance Cost:** Evaluating *every* possible action for *every* agent is computationally more expensive than a static assignment list. Optimization strategies (spatial hashing, staggered evaluation) are required.
*   **Nondeterminism:** It is harder to write integration tests for specific sequences ("Agent A *will* go to Farm B") because the decision depends on dynamic fuzzy weights.
*   **Debugging Complexity:** "Why did the agent starve?" becomes a complex question involving weight history and competing utility scores, rather than a simple logic error.

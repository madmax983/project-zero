# 18. Faction System & State Injection

Date: 2024-06-05

## Status

Accepted

## Context

The simulation originally modeled Pops as individual agents driven solely by personal Needs (Hunger, Rest). However, the design requires "Social Stratification" and "Collective Bargaining" (Spec 051, Spec 113). We needed a mechanism where:
1.  Pops organize based on their economic role (Jobs/Skills).
2.  These groups (Factions) can make demands of the player (Policies).
3.  If demands are unmet, the group can collectively disrupt the colony (Strikes).

The existing Utility AI assigns actions based on *individual* scores. A Pop might *want* to farm because they are hungry or it's their job. A "Strike" implies they *have* the job but *refuse* to do it. We needed a way to interrupt this execution without rewriting the entire Utility AI to "understand" politics.

## Decision

We implemented the **Faction System** with a **State Injection Pattern**.

1.  **Resource-Based State:** Factions are stored in a global `Factions` resource, tracking `Satisfaction`, `Demands`, and `State` (Loyal, Unhappy, Striking).
2.  **Component Membership:** Pops have a `FactionMember` component, automatically updated based on their highest Skill (e.g., Mining -> Miners' Guild).
3.  **Execution-Level Interruption (State Injection):**
    Instead of preventing the *assignment* of work (Utility AI), we prevent the *execution* of work.
    Systems that perform labor (e.g., `produce_food_system`, `mine_rock_system`) must:
    *   Inject `Option<Res<Factions>>` and `Option<&FactionMember>`.
    *   Check if the Pop's faction is in `FactionState::Striking`.
    *   If striking, the system **skips logic execution** (yields 0 produce) but the Pop remains in the `Work` action state (effectively "standing around").

## Consequences

### Positive
*   **Decoupling:** The Utility AI (`evaluate_work`) doesn't need to know about complex political states. It assigns work as normal.
*   **Emergence:** Pops appear to "protest" by occupying the workplace but producing nothing.
*   **Centralization:** Faction logic is contained in `factions.rs`, and systems only need a read-only check.

### Negative
*   **Boilerplate:** Every "productive" system (Farming, Mining, Crafting, Construction) must manually implement the Faction check. Forgetting one allows "scabs" (striking pops who still work).
*   **Opacity:** Without UI feedback, it might look like a bug (Pop is at farm, Action says "Farming", but no food appears). We rely on the Inspector/Notifications to explain the Strike status.
